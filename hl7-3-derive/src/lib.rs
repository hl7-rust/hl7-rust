//! Derive macro for [`hl7-3`](https://crates.io/crates/hl7-3): map a
//! struct's fields to XML element attributes and children once, in the
//! type definition, instead of writing the same accessor calls at every
//! call site.
//!
//! This crate is not used directly. `hl7-3` re-exports the macro behind
//! its `derive` feature, so the dependency to add is:
//!
//! ```toml
//! hl7-3 = { version = "0.1", features = ["derive"] }
//! ```
//!
//! Keeping the macro in a crate of their own is what lets the default
//! build of `hl7-3` keep exactly one dependency: `syn` and `quote` are
//! compiled only for callers who ask for the macro.
//!
//! ```ignore
//! use hl7_3::FromElement;
//! use hl7_3::rim::Act;
//!
//! #[derive(FromElement, Default)]
//! struct Observation {
//!     #[element("classCode")]     class_code: String,
//!     #[element("moodCode")]      mood_code: String,
//!     #[element(child = "note")]  note: Option<String>,
//!     #[element(nested = "component")] component: Act, // its own FromElement
//!     #[element(raw)]             raw: hl7_3::xml::Element, // the escape hatch
//! }
//! ```
//!
//! One attribute per field, and a field with none is `Default::default()`:
//!
//! | attribute | reads |
//! |---|---|
//! | `#[element("classCode")]` | the `classCode` attribute, via `FromElementValue::from_attribute` |
//! | `#[element(child = "note")]` | the `note` child's text, via `FromElementValue::from_child_text` |
//! | `#[element(nested = "component")]` | the `component` child, via the field type's own `FromElement` |
//! | `#[element(raw)]` | the whole element (field type must be `hl7_3::xml::Element`) |
//! | none | `Default::default()` |
//!
//! There is no `#[derive(ToElement)]`: `hl7-3` has no XML-writing
//! capability yet (see its `spec/index.md` §1), so a write-direction macro
//! would have nothing real to generate.
//!
//! ## When `hl7-3` is not called `hl7_3`
//!
//! The generated code names the crate absolutely, as `::hl7_3`, so that it
//! works wherever the type is defined without the caller importing
//! anything. A caller who renames the dependency —
//! `hl7 = { package = "hl7-3" }` in `Cargo.toml`, or a workspace that
//! aliases it — has no `::hl7_3` for the macro to reach, and the generated
//! code stops compiling. Say where it is instead, once, on the struct:
//!
//! ```ignore
//! #[derive(FromElement)]
//! #[element(crate = hl7)]        // or `crate = "::some::path::to::hl7_3"`
//! struct Author {
//!     #[element("classCode")] class_code: String,
//! }
//! ```

#![warn(missing_docs, clippy::pedantic)]

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Field, Fields, LitStr, Path, Token, parse_macro_input};

