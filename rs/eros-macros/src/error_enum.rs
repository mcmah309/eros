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

#[derive(Clone, Copy)]
pub(crate) enum EnumKind {
    Owned,
    Ref,
    Mut,
}

impl EnumKind {
    fn from_path(path: &syn::Path) -> Option<Self> {
        match path.segments.last()?.ident.to_string().as_str() {
            "error_enum" => Some(Self::Owned),
            "error_enum_ref" => Some(Self::Ref),
            "error_enum_mut" => Some(Self::Mut),
            _ => None,
        }
    }

    fn parse_args(self, tokens: TokenStream) -> syn::Result<ErrorEnumArgs> {
        if matches!(self, Self::Owned) {
            syn::parse2(tokens)
        } else if tokens.is_empty() {
            Ok(ErrorEnumArgs::default())
        } else {
            Err(syn::Error::new_spanned(
                tokens,
                "borrowed enum markers take no arguments; place annotations below the marker",
            ))
        }
    }
}

#[derive(Default)]
struct EnumAnnotations {
    owned: Vec<Attribute>,
    shared: Vec<Attribute>,
    mutable: Vec<Attribute>,
}

fn collect_annotations(
    mut kind: EnumKind,
    tokens: TokenStream,
    alias: &mut ItemType,
) -> syn::Result<(ErrorEnumArgs, EnumAnnotations)> {
    let args = kind.parse_args(tokens)?;
    let mut owned_args = matches!(kind, EnumKind::Owned).then_some(args);
    let mut annotations = EnumAnnotations::default();
    for attr in std::mem::take(&mut alias.attrs) {
        if let Some(next_kind) = EnumKind::from_path(attr.path()) {
            let tokens = match &attr.meta {
                Meta::Path(_) => TokenStream::new(),
                Meta::List(list) => list.tokens.clone(),
                Meta::NameValue(_) => {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "expected an enum marker, optionally followed by parentheses",
                    ));
                }
            };
            let args = next_kind.parse_args(tokens)?;
            if matches!(next_kind, EnumKind::Owned) {
                if owned_args.is_some() {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "error_enum must appear only once on a tuple alias",
                    ));
                }
                owned_args = Some(args);
            }
            kind = next_kind;
        } else {
            match kind {
                EnumKind::Owned => &mut annotations.owned,
                EnumKind::Ref => &mut annotations.shared,
                EnumKind::Mut => &mut annotations.mutable,
            }
            .push(attr);
        }
    }
    let args = owned_args.ok_or_else(|| {
        syn::Error::new_spanned(
            &alias.ident,
            "borrowed enum markers require #[eros::error_enum] on the same tuple alias",
        )
    })?;
    Ok((args, annotations))
}

