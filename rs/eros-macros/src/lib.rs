// Enables feature flag documentation on things in docs.rs https://github.com/rust-lang/rust/issues/43781 http://doc.rust-lang.org/rustdoc/unstable-features.html#doccfg-and-docauto_cfg
#![cfg_attr(docsrs, feature(doc_cfg))]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Expr, ItemFn, LitStr, Token, parse::ParseStream, parse_macro_input, punctuated::Punctuated,
    token::Comma,
};

mod error_enum;

/// Keeps a tuple alias and generates a named error enum with union conversions.
///
/// ```rust
/// use std::fmt;
/// #[eros_macros::error_enum("operation failed: {0}")]
/// #[non_exhaustive]
/// type Name = (std::io::Error, fmt::Error);
///
/// let union: eros::ErrorUnion<Name> = eros::ErrorUnion::new(fmt::Error);
/// let error = NameError::from(union);
/// assert!(matches!(error, NameError::FmtError(_)));
/// assert_eq!(error.to_string(), "operation failed: an error occurred when formatting an argument");
/// ```
///
/// Pass an identifier to override the generated enum name:
///
/// ```rust
/// #[eros_macros::error_enum(PublicError)]
/// type Internal = (std::fmt::Error,);
/// let union: eros::ErrorUnion<Internal> = eros::ErrorUnion::new(std::fmt::Error);
/// let error = PublicError::from(union);
/// assert_eq!(error.to_string(), std::fmt::Error.to_string());
/// ```
///
/// A custom name can also be followed by a display format string:
/// `#[error_enum(PublicError, "operation failed: {0}")]`.
///
/// The alias must be a nongeneric tuple of 1–26 path types. Variant names join
/// the path segments in PascalCase (ignoring generic arguments). This attribute
/// generates only `NameError` with concrete payloads. Add [`error_enum_ref`] for
/// `NameErrorRef<'a>` with shared references, or [`error_enum_mut`] for
/// `NameErrorMut<'a>` with mutable references. Convert with
/// `NameError::from(union_of)`, `NameErrorRef::from(&union_of)`, or
/// `NameErrorMut::from(&mut union_of)`. Custom names use the same `Ref`/`Mut` suffixes.
/// With no arguments (`#[error_enum]` or `#[error_enum()]`), `Display` delegates
/// directly to the contained error, preserving the formatter's flags. An
/// optional format string can customize the message: `{0}` (or `{}`) formats
/// the contained error. Fixed strings and escaped braces also work. `Debug`,
/// `Display`, and `core::error::Error` are implemented automatically, with the contained error
/// returned by `Error::source`, including for borrowed enums.
/// Attributes below this macro apply to the owned enum until the next enum marker.
/// Rust evaluates `cfg` and `cfg_attr` before macro expansion, so disabling
/// conditions anywhere in the declaration remove the alias and all requested enums.
///
/// Use [`error_enum_ref`] and [`error_enum_mut`] to generate the shared or mutable
/// enum and direct the following annotations to it. Each marker takes annotations until the next
/// enum marker or the tuple alias:
///
/// ```rust
/// #[eros_macros::error_enum]
/// #[non_exhaustive]
/// #[eros_macros::error_enum_ref]
/// #[derive(Clone, Copy)]
/// #[non_exhaustive]
/// #[eros_macros::error_enum_mut]
/// #[non_exhaustive]
/// type Name = (std::io::Error, std::fmt::Error);
///
/// let mut union_of: eros::ErrorUnion<Name> = eros::ErrorUnion::new(std::fmt::Error);
/// let shared = NameErrorRef::from(&union_of);
/// let shared_copy = shared;
/// assert!(matches!(shared, NameErrorRef::StdFmtError(_)));
/// assert!(matches!(shared_copy, NameErrorRef::StdFmtError(_)));
/// let mutable = NameErrorMut::from(&mut union_of);
/// assert!(matches!(mutable, NameErrorMut::StdFmtError(_)));
/// ```
///
/// Each marker works on its own and may appear at most once, in any order.
/// Only the requested enums and conversions are generated. Set the name and display
/// on `error_enum`; the borrowed markers take no arguments. Without `error_enum`,
/// borrowed names use the tuple alias followed by `ErrorRef` or `ErrorMut`, and
/// `Display` delegates to the contained error. Automatic `Debug`, `Display`, and
/// `Error` implementations apply to every requested enum.
#[proc_macro_attribute]
pub fn error_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_error_enum(error_enum::EnumKind::Owned, attr, item)
}

