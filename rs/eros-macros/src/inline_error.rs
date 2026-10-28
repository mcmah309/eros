use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Ident, Item, LitStr, Token,
    parse::{Parse, ParseStream},
};

pub(crate) struct Args {
    message: Option<LitStr>,
    arguments: TokenStream,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self {
                message: None,
                arguments: TokenStream::new(),
            });
        }
        let message = Some(input.parse()?);
        let arguments = if input.is_empty() {
            TokenStream::new()
        } else {
            input.parse::<Token![,]>()?;
            input.parse()?
        };
        Ok(Self { message, arguments })
    }
}

pub(crate) fn expand(args: Args, item: Item) -> syn::Result<TokenStream> {
    let Args { message, arguments } = args;
    let (name, generics) = match &item {
        Item::Struct(item) => (&item.ident, &item.generics),
        Item::Enum(item) => (&item.ident, &item.generics),
        _ => {
            return Err(syn::Error::new_spanned(
                item,
                "inline_error requires a struct or enum",
            ));
        }
    };
    if !generics.params.is_empty() || generics.where_clause.is_some() {
        return Err(syn::Error::new_spanned(
            generics,
            "inline_error requires a nongeneric struct or enum",
        ));
    }
    let display = message.map(|message| {
        let formatter = Ident::new("__eros_formatter", Span::mixed_site());
        quote! {
            impl ::core::fmt::Display for #name {
                fn fmt(&self, #formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    ::core::write!(#formatter, #message, #arguments)
                }
            }
        }
    });
    Ok(quote! {
        #[derive(::core::fmt::Debug)]
        #item

        const _: () = {
            ::core::assert!(
                ::core::mem::size_of::<#name>() <= ::core::mem::size_of::<::core::primitive::usize>(),
                ::core::concat!(
                    "inline_error: `", ::core::stringify!(#name),
                    "` exceeds the one-usize size limit (including padding and enum discriminants)",
                ),
            );
            ::core::assert!(
                ::core::mem::size_of::<#name>() == 0
                    || ::core::mem::align_of::<#name>() <= ::core::mem::align_of::<::core::primitive::usize>(),
                ::core::concat!(
                    "inline_error: `", ::core::stringify!(#name),
                    "` exceeds the alignment of usize (zero-sized errors support any alignment)",
                ),
            );
        };

        #display

        impl ::core::error::Error for #name {}
    })
}
