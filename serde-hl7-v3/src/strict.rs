//! [`Strict`]: an opt-in wrapper that rejects unknown fields on
//! deserialize.

use std::ops::{Deref, DerefMut};

use serde::{Serialize, Serializer};

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
/// A `Deserialize` implementation exists for `Strict<T>` for every
/// object-shaped type in this crate — [`crate::Message`],
/// [`crate::ControlAct`], [`crate::Element`], the five data types, and the
/// six RIM classes. [`crate::NullFlavor`] is a bare string that accepts
/// any code by design, so it has no `Strict` form.
///
/// **Strictness nests.** `Strict<Message>` rejects an unrecognized key not
/// only on the message object itself but inside its identifiers, its
/// control act, that act's trigger-event code, and every element of the
/// domain payload's tree — a typo four levels down is caught the same as
/// one at the top.
///
/// As rule S11 asks of every wrapper type in this crate, `Strict<T>`
/// implements `Deref`, `DerefMut`, and `From` both ways, plus a
/// `Serialize` impl that delegates to `T`'s own — strictness is a
/// deserialize-only concept, but a `Strict<T>` should still be usable
/// anywhere a `T` is.
///
/// Example:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use serde_hl7_v3::{Ii, Strict};
///
/// let good = r#"{"root":"1.2.3","extension":"7"}"#;
/// let typo = r#"{"root":"1.2.3","extention":"7"}"#;
///
/// // The plain type ignores the mistyped optional key — and silently
/// // reads `extension` as absent. `Strict<Ii>` reports it:
/// assert_eq!(serde_json::from_str::<Ii>(typo)?.extension, None);
/// assert!(serde_json::from_str::<Strict<Ii>>(typo).is_err());
/// assert!(serde_json::from_str::<Strict<Ii>>(good).is_ok());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
// Each object wrapper gets its own concrete impl instead, written by the
// `object_wrapper!` macro alongside the type.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ii;

    fn ii() -> Ii {
        Ii(hl7_3::Ii {
            root: "1".into(),
            extension: None,
        })
    }

    #[test]
    fn deref_reaches_the_inner_value() {
        let strict = Strict(ii());
        assert_eq!(strict.root, "1");
    }

    #[test]
    fn from_and_back_round_trip() {
        let strict: Strict<Ii> = ii().into();
        let back: Ii = strict.into();
        assert_eq!(back, ii());
    }

    #[test]
    fn serialize_delegates_to_the_inner_type() {
        let plain = serde_json::to_string(&ii()).unwrap();
        let wrapped = serde_json::to_string(&Strict(ii())).unwrap();
        assert_eq!(plain, wrapped);
    }
}