/// Keeps a tuple alias and generates its shared error enum and union conversion.
/// Subsequent annotations apply to that enum, stopping at the next enum marker
/// or the tuple alias.
///
/// This marker takes no arguments and works on its own or alongside [`error_enum`]
/// and [`error_enum_mut`] in any order. See [`error_enum`] for a stacked example.
///
/// ```rust
/// #[eros_macros::error_enum_ref]
/// #[derive(Clone, Copy)]
/// type Name = (std::fmt::Error,);
/// let union: eros::ErrorUnion<Name> = eros::ErrorUnion::new(std::fmt::Error);
/// match (&union).into() {
///     NameErrorRef::StdFmtError(error) => assert_eq!(error, &std::fmt::Error),
/// }
/// ```
#[proc_macro_attribute]
pub fn error_enum_ref(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_error_enum(error_enum::EnumKind::Ref, attr, item)
}

/// Keeps a tuple alias and generates its mutable error enum and union conversion.
/// Subsequent annotations apply to that enum, stopping at the next enum marker
/// or the tuple alias.
///
/// This marker takes no arguments and works on its own or alongside [`error_enum`]
/// and [`error_enum_ref`] in any order. See [`error_enum`] for a stacked example.
///
/// ```rust
/// #[eros_macros::error_enum_mut]
/// type Name = (std::fmt::Error,);
/// let mut union: eros::ErrorUnion<Name> = eros::ErrorUnion::new(std::fmt::Error);
/// match (&mut union).into() {
///     NameErrorMut::StdFmtError(error) => *error = std::fmt::Error,
/// }
/// ```
#[proc_macro_attribute]
pub fn error_enum_mut(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_error_enum(error_enum::EnumKind::Mut, attr, item)
}

fn expand_error_enum(
    kind: error_enum::EnumKind,
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let alias = parse_macro_input!(item as syn::ItemType);
    match error_enum::expand_alias(kind, attr.into(), alias) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Generates the legacy numbered enums' conversions at their declarations.
#[doc(hidden)]
#[proc_macro_attribute]
pub fn __error_enum(_: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemEnum);
    match error_enum::expand_numbered(item) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

struct FormatErrorInput {
    crate_path: syn::Path,
    message: Expr,
}

impl syn::parse::Parse for FormatErrorInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let crate_path = input.parse()?;
        input.parse::<Comma>()?;
        let message = input.parse()?;
        Ok(Self {
            crate_path,
            message,
        })
    }
}

