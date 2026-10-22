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

/// Keeps a tuple alias and generates the explicitly named owned error enum.
///
/// ```rust
/// use std::fmt;
/// #[eros_macros::error_enum(AppFailure, "operation failed: {0}")]
/// #[non_exhaustive]
/// type AppErrors = (std::io::Error, fmt::Error);
///
/// let union: eros::ErrorUnion<AppErrors> = eros::ErrorUnion::new(fmt::Error);
/// match union.into() {
///     AppFailure::StdIo(error) => eprintln!("I/O error: {error}"),
///     AppFailure::Fmt(error) => assert_eq!(error, fmt::Error),
/// }
/// ```
///
/// The name is required and used exactly as written. An optional display format
/// follows it: `#[error_enum(AppFailure, "operation failed: {0}")]`. Without a
/// format, `Display` delegates to the contained error, preserving formatter flags.
/// `{0}` or `{}` formats the contained error; fixed strings and escaped braces
/// also work. `Debug`, `Display`, and `core::error::Error` are implemented
/// automatically, and `Error::source()` returns the contained error.
///
/// `From<ErrorUnion<Subset>>` converts any tuple of the enum's error types into
/// the enum, in any order. Shared and mutable references work the same way.
/// `TryFrom<ErrorUnion<AnyError>>` checks the concrete inner error and returns
/// the original union on a mismatch. Successful owned conversions discard
/// context, location, and backtrace. The shared and mutable enum macros generate
/// corresponding conversions from references, returning the original borrow
/// on a mismatch and preserving diagnostics.
///
/// The alias must be a nongeneric tuple of 1–26 path types. Variant names join
/// path segments in PascalCase, ignoring generic arguments, then strip a trailing
/// `Error` unless that would leave an empty name. Attributes below
/// each macro apply to its enum until the next enum macro or the alias.
/// [`error_enum_ref`] and [`error_enum_mut`] accept the same name and optional
/// display arguments, work independently, and can be stacked in any order:
///
/// ```rust
/// #[eros_macros::error_enum_mut(MutableFailure)]
/// #[non_exhaustive]
/// #[eros_macros::error_enum(OwnedFailure)]
/// #[non_exhaustive]
/// #[eros_macros::error_enum_ref(SharedFailure)]
/// #[derive(Clone, Copy)]
/// type AppErrors = (std::fmt::Error,);
///
/// let mut union: eros::ErrorUnion<AppErrors> = eros::ErrorUnion::new(std::fmt::Error);
/// let shared: SharedFailure<'_> = (&union).into();
/// let _copy = shared;
/// let _shared_again = shared;
/// let _mutable: MutableFailure<'_> = (&mut union).into();
/// let _owned: OwnedFailure = union.into();
/// ```
///
/// Only explicitly requested enums and conversions are generated.
/// Use [`error_enum_kind`] to generate an enum with payload-free variants.
/// Rust evaluates `cfg` and `cfg_attr` before macro expansion, so disabling
/// conditions anywhere in the declaration remove the alias and its enums.
#[proc_macro_attribute]
pub fn error_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_error_enum(error_enum::EnumKind::Owned, attr, item)
}

/// Keeps a tuple alias and generates the explicitly named shared error enum.
/// Accepts the same arguments as [`error_enum`]; subsequent attributes apply to
/// this enum until the next enum macro or the alias.
///
/// ```rust
/// #[eros_macros::error_enum_ref(SharedFailure, "shared: {0}")]
/// #[derive(Clone, Copy)]
/// type AppErrors = (std::fmt::Error,);
/// let union: eros::ErrorUnion<AppErrors> = eros::ErrorUnion::new(std::fmt::Error);
/// match (&union).into() {
///     SharedFailure::StdFmt(error) => assert_eq!(error, &std::fmt::Error),
/// }
/// ```
#[proc_macro_attribute]
pub fn error_enum_ref(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_error_enum(error_enum::EnumKind::Ref, attr, item)
}

/// Keeps a tuple alias and generates the explicitly named mutable error enum.
/// Accepts the same arguments as [`error_enum`]; subsequent attributes apply to
/// this enum until the next enum macro or the alias.
///
/// ```rust
/// #[eros_macros::error_enum_mut(MutableFailure, "mutable: {0}")]
/// type AppErrors = (std::fmt::Error,);
/// let mut union: eros::ErrorUnion<AppErrors> = eros::ErrorUnion::new(std::fmt::Error);
/// match (&mut union).into() {
///     MutableFailure::StdFmt(error) => *error = std::fmt::Error,
/// }
/// ```
#[proc_macro_attribute]
pub fn error_enum_mut(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_error_enum(error_enum::EnumKind::Mut, attr, item)
}

/// Keeps a tuple alias and generates the explicitly named error kind enum.
///
/// Every variant is a unit variant: the enum stores no error payloads and has
/// no lifetime parameter. Names follow the same rules as [`error_enum`]. Only
/// an enum name is accepted; `Debug` is implemented automatically. Additional
/// attributes apply to this enum until the next enum macro or the alias.
///
/// `From` accepts owned, shared, and mutable `ErrorUnion` values whose tuple
/// contains only listed error types, including subsets and reordered tuples.
/// Borrowing leaves the error and its diagnostics intact. Owned conversion
/// drops the error and its diagnostics. `TryFrom` accepts erased unions in all
/// three forms and returns the original value or borrow when no type matches.
///
/// ```rust
/// use std::{fmt, io};
/// #[eros_macros::error_enum_kind(AppErrorKind)]
/// #[derive(Clone, Copy, PartialEq, Eq)]
/// type AppErrors = (io::Error, fmt::Error);
///
/// let union: eros::ErrorUnion<AppErrors> = eros::ErrorUnion::new(fmt::Error);
/// assert_eq!(AppErrorKind::from(&union), AppErrorKind::Fmt);
/// assert!(union.is_inner::<fmt::Error>());
/// ```
#[proc_macro_attribute]
pub fn error_enum_kind(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_error_enum(error_enum::EnumKind::Kind, attr, item)
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
                #crate_path::__private::formatted_message!(#message)
            }
            .into();
        }
        literal.push(ch);
    }

    // All braces were escaped pairs; unescape them in the static message.
    let literal = LitStr::new(&literal, message.span());
    quote! { #crate_path::MsgError::from_static_ref(&#literal) }.into()
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
