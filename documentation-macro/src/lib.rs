use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Expr, Lit, Meta, MetaNameValue};

#[proc_macro_derive(Documentation)]
pub fn documentation_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    impl_documentation(&input)
}

fn impl_documentation(ast: &DeriveInput) -> TokenStream {
    let crate_ = match std::env::var("CARGO_PKG_NAME") {
        Ok(v) if v == "archk" => quote! { crate },
        _ => quote!( ::archk ),
    };
    
    let name = &ast.ident;
    // TODO: description

    let fields = if let syn::Data::Struct(data) = &ast.data {
        data.fields
            .iter()
            .map(|field| {
                let doc: String = field
                    .attrs
                    .iter()
                    .flat_map(|attr| {
                        if attr.path().is_ident("doc") {
                            let Meta::NameValue(MetaNameValue { value, .. }) = &attr.meta else {
                                return None;
                            };
                            let Expr::Lit(syn::ExprLit {
                                lit: Lit::Str(s), ..
                            }) = value
                            else {
                                return None;
                            };

                            Some(s.value())
                        } else {
                            None
                        }
                    })
                    .collect();
                let ty = field.ty.to_token_stream();
                let name = field.ident.as_ref().map(|v| v.to_string()).unwrap_or_else(|| "0".into());

                quote! { 
                    #crate_::v1::docs::DocumentationField {
                        name: #name,
                        documentation:
                            <#ty as #crate_::v1::docs::Documentation>::DOCUMENTATION_OBJECT.set_description(#doc)
                    }
                }
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let name_str = name.to_string();

    let gen = quote! {
        impl #crate_::v1::docs::Documentation for #name {
            const DOCUMENTATION_OBJECT: #crate_::v1::docs::DocumentationObject = #crate_::v1::docs::DocumentationObject::new(
                #name_str,
                "", // description
                &[
                    #(#fields),*
                ]
            );
        }
    };
    gen.into()
}