pub(crate) fn expand_alias(
    kind: EnumKind,
    tokens: TokenStream,
    mut alias: ItemType,
) -> syn::Result<TokenStream> {
    let (args, annotations) = collect_annotations(kind, tokens, &mut alias)?;
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
    let base_name = enum_name.to_string();
    let base_name = base_name.trim_start_matches("r#");
    let ref_name = format_ident!("{base_name}Ref", span = enum_name.span());
    let mut_name = format_ident!("{base_name}Mut", span = enum_name.span());
    if [&enum_name, &ref_name, &mut_name].contains(&&alias.ident) {
        return Err(syn::Error::new_spanned(
            &enum_name,
            "generated enum name must differ from the tuple alias name",
        ));
    }
    let EnumAnnotations {
        owned: attrs,
        shared: ref_attrs,
        mutable: mut_attrs,
    } = annotations;
    let gating = gating_attrs(&attrs)?;
    alias.attrs = gating.clone();
    let mut ref_gating = gating.clone();
    ref_gating.extend(gating_attrs(&ref_attrs)?);
    let mut mut_gating = gating.clone();
    mut_gating.extend(gating_attrs(&mut_attrs)?);
    let vis = &alias.vis;
    let crate_path = eros_path()?;
    let conversions = conversions(
        [
            quote!(#enum_name),
            quote!(#ref_name<'__eros_enum>),
            quote!(#mut_name<'__eros_enum>),
        ],
        &Generics::default(),
        &types,
        &variants,
        &crate_path,
        [&gating, &ref_gating, &mut_gating],
    );
    let owned_traits = error_traits(
        &enum_name,
        &Generics::default(),
        &variants,
        args.display.as_ref(),
        &gating,
        false,
    );
    let borrow_generics = syn::parse_quote!(<'__eros_enum>);
    let ref_traits = error_traits(
        &ref_name,
        &borrow_generics,
        &variants,
        args.display.as_ref(),
        &ref_gating,
        true,
    );
    let mut_traits = error_traits(
        &mut_name,
        &borrow_generics,
        &variants,
        args.display.as_ref(),
        &mut_gating,
        true,
    );

    Ok(quote! {
        #alias
        #(#attrs)*
        #[derive(::core::fmt::Debug)]
        #vis enum #enum_name {
            #(#variants(#types)),*
        }
        #(#gating)*
        #(#ref_attrs)*
        #[derive(::core::fmt::Debug)]
        #vis enum #ref_name<'__eros_enum> {
            #(#variants(&'__eros_enum #types)),*
        }
        #(#gating)*
        #(#mut_attrs)*
        #[derive(::core::fmt::Debug)]
        #vis enum #mut_name<'__eros_enum> {
            #(#variants(&'__eros_enum mut #types)),*
        }
        #conversions
        #owned_traits
        #ref_traits
        #mut_traits
    })
}

fn error_traits(
    name: &Ident,
    generics: &Generics,
    variants: &[Ident],
    display: Option<&LitStr>,
    gating: &[Attribute],
    borrowed: bool,
) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    // Return the concrete error, rather than the reference payload. This lets
    // borrowed enums implement Error without requiring the borrow to be 'static.
    let source = if borrowed {
        quote!(&**error)
    } else {
        quote!(error)
    };
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
        impl #impl_generics ::core::fmt::Display for #name #type_generics #where_clause
        {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self { #(#arms),* }
            }
        }
        #(#gating)*
        impl #impl_generics ::core::error::Error for #name #type_generics #where_clause
        {
            fn source(&self) -> ::core::option::Option<&(dyn ::core::error::Error + 'static)> {
                match self {
                    #(Self::#variants(error) => ::core::option::Option::Some(#source)),*
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
    let name = &item.ident;
    let gating = gating_attrs(&item.attrs)?;
    let conversions = conversions(
        [
            quote!(#name<#(#types),*>),
            quote!(#name<#(&'__eros_enum #types),*>),
            quote!(#name<#(&'__eros_enum mut #types),*>),
        ],
        &item.generics,
        &types,
        &variants,
        &quote!(crate),
        [&gating, &gating, &gating],
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
    targets: [TokenStream; 3],
    generics: &Generics,
    types: &[Type],
    variants: &[Ident],
    crate_path: &TokenStream,
    gating: [&[Attribute]; 3],
) -> TokenStream {
    let [owned_enum, ref_enum, mut_enum] = targets;
    let [owned_gating, ref_gating, mut_gating] = gating;
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
        #(#owned_gating)*
        impl #impl_generics ::core::convert::From<#union_type> for #owned_enum #where_clause {
            #[inline]
            fn from(union_of: #union_type) -> Self {
                #owned
            }
        }
        #(#ref_gating)*
        impl #borrow_generics ::core::convert::From<&'__eros_enum #union_type>
            for #ref_enum #borrow_where
        {
            #[inline]
            fn from(union_of: &'__eros_enum #union_type) -> Self {
                #shared
            }
        }
        #(#mut_gating)*
        impl #borrow_generics ::core::convert::From<&'__eros_enum mut #union_type>
            for #mut_enum #borrow_where
        {
            #[inline]
            fn from(union_of: &'__eros_enum mut #union_type) -> Self {
                #mutable
            }
        }
    }
}
