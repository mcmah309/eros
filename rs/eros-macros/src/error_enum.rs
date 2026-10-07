use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Generics, Ident, ItemEnum, ItemType, LitStr, Meta, Type};

#[derive(Clone)]
pub(crate) struct ErrorEnumArgs {
    name: Ident,
    display: Option<LitStr>,
}

impl syn::parse::Parse for ErrorEnumArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(input.error("expected an enum name"));
        }
        let name = input.parse()?;
        let mut display = None;
        if !input.is_empty() {
            input.parse::<syn::Token![,]>()?;
            if !input.is_empty() {
                display = Some(input.parse()?);
            }
        }
        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
        }
        Ok(Self { name, display })
    }
}

#[derive(Clone, Copy)]
pub(crate) enum EnumKind {
    Owned,
    Ref,
    Mut,
    All,
}

impl EnumKind {
    fn indices(self) -> &'static [usize] {
        match self {
            Self::Owned => &[0],
            Self::Ref => &[1],
            Self::Mut => &[2],
            Self::All => &[0, 1, 2],
        }
    }

    fn marker(self) -> &'static str {
        match self {
            Self::Owned => "error_enum",
            Self::Ref => "error_enum_ref",
            Self::Mut => "error_enum_mut",
            Self::All => "error_enums",
        }
    }

    fn from_path(path: &syn::Path) -> Option<Self> {
        match path.segments.last()?.ident.to_string().as_str() {
            "error_enum" => Some(Self::Owned),
            "error_enum_ref" => Some(Self::Ref),
            "error_enum_mut" => Some(Self::Mut),
            "error_enums" => Some(Self::All),
            _ => None,
        }
    }
}

struct EnumSpec {
    args: ErrorEnumArgs,
    attrs: Vec<Attribute>,
}

fn request_enums(
    kind: EnumKind,
    args: ErrorEnumArgs,
    enums: &mut [Option<EnumSpec>; 3],
    source: impl quote::ToTokens,
) -> syn::Result<()> {
    for &index in kind.indices() {
        if enums[index].is_some() {
            return Err(syn::Error::new_spanned(
                source,
                format!(
                    "{} overlaps an enum already requested on this tuple alias",
                    kind.marker()
                ),
            ));
        }
    }
    for &index in kind.indices() {
        let mut args = args.clone();
        if matches!(kind, EnumKind::All) && index != 0 {
            let base = args.name.to_string();
            let base = base.trim_start_matches("r#");
            let suffix = if index == 1 { "Ref" } else { "Mut" };
            args.name = format_ident!("{base}{suffix}", span = args.name.span());
        }
        enums[index] = Some(EnumSpec {
            args,
            attrs: Vec::new(),
        });
    }
    Ok(())
}

fn collect_annotations(
    mut kind: EnumKind,
    tokens: TokenStream,
    alias: &mut ItemType,
) -> syn::Result<[Option<EnumSpec>; 3]> {
    let args: ErrorEnumArgs = syn::parse2(tokens)?;
    let mut enums = [None, None, None];
    request_enums(kind, args, &mut enums, &alias.ident)?;
    for attr in std::mem::take(&mut alias.attrs) {
        if let Some(next_kind) = EnumKind::from_path(attr.path()) {
            let tokens = match &attr.meta {
                Meta::Path(_) => TokenStream::new(),
                Meta::List(list) => list.tokens.clone(),
                Meta::NameValue(_) => {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "expected an enum macro with a name in parentheses",
                    ));
                }
            };
            let args = syn::parse2(tokens)?;
            request_enums(next_kind, args, &mut enums, &attr)?;
            kind = next_kind;
        } else {
            for &index in kind.indices() {
                enums[index].as_mut().unwrap().attrs.push(attr.clone());
            }
        }
    }
    Ok(enums)
}

pub(crate) fn expand_alias(
    kind: EnumKind,
    tokens: TokenStream,
    mut alias: ItemType,
) -> syn::Result<TokenStream> {
    let alias_gating = gating_attrs(&alias.attrs)?;
    let enums = collect_annotations(kind, tokens, &mut alias)?;
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

    let mut generated_names = HashSet::new();
    let alias_name = alias.ident.to_string();
    let alias_name = alias_name.trim_start_matches("r#");
    for spec in enums.iter().flatten() {
        let name = spec.args.name.to_string();
        let name = name.trim_start_matches("r#");
        if name == alias_name {
            return Err(syn::Error::new_spanned(
                &spec.args.name,
                "generated enum name must differ from the tuple alias name",
            ));
        }
        if !generated_names.insert(name.to_owned()) {
            return Err(syn::Error::new_spanned(
                &spec.args.name,
                "generated enum names must be distinct",
            ));
        }
    }

    alias.attrs = alias_gating;
    let vis = &alias.vis;
    let crate_path = eros_path()?;
    let mut targets = [None, None, None];
    let mut gating = [Vec::new(), Vec::new(), Vec::new()];
    let mut declarations = Vec::new();
    for (index, spec) in enums.iter().enumerate() {
        let Some(spec) = spec else { continue };
        let name = &spec.args.name;
        let attrs = &spec.attrs;
        gating[index] = gating_attrs(attrs)?;
        let generics: Generics = if index == 0 {
            Generics::default()
        } else {
            syn::parse_quote!(<'__eros_enum>)
        };
        let (_, type_generics, _) = generics.split_for_impl();
        targets[index] = Some(quote!(#name #type_generics));
        let payloads = types.iter().map(|ty| match index {
            0 => quote!(#ty),
            1 => quote!(&'__eros_enum #ty),
            _ => quote!(&'__eros_enum mut #ty),
        });
        let traits = error_traits(
            name,
            &generics,
            &variants,
            spec.args.display.as_ref(),
            &gating[index],
            index != 0,
        );
        declarations.push(quote! {
            #(#attrs)*
            #[derive(::core::fmt::Debug)]
            #vis enum #name #generics {
                #(#variants(#payloads)),*
            }
            #traits
        });
    }
    let conversions = conversions(
        targets,
        &Generics::default(),
        &types,
        &variants,
        &crate_path,
        [&gating[0], &gating[1], &gating[2]],
    );
    Ok(quote! {
        #alias
        #(#declarations)*
        #conversions
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
            Some(quote!(#name<#(#types),*>)),
            Some(quote!(#name<#(&'__eros_enum #types),*>)),
            Some(quote!(#name<#(&'__eros_enum mut #types),*>)),
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
    targets: [Option<TokenStream>; 3],
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
    let owned_conversion = owned_enum.map(|owned_enum| {
        quote! {
            #(#owned_gating)*
            impl #impl_generics ::core::convert::From<#union_type> for #owned_enum #where_clause {
                #[inline]
                fn from(union_of: #union_type) -> Self {
                    #owned
                }
            }
        }
    });
    let ref_conversion = ref_enum.map(|ref_enum| {
        quote! {
            #(#ref_gating)*
            impl #borrow_generics ::core::convert::From<&'__eros_enum #union_type>
                for #ref_enum #borrow_where
            {
                #[inline]
                fn from(union_of: &'__eros_enum #union_type) -> Self {
                    #shared
                }
            }
        }
    });
    let mut_conversion = mut_enum.map(|mut_enum| {
        quote! {
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
    });
    quote! {
        #owned_conversion
        #ref_conversion
        #mut_conversion
    }
}
