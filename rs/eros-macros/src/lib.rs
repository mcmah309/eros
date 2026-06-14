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
struct ContextArgs {
    format_str: LitStr,
    format_args: Punctuated<Expr, Comma>,
}

impl syn::parse::Parse for ContextArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let format_str: LitStr = input.parse()?;

        // Collect any trailing `, arg1, arg2, ...`
        let format_args = if input.peek(Token![,]) {
            let _comma: Token![,] = input.parse()?;
            Punctuated::<Expr, Comma>::parse_terminated(input)?
        } else {
            Punctuated::new()
        };

        Ok(ContextArgs {
            format_str,
            format_args,
        })
    }
}

/// Automatically wraps a function body with `eros` context.
///
/// ```rust,ignore
/// #[context("Param 1 is {} and param2 is {:?}", param1, param2)]
/// fn function_name(param1: &str, param2: i32) -> eros::Result<()> {
///     // ...
/// }
/// ```
///
/// Expands to:
///
/// ```rust,ignore
/// #[doc(hidden)]
/// #[track_caller]
/// fn __function_name_internal(param1: &str, param2: i32) -> eros::Result<()> {
///     // ...
/// }
///
/// fn function_name(param1: &str, param2: i32) -> eros::Result<()> {
///     use eros::Context as _;
///     __function_name_internal(param1, param2)
///         .with_context(|| format!("Param 1 is {} and param2 is {:?}", param1, param2))
/// }
/// ```
///
/// # Example — async method with `&mut self`
///
/// ```rust,ignore
/// #[context("push failed, queue len {}", self.items.len())]
/// async fn push(&mut self, item: String) -> eros::Result<()> {
///     // ...
/// }
/// ```
///
/// Expands to (inside the same `impl` block):
///
/// ```rust,ignore
/// #[doc(hidden)]
/// #[track_caller]
/// async fn __push_internal(&mut self, item: String) -> eros::Result<()> {
///     // original body — `self` is the real receiver, no renaming needed
/// }
///
/// async fn push(&mut self, item: String) -> eros::Result<()> {
///     use eros::Context as _;
///     self.__push_internal(item)
///         .await
///         .with_context(|| format!("push failed, queue len {}", self.items.len()))
/// }
/// ```
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
    let ContextArgs {
        format_str,
        format_args,
    } = args;

    let vis = &func.vis;
    let sig = &func.sig;
    let attrs = &func.attrs;
    let is_async = sig.asyncness.is_some();

    let outer_name = &sig.ident;
    let inner_name = syn::Ident::new(
        &format!("__{}_internal", outer_name),
        outer_name.span(),
    );

    let has_receiver = matches!(
        sig.inputs.first(),
        Some(syn::FnArg::Receiver(_))
    );

    let mut inner_sig = sig.clone();
    inner_sig.ident = inner_name.clone();
    let body = &func.block;

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

    // ── Format call ─────────────────────────────────────────────────────────

    let format_call = if format_args.is_empty() {
        quote! { format!(#format_str) }
    } else {
        quote! { format!(#format_str, #format_args) }
    };

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

    let expanded = quote! {
        #[doc(hidden)]
        #[track_caller]
        #inner_sig #body

        #(#attrs)*
        #vis #sig {
            use eros::Context as _;
            #awaited_call.with_context(|| #format_call)
        }
    };

    Ok(expanded)
}