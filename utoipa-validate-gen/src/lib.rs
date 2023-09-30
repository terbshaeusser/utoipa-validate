use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Attribute, Data, DeriveInput, Field, Fields, GenericParam,
    Generics, Index, Meta, MetaNameValue, Token, Type,
};

#[proc_macro_derive(Validatable)]
pub fn derive_validatable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let validator_name_str = format!("{}Validator", name.to_string());
    let validator_name = Ident::new(&validator_name_str, name.span());
    let checks = create_checks(&name, input.data);

    let output = quote! {
        impl #impl_generics utoipa_validate::Validatable for #name #ty_generics #where_clause {
            type DefaultValidator = #validator_name;
        }

        #[derive(Default)]
        pub struct #validator_name {}

        impl utoipa_validate::Validator<#name> for #validator_name {
            fn validate(&self, path: &utoipa_validate::ValidationPath, value: &#name, errors: &mut std::vec::Vec<utoipa_validate::ValidationError>) {
                #checks
            }
        }
    };

    proc_macro::TokenStream::from(output)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(utoipa_validate::Validatable));
        }
    }

    generics
}

fn create_checks(self_type_name: &Ident, data: Data) -> TokenStream {
    match data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                let recurse = fields.named.into_iter().map(|field| {
                    let span = field.span();
                    let field_name = field.ident.clone().unwrap();
                    let field_name_str = field_name.to_string();
                    let checks = create_checks_for_field(
                        field,
                        quote! {
                            value.#field_name
                        },
                        quote! {
                            utoipa_validate::ValidationPath::Field {
                                parent: path,
                                name: #field_name_str,
                            }
                        },
                    );

                    quote_spanned! {span=>
                        #checks
                    }
                });

                quote! {
                    #(#recurse)*
                }
            }
            Fields::Unnamed(fields) => {
                let recurse = fields
                    .unnamed
                    .into_iter()
                    .enumerate()
                    .map(|(index, field)| {
                        let span = field.span();
                        let field_index = Index::from(index);
                        let field_index_str = index.to_string();
                        let checks = create_checks_for_field(
                            field,
                            quote! {
                                value.#field_index
                            },
                            quote! {
                                utoipa_validate::ValidationPath::Field {
                                    parent: path,
                                    name: #field_index_str,
                                }
                            },
                        );

                        quote_spanned! {span=>
                            #checks
                        }
                    });

                quote! {
                    #(#recurse)*
                }
            }
            Fields::Unit => {
                quote!()
            }
        },
        Data::Enum(data) => {
            let recurse = data.variants.into_iter().map(|variant| {
                let variant_name = variant.ident;

                let fields =
                    match variant.fields.iter().next() {
                        None => quote!(),
                        Some(first_field) => {
                            let is_tuple = first_field.ident.is_none();

                            let fields = variant.fields.clone().into_iter().enumerate().map(
                                |(index, field)| {
                                    field.ident.unwrap_or_else(|| generate_field_name(index))
                                },
                            );

                            if is_tuple {
                                quote! {
                                    (
                                        #(#fields, )*
                                    )
                                }
                            } else {
                                quote! {
                                    {
                                        #(#fields, )*
                                    }
                                }
                            }
                        }
                    };

                let checks = variant
                    .fields
                    .into_iter()
                    .enumerate()
                    .map(|(index, field)| {
                        let field_name = field
                            .clone()
                            .ident
                            .unwrap_or_else(|| generate_field_name(index));
                        let field_name_str = format!("{}.{}", variant_name, field_name);

                        create_checks_for_field(
                            field,
                            quote! {
                                #field_name
                            },
                            quote! {
                                utoipa_validate::ValidationPath::Field {
                                    parent: path,
                                    name: #field_name_str,
                                }
                            },
                        )
                    });

                quote! {
                    #self_type_name::#variant_name #fields => {
                        #(#checks)*
                    }
                }
            });

            quote! {
                match value {
                    #(#recurse)*
                }
            }
        }
        Data::Union(_) => {
            unimplemented!("Union types are not supported")
        }
    }
}

