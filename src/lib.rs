//! Derive macros for [`hl7-rust`](https://crates.io/crates/hl7-rust): map a
//! struct's fields to HL7 v2 message paths once, in the type definition,
//! instead of writing the same accessor calls at every call site.
//!
//! This crate is not used directly. `hl7-rust` re-exports both macros behind
//! its `derive` feature, so the dependency to add is:
//!
//! ```toml
//! hl7-rust = { version = "0.1", features = ["derive"] }
//! ```
//!
//! Keeping the macros in a crate of their own is what lets the default
//! build of `hl7-rust` keep exactly one dependency: `syn` and `quote` are
//! compiled only for callers who ask for the macros.
//!
//! ```ignore
//! use hl7::v2::{FromHl7, ToHl7, Raw};
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

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Field, Fields, LitStr, parse_macro_input};

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
/// `hl7::v2::Builder` (whose `encode` method takes a `ToHl7`) or add them
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
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let mut reads = Vec::new();
    for field in named_fields(input)? {
        let ident = field.ident.as_ref().expect("named fields");
        let ty = &field.ty;
        reads.push(match mapping(field)? {
            Mapping::Path(path) => quote! {
                #ident: <#ty as ::hl7::v2::FromHl7Value>::from_hl7_value(message, #path)?
            },
            Mapping::Nested => quote! {
                #ident: <#ty as ::hl7::v2::FromHl7>::from_hl7(message)?
            },
            Mapping::Raw => quote! {
                #ident: ::hl7::v2::Raw::new(::core::clone::Clone::clone(message)).into()
            },
            Mapping::None => quote! {
                #ident: ::core::default::Default::default()
            },
        });
    }
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::hl7::v2::FromHl7 for #name #type_generics #where_clause {
            fn from_hl7(message: &::hl7::v2::Message) -> ::core::result::Result<Self, ::hl7::v2::Error> {
                ::core::result::Result::Ok(#name { #(#reads),* })
            }
        }
    })
}

fn to_hl7(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let mut writes = Vec::new();
    for field in named_fields(input)? {
        let ident = field.ident.as_ref().expect("named fields");
        let ty = &field.ty;
        match mapping(field)? {
            Mapping::Path(path) => writes.push(quote! {
                <#ty as ::hl7::v2::ToHl7Value>::to_hl7_value(&self.#ident, message, #path)?;
            }),
            Mapping::Nested => writes.push(quote! {
                <#ty as ::hl7::v2::ToHl7>::to_hl7(&self.#ident, message)?;
            }),
            // The raw message is where the struct came from, not something
            // to write back over it.
            Mapping::Raw | Mapping::None => {}
        }
    }
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::hl7::v2::ToHl7 for #name #type_generics #where_clause {
            fn to_hl7(
                &self,
                message: &mut ::hl7::v2::Message,
            ) -> ::core::result::Result<(), ::hl7::v2::Error> {
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
                "hl7-v2 derives map field names to HL7 paths, so the struct needs named fields",
            )),
        },
        Data::Enum(_) | Data::Union(_) => Err(syn::Error::new(
            input.ident.span(),
            "hl7-v2 derives apply to structs; an enum or union has no single message shape",
        )),
    }
}

/// Read one field's `#[hl7(...)]` attribute.
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
