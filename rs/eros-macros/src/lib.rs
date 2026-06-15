// Enables feature flag documentation on things in docs.rs https://github.com/rust-lang/rust/issues/43781 http://doc.rust-lang.org/rustdoc/unstable-features.html#doccfg-and-docauto_cfg
#![cfg_attr(docsrs, feature(doc_cfg))]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Expr, ItemFn, LitStr, Token, parse::ParseStream, parse_macro_input, punctuated::Punctuated,
    token::Comma,
};

/// Arguments parsed from `#[context("format string", arg1, arg2, ...)]`
/// or `#[context]` / `#[context()]` (auto-build from `#[fmt("...")]`
/// parameter attributes).
enum ContextArgs {
    /// Explicit format string (and optional extra arguments).
    Explicit {
        format_str: LitStr,
        format_args: Punctuated<Expr, Comma>,
    },
    /// No format string supplied — derive from `#[fmt("...")]`
    /// annotations on individual parameters.
    Auto,
}

impl syn::parse::Parse for ContextArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(ContextArgs::Auto);
        }

        let format_str: LitStr = input.parse()?;

        let format_args = if input.peek(Token![,]) {
            let _comma: Token![,] = input.parse()?;
            Punctuated::<Expr, Comma>::parse_terminated(input)?
        } else {
            Punctuated::new()
        };

        Ok(ContextArgs::Explicit {
            format_str,
            format_args,
        })
    }
}

/// The custom format specifier string extracted from `#[fmt("...")]`.
struct ParamFmt {
    /// The raw format string contents, e.g. `"{}"`, `"{:?}"`, `"{:.2}"`.
    specifier: String,
}

/// If `expr` is of the form `<ident>.clone()` with no other method chaining,
/// returns `Some(ident)`. Otherwise returns `None`.
fn extract_clone_ident(expr: &Expr) -> Option<&syn::Ident> {
    let Expr::MethodCall(mc) = expr else {
        return None;
    };
    if mc.method != "clone" || !mc.args.is_empty() || mc.turbofish.is_some() {
        return None;
    }
    let Expr::Path(path_expr) = mc.receiver.as_ref() else {
        return None;
    };
    path_expr.path.get_ident()
}

