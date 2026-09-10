//! [`Strict`]: an opt-in wrapper that rejects unknown fields on
//! deserialize.

use std::ops::{Deref, DerefMut};

use serde::{Serialize, Serializer};

use crate::{Diagnostic, Message};

/// Deserialize `T` strictly: an unrecognized field is a `serde::de::Error`
/// instead of being ignored.
///
/// `T::deserialize` alone stays tolerant of unknown fields, unconditionally
/// (rule S8) — that default does not change. `Strict<T>` is a separate,
/// additive entry point for a caller who wants the opposite for one
/// particular call, such as validating a hand-written JSON fixture for
/// typos. See the spec's strict-mode section (rule S13) for the full
/// rationale, including why this is a distinct type rather than a flag on
/// the existing one.
///
/// A `Deserialize` implementation exists for `Strict<Message>` and
/// `Strict<Diagnostic>` — the two object-shaped types this crate can
/// deserialize. [`crate::Version`], [`crate::NodeKind`],
/// [`crate::Severity`], and [`crate::DiagnosticKind`] are bare strings that
/// already reject an unrecognized value either way, and [`crate::Node`]
/// has no `Deserialize` at all (rule S14), so none of them has a `Strict`
/// form.
///
/// Like every wrapper type in this crate (rule S11), `Strict<T>` implements
/// `Deref`, `DerefMut`, and `From` both ways, plus a `Serialize` impl that
/// delegates to `T`'s own — strictness is a deserialize-only concept, but a
/// `Strict<T>` should still be usable anywhere a `T` is.
///
/// Example:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use serde_hl7_v2::{Message, Strict};
///
/// let good = r#"{"version":"2.5","er7":"MSH|^~\\&|LAB"}"#;
/// let typo = r#"{"verison":"2.5","er7":"MSH|^~\\&|LAB"}"#;
///
/// // The plain type ignores the mistyped optional key — and silently
/// // falls back to MSH-12 for the version. `Strict<Message>` reports it:
/// assert!(serde_json::from_str::<Message>(typo).is_ok());
/// assert!(serde_json::from_str::<Strict<Message>>(typo).is_err());
/// assert!(serde_json::from_str::<Strict<Message>>(good).is_ok());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Strict<T>(pub T);

impl<T> From<T> for Strict<T> {
    fn from(inner: T) -> Strict<T> {
        Strict(inner)
    }
}

impl<T> Deref for Strict<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Strict<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Serialize> Serialize for Strict<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

// The reverse `From<Strict<T>> for T` cannot be written generically over
// `T` — Rust's orphan rule rejects `impl<T> From<Strict<T>> for T` because
// the `Self` type, `T`, would be a completely uncovered impl parameter.
// Each type `Strict` supports gets its own concrete impl instead.
impl From<Strict<Message>> for Message {
    fn from(outer: Strict<Message>) -> Message {
        outer.0
    }
}

impl From<Strict<Diagnostic>> for Diagnostic {
    fn from(outer: Strict<Diagnostic>) -> Diagnostic {
        outer.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deref_reaches_the_inner_value() {
        let strict = Strict(Message::parse("MSH|^~\\&|LAB").unwrap());
        assert_eq!(strict.to_er7(), "MSH|^~\\&|LAB");
    }

    #[test]
    fn from_and_back_round_trip() {
        let message = Message::parse("MSH|^~\\&|LAB").unwrap();
        let strict: Strict<Message> = message.clone().into();
        let back: Message = strict.into();
        assert_eq!(back.to_er7(), message.to_er7());
    }

    #[test]
    fn serialize_delegates_to_the_inner_type() {
        let message = Message::parse("MSH|^~\\&|LAB").unwrap();
        let plain = serde_json::to_string(&message).unwrap();
        let wrapped = serde_json::to_string(&Strict(message)).unwrap();
        assert_eq!(plain, wrapped);
    }
}
