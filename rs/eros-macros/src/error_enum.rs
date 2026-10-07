use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Generics, Ident, ItemEnum, ItemType, LitStr, Meta, Type};

#[derive(Default)]
pub(crate) struct ErrorEnumArgs {
    name: Option<Ident>,
    display: Option<LitStr>,
}

impl syn::parse::Parse for ErrorEnumArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();
        if input.is_empty() {
            return Ok(args);
        }
        if input.peek(LitStr) {
            args.display = Some(input.parse()?);
        } else {
            args.name = Some(input.parse()?);
            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
                if !input.is_empty() {
                    args.display = Some(input.parse()?);
                }
            }
        }
        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
        }
        Ok(args)
    }
}

pub(crate) fn expand_alias(args: ErrorEnumArgs, mut alias: ItemType) -> syn::Result<TokenStream> {
    if !alias.generics.params.is_empty() || alias.generics.where_clause.is_some() {
        return Err(syn::Error::new_spanned(
            &alias.generics,
            "error_enum requires a nongeneric tuple alias",
        ));
    }
    let Type::Tuple(tuple) = alias.ty.as_ref() else {
        return Err(syn::Error::new_spanned(
            &alias.ty,
            "error_enum requires a tuple alias",
        ));
    };
    validate_arity(tuple.elems.len(), tuple)?;

    let types: Vec<_> = tuple.elems.iter().cloned().collect();
    let mut names = HashSet::new();
    let variants = types
        .iter()
        .map(|ty| {
            let Type::Path(path) = ty else {
                return Err(syn::Error::new_spanned(
                    ty,
                    "error_enum variants must be path types",
                ));
            };
            if path.qself.is_some() {
                return Err(syn::Error::new_spanned(
                    ty,
                    "error_enum does not support qualified associated types",
                ));
            }
            let name: String = path
                .path
                .segments
                .iter()
                .map(|segment| {
                    let ident = segment.ident.to_string();
                    let ident = ident.strip_prefix("r#").unwrap_or(&ident);
                    ident
                        .split('_')
                        .filter(|word| !word.is_empty())
                        .map(|word| {
                            let mut chars = word.chars();
                            let first = chars.next().unwrap().to_uppercase().to_string();
                            first + chars.as_str()
                        })
                        .collect::<String>()
                })
                .collect();
            if !names.insert(name.clone()) {
                return Err(syn::Error::new_spanned(
                    ty,
                    format!("duplicate generated variant `{name}`; use distinct type aliases"),
                ));
            }
            syn::parse_str::<Ident>(&name)
                .map(|mut ident| {
                    ident.set_span(path.path.segments.last().unwrap().ident.span());
                    ident
                })
                .map_err(|_| syn::Error::new_spanned(ty, "cannot generate a variant name"))
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let enum_name = args.name.unwrap_or_else(|| {
        format_ident!("{}Error", alias.ident.to_string().trim_start_matches("r#"))
    });
    if enum_name == alias.ident {
        return Err(syn::Error::new_spanned(
            &enum_name,
            "generated enum name must differ from the tuple alias name",
        ));
    }
    // Defaults resolve in the enum's scope, so payload parameters must not
    // shadow any identifiers in the original error types.
    fn type_idents(tokens: TokenStream, names: &mut HashSet<String>) {
        for token in tokens {
            match token {
                proc_macro2::TokenTree::Ident(ident) => {
                    names.insert(ident.to_string().trim_start_matches("r#").to_owned());
                }
                proc_macro2::TokenTree::Group(group) => type_idents(group.stream(), names),
                _ => {}
            }
        }
    }
    let mut used = HashSet::new();
    used.insert(enum_name.to_string().trim_start_matches("r#").to_owned());
    type_idents(quote!(#(#types)*), &mut used);
    let payloads: Vec<_> = (0..types.len())
        .map(|i| {
            let mut name = format!("E{i}");
            while used.contains(&name) {
                name.push('_');
            }
            format_ident!("{name}")
        })
        .collect();
    let gating = gating_attrs(&alias.attrs)?;
    let attrs = std::mem::replace(&mut alias.attrs, gating.clone());
    let vis = &alias.vis;
    let crate_path = eros_path()?;
    let conversions = conversions(
        &enum_name,
        &Generics::default(),
        &types,
        &variants,
        &crate_path,
        &gating,
    );
    let traits = error_traits(
        &enum_name,
        &payloads,
        &variants,
        args.display.as_ref(),
        &gating,
    );

    Ok(quote! {
        #alias
        #(#attrs)*
        #[derive(::core::fmt::Debug)]
        #vis enum #enum_name<#(#payloads = #types),*> {
            #(#variants(#payloads)),*
        }
        #conversions
        #traits
    })
}

fn error_traits(
    name: &Ident,
    payloads: &[Ident],
    variants: &[Ident],
    display: Option<&LitStr>,
    gating: &[Attribute],
) -> TokenStream {
    // A fixed message (including escaped braces) takes no formatting argument.
    // Otherwise Rust's formatter validates the format string against the one
    // positional argument: the variant's contained error.
    let value = display.map(LitStr::value).unwrap_or_default();
    let mut chars = value.chars().peekable();
    let mut uses_payload = false;
    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                chars.next();
            } else {
                uses_payload = true;
                break;
            }
        }
    }
    let arms: Vec<_> = variants
        .iter()
        .map(|variant| match display {
            None => quote!(Self::#variant(error) => ::core::fmt::Display::fmt(error, f)),
            Some(display) if uses_payload => {
                quote!(Self::#variant(error) => ::core::write!(f, #display, error))
            }
            Some(display) => quote!(Self::#variant(_) => ::core::write!(f, #display)),
        })
        .collect();
    quote! {
        #(#gating)*
        impl<#(#payloads: ::core::fmt::Display + ::core::fmt::Debug),*>
            ::core::fmt::Display for #name<#(#payloads),*>
        {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self { #(#arms),* }
            }
        }
        #(#gating)*
        impl<#(#payloads: ::core::error::Error + 'static),*>
            ::core::error::Error for #name<#(#payloads),*>
        {
            fn source(&self) -> ::core::option::Option<&(dyn ::core::error::Error + 'static)> {
                match self {
                    #(Self::#variants(error) => ::core::option::Option::Some(error)),*
                }
            }
        }
    }
}

pub(crate) fn expand_numbered(item: ItemEnum) -> syn::Result<TokenStream> {
    validate_arity(item.variants.len(), &item)?;
    let mut types = Vec::new();
    let mut variants = Vec::new();
    for variant in &item.variants {
        let syn::Fields::Unnamed(fields) = &variant.fields else {
            return Err(syn::Error::new_spanned(variant, "expected a tuple variant"));
        };
        if fields.unnamed.len() != 1 {
            return Err(syn::Error::new_spanned(variant, "expected one payload"));
        }
        types.push(fields.unnamed[0].ty.clone());
        variants.push(variant.ident.clone());
    }
    let conversions = conversions(
        &item.ident,
        &item.generics,
        &types,
        &variants,
        &quote!(crate),
        &gating_attrs(&item.attrs)?,
    );
    Ok(quote! { #item #conversions })
}

fn validate_arity(count: usize, item: impl quote::ToTokens) -> syn::Result<()> {
    if !(1..=26).contains(&count) {
        return Err(syn::Error::new_spanned(
            item,
            "error_enum requires between 1 and 26 variants",
        ));
    }
    Ok(())
}

fn eros_path() -> syn::Result<TokenStream> {
    match proc_macro_crate::crate_name("eros") {
        Ok(proc_macro_crate::FoundCrate::Itself) => Ok(quote!(::eros)),
        Ok(proc_macro_crate::FoundCrate::Name(name)) => {
            let ident = format_ident!("{name}");
            Ok(quote!(::#ident))
        }
        Err(error) => Err(syn::Error::new(proc_macro2::Span::call_site(), error)),
    }
}

fn gating_attrs(attrs: &[Attribute]) -> syn::Result<Vec<Attribute>> {
    fn filter(meta: &Meta) -> syn::Result<Option<Meta>> {
        if meta.path().is_ident("cfg") {
            return Ok(Some(meta.clone()));
        }
        if meta.path().is_ident("cfg_attr") {
            let Meta::List(list) = meta else {
                return Ok(None);
            };
            let args = list.parse_args_with(
                syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
            )?;
            if let Some(condition) = args.first() {
                let nested = args
                    .iter()
                    .skip(1)
                    .filter_map(|arg| filter(arg).transpose())
                    .collect::<syn::Result<Vec<_>>>()?;
                if !nested.is_empty() {
                    return Ok(Some(syn::parse_quote!(cfg_attr(#condition, #(#nested),*))));
                }
            }
        }
        Ok(None)
    }
    attrs
        .iter()
        .filter_map(|attr| {
            filter(&attr.meta)
                .transpose()
                .map(|result| result.map(|meta| syn::parse_quote!(#[#meta])))
        })
        .collect()
}

fn conversions(
    name: &Ident,
    generics: &Generics,
    types: &[Type],
    variants: &[Ident],
    crate_path: &TokenStream,
    gating: &[Attribute],
) -> TokenStream {
    let mut bounded = generics.clone();
    for ty in types {
        bounded
            .make_where_clause()
            .predicates
            .push(syn::parse_quote!(#ty: #crate_path::SendSyncError));
    }
    let (impl_generics, _, where_clause) = bounded.split_for_impl();
    let mut borrowed = bounded.clone();
    borrowed.params.insert(0, syn::parse_quote!('__eros_enum));
    let (borrow_generics, _, borrow_where) = borrowed.split_for_impl();
    let union_type = quote!(#crate_path::ErrorUnion<(#(#types,)*)>);

    let dispatch = |method: Ident| {
        let last = types.len() - 1;
        let branches = variants.iter().zip(types).take(last).map(|(variant, ty)| {
            // SAFETY: The dispatch condition checks the exact concrete type.
            quote! {
                if union_of.is_inner::<#ty>() {
                    return Self::#variant(unsafe {
                        #crate_path::__private::#method::<#ty>(union_of)
                    });
                }
            }
        });
        let last_type = &types[last];
        let last_variant = &variants[last];
        // SAFETY: ErrorUnion's sealed type-set relations and invariant type
        // parameter guarantee it contains one of these exact types. After
        // ruling out every earlier type, only the last one remains. This also
        // covers singletons without a dispatch check.
        quote! {
            #(#branches)*
            Self::#last_variant(unsafe {
                #crate_path::__private::#method::<#last_type>(union_of)
            })
        }
    };
    let owned = dispatch(format_ident!("downcast_error_unchecked"));
    let shared = dispatch(format_ident!("downcast_error_ref_unchecked"));
    let mutable = dispatch(format_ident!("downcast_error_mut_unchecked"));
    quote! {
        #(#gating)*
        impl #impl_generics ::core::convert::From<#union_type> for #name<#(#types),*> #where_clause {
            fn from(union_of: #union_type) -> Self {
                #owned
            }
        }
        #(#gating)*
        impl #borrow_generics ::core::convert::From<&'__eros_enum #union_type>
            for #name<#(&'__eros_enum #types),*> #borrow_where
        {
            fn from(union_of: &'__eros_enum #union_type) -> Self {
                #shared
            }
        }
        #(#gating)*
        impl #borrow_generics ::core::convert::From<&'__eros_enum mut #union_type>
            for #name<#(&'__eros_enum mut #types),*> #borrow_where
        {
            fn from(union_of: &'__eros_enum mut #union_type) -> Self {
                #mutable
            }
        }
    }
}
