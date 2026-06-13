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
/// # Example
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
/// fn function_name(param1: &str, param2: i32) -> eros::Result<()> {
///     #[track_caller]
///     fn __function_name_internal(param1: &str, param2: i32) -> eros::Result<()> {
///         // ...
///     }
///     use eros::Context;
///     __function_name_internal(param1, param2)
///         .with_context(|| format!("Param 1 is {} and param2 is {:?}", param1, param2))
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

    let outer_name = &sig.ident;
    let inner_name = syn::Ident::new(&format!("__{}_internal", outer_name), outer_name.span());

    let call_args: Vec<TokenStream2> = sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => {
                let pat = &pat_type.pat;
                Some(quote! { #pat })
            }
            // `self` receivers are not forwarded; the inner fn won't have one.
            syn::FnArg::Receiver(_) => None,
        })
        .collect();

    let mut inner_sig = sig.clone();
    inner_sig.ident = inner_name.clone();

    let body = &func.block;

    let format_call = if format_args.is_empty() {
        quote! { format!(#format_str) }
    } else {
        quote! { format!(#format_str, #format_args) }
    };

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            #[track_caller]
            #inner_sig #body

            use eros::Context as _;
            #inner_name(#(#call_args),*).with_context(|| #format_call)
        }
    };

    Ok(expanded)
}
