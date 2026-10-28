use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, Ident, Token, Type, Visibility, parse::Parse, parse::ParseStream};

enum Operand {
    Tuple(syn::TypeTuple),
    Set(Ident),
}

struct Definition {
    attrs: Vec<Attribute>,
    vis: Visibility,
    name: Ident,
    operands: Vec<Operand>,
}

impl Parse for Definition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        input.parse::<Token![type]>()?;
        let name = input.parse()?;
        if input.peek(Token![<]) {
            return Err(input.error("type_set! requires nongeneric aliases"));
        }
        input.parse::<Token![=]>()?;
        let mut operands = Vec::new();
        loop {
            let ty: Type = input.parse()?;
            operands.push(match ty {
                Type::Tuple(tuple) => Operand::Tuple(tuple),
                Type::Path(path) if path.qself.is_none() && path.path.get_ident().is_some() => {
                    Operand::Set(path.path.get_ident().unwrap().clone())
                }
                ty => {
                    return Err(syn::Error::new_spanned(
                        ty,
                        "expected a tuple or a set name declared in this type_set! invocation",
                    ));
                }
            });
            if !input.peek(Token![||]) {
                break;
            }
            input.parse::<Token![||]>()?;
        }
        input.parse::<Token![;]>()?;
        Ok(Self {
            attrs,
            vis,
            name,
            operands,
        })
    }
}

pub(crate) struct TypeSets {
    definitions: Vec<Definition>,
}

impl Parse for TypeSets {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut definitions = Vec::new();
        while !input.is_empty() {
            definitions.push(input.parse()?);
        }
        Ok(Self { definitions })
    }
}

impl TypeSets {
    pub(crate) fn expand(self) -> syn::Result<TokenStream> {
        let mut names = HashMap::new();
        for (index, definition) in self.definitions.iter().enumerate() {
            if names.insert(definition.name.to_string(), index).is_some() {
                return Err(syn::Error::new_spanned(
                    &definition.name,
                    format!("duplicate type set `{}`", definition.name),
                ));
            }
        }

        let mut resolved = vec![None; self.definitions.len()];
        let mut visiting = vec![false; self.definitions.len()];
        let mut output = TokenStream::new();
        for (index, definition) in self.definitions.iter().enumerate() {
            let types = self.resolve(index, &names, &mut resolved, &mut visiting)?;
            let Definition {
                attrs, vis, name, ..
            } = definition;
            output.extend(quote! {
                #(#attrs)*
                #vis type #name = (#(#types,)*);
            });
        }
        Ok(output)
    }

    fn resolve(
        &self,
        index: usize,
        names: &HashMap<String, usize>,
        resolved: &mut [Option<Vec<Type>>],
        visiting: &mut [bool],
    ) -> syn::Result<Vec<Type>> {
        if let Some(types) = &resolved[index] {
            return Ok(types.clone());
        }
        visiting[index] = true;
        let definition = &self.definitions[index];
        let mut types = Vec::new();
        let mut seen = HashSet::new();
        for operand in &definition.operands {
            let members = match operand {
                Operand::Tuple(tuple) => tuple.elems.iter().cloned().collect(),
                Operand::Set(name) => {
                    let Some(&target) = names.get(&name.to_string()) else {
                        return Err(syn::Error::new_spanned(
                            name,
                            format!(
                                "unknown type set `{name}`; declare it in this type_set! invocation"
                            ),
                        ));
                    };
                    if visiting[target] {
                        return Err(syn::Error::new_spanned(
                            name,
                            format!("cyclic type set reference to `{name}`"),
                        ));
                    }
                    self.resolve(target, names, resolved, visiting)?
                }
            };
            for ty in members {
                // Spans and whitespace do not affect membership. Rust name
                // resolution is unavailable here, so differently spelled paths
                // and external aliases cannot be recognized as the same type.
                if seen.insert(ty.to_token_stream().to_string()) {
                    types.push(ty);
                }
            }
        }
        if types.len() > 26 {
            return Err(syn::Error::new_spanned(
                &definition.name,
                "type sets support at most 26 distinct members",
            ));
        }
        visiting[index] = false;
        resolved[index] = Some(types.clone());
        Ok(types)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expand(input: TokenStream) -> syn::Result<TokenStream> {
        syn::parse2::<TypeSets>(input)?.expand()
    }

    #[test]
    fn resolves_forward_references_and_deduplicates_in_first_seen_order() {
        let output = expand(quote! {
            /// Combined errors.
            pub(crate) type Combined = Left || Right || (A, D,);
            type Left = (B, A, B);
            type Right = Left || (C, A);
        })
        .unwrap();
        assert_eq!(
            output.to_string(),
            quote! {
                /// Combined errors.
                pub(crate) type Combined = (B, A, C, D,);
                type Left = (B, A,);
                type Right = (B, A, C,);
            }
            .to_string()
        );
    }

    #[test]
    fn accepts_empty_invocations_and_empty_sets() {
        assert!(expand(quote! {}).unwrap().is_empty());
        assert_eq!(
            expand(quote! { type Empty = () || (); })
                .unwrap()
                .to_string(),
            quote! { type Empty = (); }.to_string()
        );
    }

    #[test]
    fn reports_invalid_definitions() {
        for (input, expected) in [
            (quote! { type A = B; }, "unknown type set `B`"),
            (quote! { type A = A; }, "cyclic type set reference to `A`"),
            (
                quote! { type A = B; type B = C; type C = A; },
                "cyclic type set reference to `A`",
            ),
            (
                quote! { type A = (); type A = (); },
                "duplicate type set `A`",
            ),
            (
                quote! { type A<T> = (T,); },
                "type_set! requires nongeneric aliases",
            ),
            (
                quote! { type A = other::Set; },
                "expected a tuple or a set name",
            ),
            (quote! { type A = (B); }, "expected a tuple or a set name"),
            (quote! { type A = () ||; }, "expected one of"),
            (quote! { type A = () }, "expected `;`"),
        ] {
            let error = expand(input).unwrap_err().to_string();
            assert!(
                error.contains(expected),
                "{error:?} should contain {expected:?}"
            );
        }
    }

    #[test]
    fn limits_arity_after_deduplication() {
        let types: Vec<_> = (0..27).map(|i| quote::format_ident!("Error{i}")).collect();
        assert!(
            expand(quote! { type Full = (#(#types,)*); })
                .unwrap_err()
                .to_string()
                .contains("at most 26")
        );
        let types = &types[..26];
        assert!(
            expand(quote! {
                type Full = (#(#types,)*) || (#(#types,)*);
            })
            .is_ok()
        );
    }
}
