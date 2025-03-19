//! Derive macro for `FromMultiPart`

use darling::{FromDeriveInput, FromField, ast::Data, util::Ignored};
use proc_macro::TokenStream;
use syn::{DeriveInput, Error, Ident};
use thiserror::Error;

#[proc_macro_derive(FromMultiPart, attributes(multipart))]
pub fn derive_from_multi_part(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as syn::DeriveInput);

    match generate(args) {
        Ok(stream) => stream,
        Err(err) => err.write_errors().into(),
    }
}

fn generate(args: DeriveInput) -> Result<TokenStream, GeneratorError> {
    let args: FromMultiPartArgs = FromMultiPartArgs::from_derive_input(&args)?;

    let struct_ident = args.ident;

    let structure = match args.data {
        Data::Enum(_) => Err(Error::new_spanned(
            &struct_ident,
            "FromMultiPart can only be applied to an struct.",
        ))?,
        Data::Struct(fields) => fields,
    };

    let mut fields = vec![];

    for field_data in structure.fields {
        let ident = match field_data.ident {
            Some(ident) => ident,
            None => Err(Error::new_spanned(
                &struct_ident,
                "FromMultiPart does not work for tuple structs.",
            ))?,
        };

        fields.push((
            ident.clone(),
            field_data.rename.unwrap_or_else(|| ident.to_string()),
        ))
    }

    let field_names = fields
        .iter()
        .map(|(ident, _)| ident.clone())
        .collect::<Vec<_>>();

    let field_declarations = fields
        .iter()
        .map(|(ident, name)| quote::quote! { let #ident = map.get(#name)?; })
        .collect::<Vec<_>>();

    let stream = quote::quote! {
        impl poem_typed_multipart::FromMultiPart for #struct_ident {
            fn decode(map: poem_typed_multipart::map::MultiPartMap) -> Result<Self, poem::Error> {
                #(#field_declarations)*

                Ok(Self { #(#field_names),* })
            }
        }
    };

    Ok(stream.into())
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(multipart), forward_attrs(doc))]
struct FromMultiPartArgs {
    ident: Ident,
    data: Data<Ignored, FromMultiPartData>,
}

#[derive(Debug, FromField)]
#[darling(attributes(multipart), forward_attrs(doc))]
struct FromMultiPartData {
    ident: Option<Ident>,
    #[darling(default)]
    rename: Option<String>,
}

#[derive(Debug, Error)]
pub(crate) enum GeneratorError {
    #[error("{0}")]
    Syn(#[from] syn::Error),
    #[error("{0}")]
    Darling(#[from] darling::Error),
}

impl GeneratorError {
    pub(crate) fn write_errors(self) -> proc_macro2::TokenStream {
        match self {
            GeneratorError::Syn(err) => err.to_compile_error(),
            GeneratorError::Darling(err) => err.write_errors(),
        }
    }
}