/// Selects storage for the single-argument arm of `eros::error!` during macro expansion.
#[doc(hidden)]
#[proc_macro]
pub fn format_error(input: TokenStream) -> TokenStream {
    let FormatErrorInput {
        crate_path,
        message,
    } = parse_macro_input!(input as FormatErrorInput);
    // Expressions forwarded through macro_rules! can have invisible groups.
    let mut expr = &message;
    while let Expr::Group(group) = expr {
        expr = &group.expr;
    }

    let Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(message),
        ..
    }) = expr
    else {
        return quote! { #message }.into();
    };
    let value = message.value();
    let mut chars = value.chars();
    let mut literal = String::with_capacity(value.len());

    while let Some(ch) = chars.next() {
        if matches!(ch, '{' | '}') && chars.next() != Some(ch) {
            // Preserve the original literal's span for implicit captures. Let
            // Rust's formatter parse placeholders and report invalid formats.
            return quote! {
                #crate_path::MsgError::from_owned(#crate_path::__private::format!(#message))
            }
            .into();
        }
        literal.push(ch);
    }

    // All braces were escaped pairs; unescape them in the static message.
    let literal = LitStr::new(&literal, message.span());
    quote! { #crate_path::MsgError::from_static(#literal) }.into()
}

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

enum ContextMode {
    Lazy,
    Eager,
}

impl ContextMode {
    fn name(&self) -> &'static str {
        match self {
            Self::Lazy => "context",
            Self::Eager => "eager_context",
        }
    }
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
/// ## Auto format string from parameter attributes
///
/// When no format string is provided, annotate individual parameters with
/// `#[fmt("...")]` to build the context string automatically.
/// Each annotated parameter contributes one `"<name>: <specifier>\n"` line.
///
/// ```rust,ignore
/// #[context]
/// fn process(#[fmt("{}")] name: &str, count: usize, #[fmt("{:?}")] flags: &Flags) -> eros::Result<()> {
///     // ...
/// }
/// ```
///
/// Expands to:
///
/// ```rust,ignore
/// #[doc(hidden)]
/// #[track_caller]
/// fn __process_internal(name: &str, count: usize, flags: &Flags) -> eros::Result<()> {
///     // ...
/// }
///
/// fn process(name: &str, count: usize, flags: &Flags) -> eros::Result<()> {
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
    context_impl(attr, item, ContextMode::Lazy)
}

/// Like [`context`], but formats before the body runs (including success) when context is enabled.
#[proc_macro_attribute]
pub fn eager_context(attr: TokenStream, item: TokenStream) -> TokenStream {
    context_impl(attr, item, ContextMode::Eager)
}

fn context_impl(attr: TokenStream, item: TokenStream, mode: ContextMode) -> TokenStream {
    let args = parse_macro_input!(attr as ContextArgs);
    let func = parse_macro_input!(item as ItemFn);

    match expand_context(args, func, mode) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_context(args: ContextArgs, func: ItemFn, mode: ContextMode) -> syn::Result<TokenStream2> {
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

    let format_call = match args {
        ContextArgs::Explicit {
            format_str,
            format_args,
        } => {
            if format_args.is_empty() {
                quote! { eros::__private::format!(#format_str) }
            } else {
                quote! { eros::__private::format!(#format_str, #format_args) }
            }
        }

        ContextArgs::Auto => {
            if annotated.is_empty() {
                return Err(syn::Error::new_spanned(
                    &func.sig.ident,
                    format!(
                        "`#[{}]` with no format string requires at least one parameter \
                         annotated with `#[fmt(\"...\")]`",
                        mode.name(),
                    ),
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
            quote! { eros::__private::format!(#fmt_lit, #(#arg_idents),*) }
        }
    };

    if !cfg!(feature = "context") {
        func.block.stmts.insert(
            0,
            syn::parse_quote! {
                if false {
                    let _ = #format_call;
                }
            },
        );
        return Ok(quote! { #func });
    }

    let is_async = func.sig.asyncness.is_some();
    let outer_name = &func.sig.ident;
    let inner_name = syn::Ident::new(&format!("__{}_internal", outer_name), outer_name.span());
    let has_receiver = matches!(func.sig.inputs.first(), Some(syn::FnArg::Receiver(_)));

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

    let context_call = match mode {
        ContextMode::Lazy => quote! { #awaited_call.with_context(|| #format_call) },
        ContextMode::Eager => {
            let message = syn::Ident::new("__eros_context", proc_macro2::Span::mixed_site());
            quote! {
                let #message = #format_call;
                #awaited_call.context(#message)
            }
        }
    };

    Ok(quote! {
        #[doc(hidden)]
        #[track_caller]
        #inner_sig #body

        #(#attrs)*
        #vis #sig {
            use eros::Context as _;
            #context_call
        }
    })
}