/// Automatically wraps a function body with `eros` context.
///
/// ## Explicit format string
///
/// ```rust,ignore
/// #[context("Param 1 is {} and param2 is {:?}", param1, param2)]
/// fn function_name(param1: &str, param2: i32) -> eros::Result<()> {
///     // ...
/// }
/// ```
///
/// ## Cloning owned values
///
/// When passing an owned value that would be moved into the function, use
/// `.clone()` in the format args. The macro will clone the value before the
/// inner call and use the original (un-cloned) expression inside
/// `with_context`, avoiding borrow-checker conflicts.
///
/// ```rust,ignore
/// #[context("Processing: {}", value.clone())]
/// fn process(value: String) -> eros::Result<()> {
///     // ...
/// }
/// ```
///
/// Expands to:
///
/// ```rust,ignore
/// fn process(value: String) -> eros::Result<()> {
///     let value_cloned = value.clone();
///     __process_internal(value_cloned)
///         .with_context(|| format!("Processing: {}", value))
/// }
/// ```
///
/// ## Auto format string from parameter attributes
///
/// When no format string is provided, annotate individual parameters with
/// `#[fmt("...")]` to build the context string automatically.
/// Each annotated parameter contributes one `"<name>: <specifier>\n"` line.
///
/// ```rust,ignore
/// #[context]
/// fn process(#[fmt("{}")] name: &str, count: usize, #[fmt("{:?}")] flags: Flags) -> eros::Result<()> {
///     // ...
/// }
/// ```
///
/// Expands to:
///
/// ```rust,ignore
/// #[doc(hidden)]
/// #[track_caller]
/// fn __process_internal(name: &str, count: usize, flags: Flags) -> eros::Result<()> {
///     // ...
/// }
///
/// fn process(name: &str, count: usize, flags: Flags) -> eros::Result<()> {
///     use eros::Context as _;
///     __process_internal(name, count, flags)
///         .with_context(|| format!("name: {}\nflags: {:?}\n", name, flags))
/// }
/// ```
///
/// ## Async and `self` receivers
///
/// Both modes work with `async fn` and all receiver kinds (`self`, `&self`,
/// `&mut self`). Two sibling items are emitted so that `self` in the body
/// always refers to the real receiver — no aliasing required.
#[proc_macro_attribute]
pub fn context(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ContextArgs);
    let func = parse_macro_input!(item as ItemFn);

    match expand_context(args, func) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_context(args: ContextArgs, func: ItemFn) -> syn::Result<TokenStream2> {
    let is_async = func.sig.asyncness.is_some();
    let outer_name = &func.sig.ident;
    let inner_name = syn::Ident::new(&format!("__{}_internal", outer_name), outer_name.span());

    let has_receiver = matches!(func.sig.inputs.first(), Some(syn::FnArg::Receiver(_)));

    struct AnnotatedParam {
        ident: syn::Ident,
        fmt: ParamFmt,
    }

    let mut func = func;
    let mut annotated: Vec<AnnotatedParam> = Vec::new();

    for arg in func.sig.inputs.iter_mut() {
        let syn::FnArg::Typed(pat_type) = arg else {
            continue;
        };

        let mut fmt_specifier: Option<String> = None;
        let mut duplicate = false;

        pat_type.attrs.retain(|attr| {
            if !attr.path().is_ident("fmt") {
                return true;
            }

            if fmt_specifier.is_some() {
                duplicate = true;
                return false;
            }

            let lit = attr.parse_args::<LitStr>();
            match lit {
                Ok(l) => {
                    fmt_specifier = Some(l.value());
                }
                Err(_) => {
                    duplicate = true;
                }
            }

            false
        });

        if duplicate {
            return Err(syn::Error::new_spanned(
                &pat_type.pat,
                "a parameter may have at most one `#[fmt(\"...\")]` attribute, \
                 and it must contain a single string literal",
            ));
        }

        let specifier = match fmt_specifier {
            Some(s) => s,
            None => continue,
        };

        let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
            return Err(syn::Error::new_spanned(
                &pat_type.pat,
                "`#[fmt(\"...\")]` is only supported on simple identifier patterns",
            ));
        };

        annotated.push(AnnotatedParam {
            ident: pat_ident.ident.clone(),
            fmt: ParamFmt { specifier },
        });
    }

    struct CloneBinding {
        /// `ident_cloned` — the name of the pre-cloned local.
        clone_ident: syn::Ident,
        /// The original `ident.clone()` expression, used to initialise the
        /// binding and to replace the format-arg in `with_context`.
        clone_expr: Expr,
        /// The bare `ident`, used as the call argument to the inner function.
        bare_ident: syn::Ident,
    }

    let (clone_bindings, format_call): (Vec<CloneBinding>, TokenStream2) = match args {
        ContextArgs::Explicit {
            format_str,
            format_args,
        } => {
            let mut bindings: Vec<CloneBinding> = Vec::new();

            // Rewritten args for the `format!` inside `with_context` — clones
            // are stripped back to bare idents.
            let context_args: Vec<Expr> = format_args
                .iter()
                .map(|expr| {
                    if let Some(ident) = extract_clone_ident(expr) {
                        let clone_ident =
                            syn::Ident::new(&format!("{}_cloned", ident), ident.span());
                        // Only push once per unique ident.
                        if !bindings.iter().any(|b| b.bare_ident == *ident) {
                            bindings.push(CloneBinding {
                                clone_ident: clone_ident.clone(),
                                clone_expr: expr.clone(),
                                bare_ident: ident.clone(),
                            });
                        }
                        syn::parse_quote!(#ident)
                    } else {
                        expr.clone()
                    }
                })
                .collect();

            let fmt_call = if context_args.is_empty() {
                quote! { format!(#format_str) }
            } else {
                quote! { format!(#format_str, #(#context_args),*) }
            };

            (bindings, fmt_call)
        }

        ContextArgs::Auto => {
            if annotated.is_empty() {
                return Err(syn::Error::new_spanned(
                    &func.sig.ident,
                    "`#[context]` with no format string requires at least one parameter \
                     annotated with `#[fmt(\"...\")]`",
                ));
            }

            // Build `"param1: {}\nparam2: {:.2}\n"` and the matching arg list.
            let mut fmt_str = String::new();
            let mut arg_idents: Vec<&syn::Ident> = Vec::new();

            for ap in &annotated {
                fmt_str.push_str(&format!("{}: {}\n", ap.ident, ap.fmt.specifier));
                arg_idents.push(&ap.ident);
            }

            let fmt_lit = syn::LitStr::new(&fmt_str, proc_macro2::Span::call_site());
            (vec![], quote! { format!(#fmt_lit, #(#arg_idents),*) })
        }
    };

    let vis = &func.vis;
    let attrs = &func.attrs;
    let sig = &func.sig;
    let body = &func.block;

    let mut inner_sig = sig.clone();
    inner_sig.ident = inner_name.clone();

    let call_args: Vec<TokenStream2> = sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => {
                let pat = &pat_type.pat;
                if let syn::Pat::Ident(pat_ident) = pat.as_ref()
                    && let Some(binding) = clone_bindings
                        .iter()
                        .find(|b| b.bare_ident == pat_ident.ident)
                {
                    let ci = &binding.clone_ident;
                    return Some(quote! { #ci });
                }
                Some(quote! { #pat })
            }
            syn::FnArg::Receiver(_) => None,
        })
        .collect();

    let raw_call = if has_receiver {
        quote! { self.#inner_name(#(#call_args),*) }
    } else {
        quote! { #inner_name(#(#call_args),*) }
    };

    let awaited_call = if is_async {
        quote! { #raw_call.await }
    } else {
        raw_call
    };

    let clone_let_stmts: Vec<TokenStream2> = clone_bindings
        .iter()
        .map(|b| {
            let ci = &b.clone_ident;
            let ce = &b.clone_expr;
            quote! { let #ci = #ce; }
        })
        .collect();

    Ok(quote! {
        #[doc(hidden)]
        #[track_caller]
        #inner_sig #body

        #(#attrs)*
        #vis #sig {
            use eros::Context as _;
            #(#clone_let_stmts)*
            #awaited_call.with_context(|| #format_call)
        }
    })
}