fn create_checks_for_field(
    field: Field,
    field_expr: TokenStream,
    field_path: TokenStream,
) -> TokenStream {
    let field_type = field.ty;
    let is_option = is_option(&field_type);

    let checks = field.attrs.into_iter().map(|attribute| {
        if attribute.path().is_ident("schema") || attribute.path().is_ident("param") {
            create_checks_for_schema_attribute(&field_expr, is_option, attribute)
        } else {
            quote!()
        }
    });

    quote! {
        {
            let child_path = #field_path;

            <#field_type as utoipa_validate::Validatable>::validate_ex(&#field_expr, &child_path, errors);
            #(#checks)*
        }
    }
}

fn create_checks_for_schema_attribute(
    field_expr: &TokenStream,
    is_option: bool,
    attribute: Attribute,
) -> TokenStream {
    let checks = attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).unwrap().into_iter().map(|meta| {
        match match meta {
            Meta::NameValue(MetaNameValue { path, eq_token, value }) if path.is_ident("exclusive_maximum") => {
                let _ = eq_token;

                Some(quote! {
                    utoipa_validate::ExclusiveMaximumValidator::new(#value)
                })
            }
            Meta::NameValue(MetaNameValue { path, eq_token, value }) if path.is_ident("exclusive_minimum") => {
                let _ = eq_token;

                Some(quote! {
                    utoipa_validate::ExclusiveMinimumValidator::new(#value)
                })
            }
            Meta::NameValue(MetaNameValue { path, eq_token, value }) if path.is_ident("maximum") => {
                let _ = eq_token;

                Some(quote! {
                    utoipa_validate::MaximumValidator::new(#value)
                })
            }
            Meta::NameValue(MetaNameValue { path, eq_token, value }) if path.is_ident("minimum") => {
                let _ = eq_token;

                Some(quote! {
                    utoipa_validate::MinimumValidator::new(#value)
                })
            }
            Meta::NameValue(MetaNameValue { path, eq_token, value }) if path.is_ident("max_items") => {
                let _ = eq_token;

                Some(quote! {
                    utoipa_validate::MaxItemsValidator::new(#value)
                })
            }
            Meta::NameValue(MetaNameValue { path, eq_token, value }) if path.is_ident("min_items") => {
                let _ = eq_token;

                Some(quote! {
                    utoipa_validate::MinItemsValidator::new(#value)
                })
            }
            Meta::NameValue(MetaNameValue { path, eq_token, value }) if path.is_ident("max_length") => {
                let _ = eq_token;

                Some(quote! {
                    utoipa_validate::MaxLengthValidator::new(#value)
                })
            }
            Meta::NameValue(MetaNameValue { path, eq_token, value }) if path.is_ident("min_length") => {
                let _ = eq_token;

                Some(quote! {
                    utoipa_validate::MinLengthValidator::new(#value)
                })
            }
            Meta::NameValue(MetaNameValue { path, eq_token, value }) if path.is_ident("multiple_of") => {
                let _ = eq_token;

                Some(quote! {
                    utoipa_validate::MultipleOfValidator::new(#value)
                })
            }
            Meta::NameValue(MetaNameValue { path, eq_token, value }) if path.is_ident("pattern") => {
                let _ = eq_token;

                Some(quote! {
                    utoipa_validate::PatternValidator::new(regex::Regex::new(#value).unwrap())
                })
            }
            _ => None,
        } {
            None => quote!(),
            Some(validator_expr) => if is_option {
                quote! {
                    utoipa_validate::OptionValidator::new(#validator_expr).validate(&child_path, &#field_expr, errors);
                }
            } else {
                quote! {
                    #validator_expr.validate(&child_path, &#field_expr, errors);
               }
            },
        }
    });

    quote! {
        #(#checks)*
    }
}

fn is_option(t: &Type) -> bool {
    if let Type::Path(path) = t {
        path.path
            .segments
            .last()
            .expect("Expected at least one segment")
            .ident
            .to_string()
            == "Option"
    } else {
        false
    }
}

fn generate_field_name(index: usize) -> Ident {
    let s = format!("_{}", index);
    Ident::new(&s, Span::call_site())
}
