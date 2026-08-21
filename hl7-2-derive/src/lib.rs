//! Derive macros for [`hl7-2`](https://crates.io/crates/hl7-2): map a
//! struct's fields to HL7 v2 message paths once, in the type definition,
//! instead of writing the same accessor calls at every call site.
//!
//! This crate is not used directly. `hl7-2` re-exports both macros behind
//! its `derive` feature, so the dependency to add is:
//!
//! ```toml
//! hl7-2 = { version = "0.2", features = ["derive"] }
//! ```
//!
//! Keeping the macros in a crate of their own is what lets the default
//! build of `hl7-2` keep exactly one dependency: `syn` and `quote` are
//! compiled only for callers who ask for the macros.
//!
//! ```ignore
//! use hl7_2::{FromHl7, ToHl7, Raw};
//!
//! #[derive(FromHl7, ToHl7)]
//! struct Result {
//!     #[hl7("OBX-3.1")]   code: String,
//!     #[hl7("OBX-5")]     value: Option<String>,
//!     #[hl7("OBX-6.1")]   units: Option<String>,
//!     #[hl7(nested)]      patient: Patient,   // its own FromHl7
//!     #[hl7(raw)]         raw: Raw,           // the escape hatch
//! }
//! ```
//!
//! One attribute per field, and a field with none is skipped on read
//! (it must implement [`Default`]) and on write:
//!
//! | attribute | on read | on write |
//! |---|---|---|
//! | `#[hl7("PID-5.1")]` | read the path | write the path |
//! | `#[hl7(nested)]` | the field's own `FromHl7` | the field's own `ToHl7` |
//! | `#[hl7(raw)]` | the whole message | skipped |
//! | none | `Default::default()` | skipped |
//!
//! ## When `hl7-2` is not called `hl7_2`
//!
//! The generated code names the crate absolutely, as `::hl7_2`, so that it
//! works wherever the type is defined without the caller importing
//! anything. A caller who renames the dependency —
//! `hl7 = { package = "hl7-2" }` in `Cargo.toml`, or a workspace that
//! aliases it — has no `::hl7_2` for the macro to reach, and the generated
//! code stops compiling. Say where it is instead, once, on the struct:
//!
//! ```ignore
//! #[derive(FromHl7)]
//! #[hl7(crate = hl7)]        // or `crate = "::some::path::to::hl7_2"`
//! struct Patient {
//!     #[hl7("PID-5.1")] family: String,
//! }
//! ```

#![warn(missing_docs, clippy::pedantic)]

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Field, Fields, LitStr, Path, Token, parse_macro_input};

/// Derive `FromHl7`: read each annotated field from its path.
///
/// See the crate documentation for the attributes. Only structs with named
/// fields are supported; a tuple struct has no field names to map, and an
/// enum has no single shape to read.
#[proc_macro_derive(FromHl7, attributes(hl7))]
pub fn derive_from_hl7(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match from_hl7(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Derive `ToHl7`: write each annotated field back to its path.
///
/// Writing needs the segments to exist already; build the message with
/// `hl7_2::Builder` (whose `encode` method takes a `ToHl7`) or add them
/// with `Message::append_segment`.
#[proc_macro_derive(ToHl7, attributes(hl7))]
pub fn derive_to_hl7(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match to_hl7(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// What one field's `#[hl7(...)]` attribute asked for.
enum Mapping {
    /// `#[hl7("PID-5.1")]`
    Path(LitStr),
    /// `#[hl7(nested)]`
    Nested,
    /// `#[hl7(raw)]`
    Raw,
    /// No attribute at all.
    None,
}

fn from_hl7(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let krate = crate_path(input)?;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let mut reads = Vec::new();
    for field in named_fields(input)? {
        let ident = field.ident.as_ref().expect("named fields");
        let ty = &field.ty;
        reads.push(match mapping(field)? {
            Mapping::Path(path) => quote! {
                #ident: <#ty as #krate::FromHl7Value>::from_hl7_value(message, #path)?
            },
            Mapping::Nested => quote! {
                #ident: <#ty as #krate::FromHl7>::from_hl7(message)?
            },
            Mapping::Raw => quote! {
                #ident: #krate::Raw::new(::core::clone::Clone::clone(message)).into()
            },
            Mapping::None => quote! {
                #ident: ::core::default::Default::default()
            },
        });
    }
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #krate::FromHl7 for #name #type_generics #where_clause {
            fn from_hl7(message: &#krate::Message) -> ::core::result::Result<Self, #krate::Error> {
                ::core::result::Result::Ok(#name { #(#reads),* })
            }
        }
    })
}

fn to_hl7(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let krate = crate_path(input)?;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let mut writes = Vec::new();
    for field in named_fields(input)? {
        let ident = field.ident.as_ref().expect("named fields");
        let ty = &field.ty;
        match mapping(field)? {
            Mapping::Path(path) => writes.push(quote! {
                <#ty as #krate::ToHl7Value>::to_hl7_value(&self.#ident, message, #path)?;
            }),
            Mapping::Nested => writes.push(quote! {
                <#ty as #krate::ToHl7>::to_hl7(&self.#ident, message)?;
            }),
            // The raw message is where the struct came from, not something
            // to write back over it.
            Mapping::Raw | Mapping::None => {}
        }
    }
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #krate::ToHl7 for #name #type_generics #where_clause {
            fn to_hl7(
                &self,
                message: &mut #krate::Message,
            ) -> ::core::result::Result<(), #krate::Error> {
                #(#writes)*
                ::core::result::Result::Ok(())
            }
        }
    })
}