/// Derive `FromElement`: read each annotated field from an XML element's
/// attributes or children.
///
/// See the crate documentation for the attributes. Only structs with named
/// fields are supported; a tuple struct has no field names to map, and an
/// enum has no single element shape to read.
#[proc_macro_derive(FromElement, attributes(element))]
pub fn derive_from_element(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match from_element(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// What one field's `#[element(...)]` attribute asked for.
enum Mapping {
    /// `#[element("classCode")]`
    Attribute(LitStr),
    /// `#[element(child = "note")]`
    ChildText(LitStr),
    /// `#[element(nested = "component")]`
    Nested(LitStr),
    /// `#[element(raw)]`
    Raw,
    /// No attribute at all.
    None,
}

fn from_element(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let krate = crate_path(input)?;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let mut reads = Vec::new();
    for field in named_fields(input)? {
        let ident = field.ident.as_ref().expect("named fields");
        let ty = &field.ty;
        reads.push(match mapping(field)? {
            Mapping::Attribute(name) => quote! {
                #ident: <#ty as #krate::typed::FromElementValue>::from_attribute(
                    element.attribute(#name)
                )
            },
            Mapping::ChildText(name) => quote! {
                #ident: <#ty as #krate::typed::FromElementValue>::from_child_text(
                    element.child(#name).and_then(#krate::xml::Element::text_opt)
                )
            },
            Mapping::Nested(name) => quote! {
                #ident: element
                    .child(#name)
                    .map(<#ty as #krate::typed::FromElement>::from_element)
                    .unwrap_or_default()
            },
            Mapping::Raw => quote! {
                #ident: ::core::clone::Clone::clone(element)
            },
            Mapping::None => quote! {
                #ident: ::core::default::Default::default()
            },
        });
    }
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #krate::typed::FromElement for #name #type_generics #where_clause {
            fn from_element(element: &#krate::xml::Element) -> Self {
                #name { #(#reads),* }
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
                "hl7-3 derives map field names to element attributes and children, so the struct needs named fields",
            )),
        },
        Data::Enum(_) | Data::Union(_) => Err(syn::Error::new(
            input.ident.span(),
            "hl7-3 derives apply to structs; an enum or union has no single element shape",
        )),
    }
}

/// Read one field's `#[element(...)]` attribute.
/// Where the generated code should look for `hl7-3`.
///
/// `::hl7_3` unless the struct says otherwise with
/// `#[element(crate = ...)]`, which is what a caller who renamed the
/// dependency needs: the generated code names the crate absolutely so that
/// it compiles wherever the type is defined, and an absolute name that does
/// not exist is a compile error the caller cannot work around from their
/// side.
///
/// The value is a path, written bare (`crate = hl7`) or quoted
/// (`crate = "::vendor::hl7_3"`); the quoted form is there because that is
/// how the rest of the ecosystem spells it.
fn crate_path(input: &DeriveInput) -> syn::Result<Path> {
    for attribute in &input.attrs {
        if !attribute.path().is_ident("element") {
            continue;
        }
        return attribute.parse_args_with(|stream: syn::parse::ParseStream| {
            stream.parse::<Token![crate]>().map_err(|_| {
                syn::Error::new(
                    attribute.span(),
                    "the only #[element(...)] option on a struct is `crate = ...`; \
                     name attributes belong on fields",
                )
            })?;
            stream.parse::<Token![=]>()?;
            if stream.peek(LitStr) {
                return stream.parse::<LitStr>()?.parse();
            }
            stream.parse()
        });
    }
    Ok(syn::parse_quote!(::hl7_3))
}

fn mapping(field: &Field) -> syn::Result<Mapping> {
    let mut found = Mapping::None;
    for attribute in &field.attrs {
        if !attribute.path().is_ident("element") {
            continue;
        }
        if !matches!(found, Mapping::None) {
            return Err(syn::Error::new(
                attribute.span(),
                "a field takes one #[element(...)] attribute",
            ));
        }
        // Four spellings: an attribute-name literal, `child = "..."`,
        // `nested = "..."`, or `raw`.
        found = attribute.parse_args_with(|input: syn::parse::ParseStream| {
            if input.peek(LitStr) {
                return Ok(Mapping::Attribute(input.parse()?));
            }
            let word: syn::Ident = input.parse()?;
            match word.to_string().as_str() {
                "raw" => Ok(Mapping::Raw),
                "child" => {
                    input.parse::<syn::Token![=]>()?;
                    Ok(Mapping::ChildText(input.parse()?))
                }
                "nested" => {
                    input.parse::<syn::Token![=]>()?;
                    Ok(Mapping::Nested(input.parse()?))
                }
                other => Err(syn::Error::new(
                    word.span(),
                    format!(
                        "unknown #[element(...)] option {other:?}; expected an attribute \
                         name such as #[element(\"classCode\")], `child = \"name\"`, \
                         `nested = \"name\"`, or `raw`"
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
        assert_eq!(resolved(""), "::hl7_3");
    }

    #[test]
    fn a_bare_path_is_taken_as_written() {
        assert_eq!(resolved("#[element(crate = hl7)]"), "hl7");
        assert_eq!(
            resolved("#[element(crate = ::vendor::hl7_3)]"),
            "::vendor::hl7_3"
        );
    }

    #[test]
    fn a_quoted_path_is_the_same_thing() {
        assert_eq!(
            resolved(r#"#[element(crate = "::vendor::hl7_3")]"#),
            "::vendor::hl7_3"
        );
    }

    #[test]
    fn a_struct_attribute_that_is_not_crate_says_so() {
        let input: DeriveInput = syn::parse_str("#[element(\"classCode\")] struct S { f: u32 }")
            .expect("test input parses");
        let Err(error) = crate_path(&input) else {
            panic!("a name literal is not a struct option");
        };
        assert!(error.to_string().contains("belong on fields"), "{error}");
    }
}