/// The struct's named fields, or an error explaining what this macro maps.
fn named_fields(input: &DeriveInput) -> syn::Result<impl Iterator<Item = &Field>> {
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => Ok(named.named.iter()),
            other => Err(syn::Error::new(
                other.span(),
                "hl7-2 derives map field names to HL7 paths, so the struct needs named fields",
            )),
        },
        Data::Enum(_) | Data::Union(_) => Err(syn::Error::new(
            input.ident.span(),
            "hl7-2 derives apply to structs; an enum or union has no single message shape",
        )),
    }
}

/// Read one field's `#[hl7(...)]` attribute.
/// Where the generated code should look for `hl7-2`.
///
/// `::hl7_2` unless the struct says otherwise with `#[hl7(crate = ...)]`,
/// which is what a caller who renamed the dependency needs: the generated
/// code names the crate absolutely so that it compiles wherever the type is
/// defined, and an absolute name that does not exist is a compile error the
/// caller cannot work around from their side.
///
/// The value is a path, written bare (`crate = hl7`) or quoted
/// (`crate = "::vendor::hl7_2"`); the quoted form is there because that is
/// how the rest of the ecosystem spells it.
fn crate_path(input: &DeriveInput) -> syn::Result<Path> {
    for attribute in &input.attrs {
        if !attribute.path().is_ident("hl7") {
            continue;
        }
        return attribute.parse_args_with(|stream: syn::parse::ParseStream| {
            stream.parse::<Token![crate]>().map_err(|_| {
                syn::Error::new(
                    attribute.span(),
                    "the only #[hl7(...)] option on a struct is `crate = ...`; \
                     path attributes belong on fields",
                )
            })?;
            stream.parse::<Token![=]>()?;
            if stream.peek(LitStr) {
                return stream.parse::<LitStr>()?.parse();
            }
            stream.parse()
        });
    }
    Ok(syn::parse_quote!(::hl7_2))
}

fn mapping(field: &Field) -> syn::Result<Mapping> {
    let mut found = Mapping::None;
    for attribute in &field.attrs {
        if !attribute.path().is_ident("hl7") {
            continue;
        }
        if !matches!(found, Mapping::None) {
            return Err(syn::Error::new(
                attribute.span(),
                "a field takes one #[hl7(...)] attribute",
            ));
        }
        // Three spellings: a path literal, `nested`, or `raw`.
        found = attribute.parse_args_with(|input: syn::parse::ParseStream| {
            if input.peek(LitStr) {
                return Ok(Mapping::Path(input.parse()?));
            }
            let word: syn::Ident = input.parse()?;
            match word.to_string().as_str() {
                "nested" => Ok(Mapping::Nested),
                "raw" => Ok(Mapping::Raw),
                other => Err(syn::Error::new(
                    word.span(),
                    format!(
                        "unknown #[hl7(...)] option {other:?}; expected a path such as \
                         #[hl7(\"PID-5.1\")], or `nested`, or `raw`"
                    ),
                )),
            }
        })?;
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;

    fn resolved(attributes: &str) -> String {
        let input: DeriveInput = syn::parse_str(&format!("{attributes} struct S {{ f: u32 }}"))
            .expect("test input parses");
        crate_path(&input)
            .expect("crate path resolves")
            .to_token_stream()
            .to_string()
            .replace(' ', "")
    }

    #[test]
    fn defaults_to_the_absolute_crate_name() {
        assert_eq!(resolved(""), "::hl7_2");
    }

    #[test]
    fn a_bare_path_is_taken_as_written() {
        assert_eq!(resolved("#[hl7(crate = hl7)]"), "hl7");
        assert_eq!(
            resolved("#[hl7(crate = ::vendor::hl7_2)]"),
            "::vendor::hl7_2"
        );
        assert_eq!(resolved("#[hl7(crate = crate::renamed)]"), "crate::renamed");
    }

    #[test]
    fn a_quoted_path_is_the_same_thing() {
        assert_eq!(
            resolved(r#"#[hl7(crate = "::vendor::hl7_2")]"#),
            "::vendor::hl7_2"
        );
    }

    #[test]
    fn a_struct_attribute_that_is_not_crate_says_so() {
        let input: DeriveInput =
            syn::parse_str("#[hl7(\"PID-5\")] struct S { f: u32 }").expect("test input parses");
        let Err(error) = crate_path(&input) else {
            panic!("a path literal is not a struct option");
        };
        assert!(error.to_string().contains("belong on fields"), "{error}");
    }
}
