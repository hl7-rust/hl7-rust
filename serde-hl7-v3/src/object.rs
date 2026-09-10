//! The machinery every object-shaped wrapper in this crate is built from:
//! a `macro_rules!` that writes the newtype, its conversions, and its
//! hand-shaped `Serialize`/`Deserialize` impls, plus the seed types that
//! carry strictness down into nested values.
//!
//! This is `macro_rules!`, not a derive: it pulls in no proc-macro crate
//! (rule S1), it produces exactly the visitor [serde's own
//! manual-implementation guide](https://docs.rs/serde/latest/serde/) shows
//! for a struct with named fields, and the generated code is the same for
//! all fourteen object types, which is what makes the strict-mode
//! guarantee (rule S13) hold uniformly rather than per hand-written copy.
//!
//! Each field is declared with a *mode* that fixes its wire behaviour:
//!
//! | mode | Rust field | serializes as | missing on deserialize |
//! |---|---|---|---|
//! | `req T` | `T` (a Serde-native type) | `T` | `missing_field` error (S9) |
//! | `opt T` | `Option<T>` | `T` or `null` | `None` |
//! | `dflt T` | `T: Default` | `T` | `T::default()` |
//! | `reqw W` | the `hl7-3` type wrapper `W` wraps | `W`'s shape | `missing_field` error |
//! | `optw W` | `Option<…>` | `W`'s shape or `null` | `None` |
//! | `vecw W` | `Vec<…>` | array of `W`'s shape | `[]` |

use std::fmt;
use std::marker::PhantomData;

use serde::de::{self, DeserializeSeed, Deserializer, SeqAccess, Visitor};

/// A type that can be deserialized either tolerantly (unknown keys ignored,
/// rule S8) or strictly (unknown keys rejected, rule S13), chosen at the
/// call site. Every object wrapper implements it, which is how
/// [`Seed`] carries the caller's choice into a nested value.
pub(crate) trait StrictDeserialize: Sized {
    fn deserialize_with<'de, D>(deserializer: D, strict: bool) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

/// Deserializes one `T`, strictly or not.
pub(crate) struct Seed<T> {
    strict: bool,
    marker: PhantomData<T>,
}

impl<T> Seed<T> {
    pub(crate) fn new(strict: bool) -> Seed<T> {
        Seed {
            strict,
            marker: PhantomData,
        }
    }
}

impl<'de, T: StrictDeserialize> DeserializeSeed<'de> for Seed<T> {
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize_with(deserializer, self.strict)
    }
}

/// Deserializes an `Option<T>` — `null` or absent is `None` — carrying the
/// strict flag into the `Some` case.
pub(crate) struct OptionSeed<T>(Seed<T>);

impl<T> OptionSeed<T> {
    pub(crate) fn new(strict: bool) -> OptionSeed<T> {
        OptionSeed(Seed::new(strict))
    }
}

impl<'de, T: StrictDeserialize> DeserializeSeed<'de> for OptionSeed<T> {
    type Value = Option<T>;

    fn deserialize<D>(self, deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(self)
    }
}

impl<'de, T: StrictDeserialize> Visitor<'de> for OptionSeed<T> {
    type Value = Option<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an object or null")
    }

    fn visit_none<E: de::Error>(self) -> Result<Option<T>, E> {
        Ok(None)
    }

    fn visit_unit<E: de::Error>(self) -> Result<Option<T>, E> {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.0.deserialize(deserializer).map(Some)
    }
}

/// Deserializes a `Vec<T>`, carrying the strict flag into every element.
pub(crate) struct VecSeed<T> {
    strict: bool,
    marker: PhantomData<T>,
}

impl<T> VecSeed<T> {
    pub(crate) fn new(strict: bool) -> VecSeed<T> {
        VecSeed {
            strict,
            marker: PhantomData,
        }
    }
}

impl<'de, T: StrictDeserialize> DeserializeSeed<'de> for VecSeed<T> {
    type Value = Vec<T>;

    fn deserialize<D>(self, deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, T: StrictDeserialize> Visitor<'de> for VecSeed<T> {
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an array of objects")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Vec<T>, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element_seed(Seed::<T>::new(self.strict))? {
            items.push(item);
        }
        Ok(items)
    }
}

/// The one visitor type every object wrapper's `Deserialize` runs through;
/// the macro below writes a `Visitor` impl for `ObjectVisitor<W>` per
/// wrapper `W`.
pub(crate) struct ObjectVisitor<T> {
    pub(crate) strict: bool,
    marker: PhantomData<T>,
}

impl<T> ObjectVisitor<T> {
    pub(crate) fn new(strict: bool) -> ObjectVisitor<T> {
        ObjectVisitor {
            strict,
            marker: PhantomData,
        }
    }
}

/// One field's contribution to `serialize`, its slot type while
/// deserializing, how it is read, and how it is finished — dispatched on
/// the mode word. See the module documentation for the table.
macro_rules! field {
    (@ser $state:ident, $key:literal, req $ty:ty, $val:expr) => {
        $state.serialize_field($key, &$val)?
    };
    (@ser $state:ident, $key:literal, opt $ty:ty, $val:expr) => {
        $state.serialize_field($key, &$val)?
    };
    (@ser $state:ident, $key:literal, dflt $ty:ty, $val:expr) => {
        $state.serialize_field($key, &$val)?
    };
    (@ser $state:ident, $key:literal, reqw $ty:ty, $val:expr) => {
        $state.serialize_field($key, &<$ty>::from($val.clone()))?
    };
    (@ser $state:ident, $key:literal, optw $ty:ty, $val:expr) => {
        $state.serialize_field($key, &$val.clone().map(|v| <$ty>::from(v)))?
    };
    (@ser $state:ident, $key:literal, vecw $ty:ty, $val:expr) => {
        $state.serialize_field(
            $key,
            &$val.iter().cloned().map(|v| <$ty>::from(v)).collect::<Vec<$ty>>(),
        )?
    };

    (@slot req $ty:ty) => { $ty };
    (@slot opt $ty:ty) => { Option<$ty> };
    (@slot dflt $ty:ty) => { $ty };
    (@slot reqw $ty:ty) => { $ty };
    (@slot optw $ty:ty) => { Option<$ty> };
    (@slot vecw $ty:ty) => { Vec<$ty> };

    (@read $map:ident, $strict:expr, req $ty:ty) => {
        $map.next_value::<$ty>()?
    };
    (@read $map:ident, $strict:expr, opt $ty:ty) => {
        $map.next_value::<Option<$ty>>()?
    };
    (@read $map:ident, $strict:expr, dflt $ty:ty) => {
        $map.next_value::<$ty>()?
    };
    (@read $map:ident, $strict:expr, reqw $ty:ty) => {
        $map.next_value_seed($crate::object::Seed::<$ty>::new($strict))?
    };
    (@read $map:ident, $strict:expr, optw $ty:ty) => {
        $map.next_value_seed($crate::object::OptionSeed::<$ty>::new($strict))?
    };
    (@read $map:ident, $strict:expr, vecw $ty:ty) => {
        $map.next_value_seed($crate::object::VecSeed::<$ty>::new($strict))?
    };

    (@finish $slot:ident, $key:literal, req $ty:ty) => {
        $slot.ok_or_else(|| serde::de::Error::missing_field($key))?
    };
    (@finish $slot:ident, $key:literal, opt $ty:ty) => {
        $slot.flatten()
    };
    (@finish $slot:ident, $key:literal, dflt $ty:ty) => {
        $slot.unwrap_or_default()
    };
    (@finish $slot:ident, $key:literal, reqw $ty:ty) => {
        $slot.ok_or_else(|| serde::de::Error::missing_field($key))?.0
    };
    (@finish $slot:ident, $key:literal, optw $ty:ty) => {
        $slot.flatten().map(|w| w.0)
    };
    (@finish $slot:ident, $key:literal, vecw $ty:ty) => {
        $slot.unwrap_or_default().into_iter().map(|w| w.0).collect()
    };
}

/// Writes one object wrapper: the newtype, `From` both ways, `Deref`/
/// `DerefMut`, `Serialize` as an object with the listed keys in order, a
/// tolerant `Deserialize`, a strict `Deserialize` for `Strict<W>`, and the
/// [`StrictDeserialize`] impl that lets a parent carry strictness down.
macro_rules! object_wrapper {
    (
        $(#[$meta:meta])*
        $name:ident wraps $inner:path,
        expecting $expecting:literal,
        derive [$($derive:ident),* $(,)?],
        fields {
            $( $field:ident : $mode:ident $ty:ty => $key:literal ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, $($derive),*)]
        pub struct $name(pub $inner);

        impl From<$inner> for $name {
            fn from(inner: $inner) -> $name {
                $name(inner)
            }
        }

        impl From<$name> for $inner {
            fn from(outer: $name) -> $inner {
                outer.0
            }
        }

        impl From<$crate::Strict<$name>> for $name {
            fn from(outer: $crate::Strict<$name>) -> $name {
                outer.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &$inner {
                &self.0
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut $inner {
                &mut self.0
            }
        }

        const _: () = {
            const FIELDS: &[&str] = &[$($key),+];

            impl serde::Serialize for $name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    use serde::ser::SerializeStruct;
                    let mut state = serializer.serialize_struct(stringify!($name), FIELDS.len())?;
                    $( field!(@ser state, $key, $mode $ty, self.0.$field); )+
                    state.end()
                }
            }

            impl<'de> serde::de::Visitor<'de> for $crate::object::ObjectVisitor<$name> {
                type Value = $name;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str($expecting)
                }

                fn visit_map<V>(self, mut map: V) -> Result<$name, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
                {
                    $( let mut $field: Option<field!(@slot $mode $ty)> = None; )+

                    while let Some(key) = map.next_key::<String>()? {
                        match key.as_str() {
                            $(
                                $key => {
                                    if $field.is_some() {
                                        return Err(serde::de::Error::duplicate_field($key));
                                    }
                                    $field = Some(field!(@read map, self.strict, $mode $ty));
                                }
                            )+
                            _ if self.strict => {
                                return Err(serde::de::Error::unknown_field(&key, FIELDS));
                            }
                            _ => {
                                let _ = map.next_value::<serde::de::IgnoredAny>()?;
                            }
                        }
                    }

                    // A `path` fragment cannot open a struct literal, so
                    // the value is built from its `Default` and assigned
                    // field by field; every wrapped type derives `Default`.
                    let mut inner = <$inner>::default();
                    $( inner.$field = field!(@finish $field, $key, $mode $ty); )+
                    Ok($name(inner))
                }
            }

            impl $crate::object::StrictDeserialize for $name {
                fn deserialize_with<'de, D>(deserializer: D, strict: bool) -> Result<$name, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    deserializer.deserialize_struct(
                        stringify!($name),
                        FIELDS,
                        $crate::object::ObjectVisitor::<$name>::new(strict),
                    )
                }
            }

            impl<'de> serde::Deserialize<'de> for $name {
                fn deserialize<D>(deserializer: D) -> Result<$name, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    <$name as $crate::object::StrictDeserialize>::deserialize_with(deserializer, false)
                }
            }

            impl<'de> serde::Deserialize<'de> for $crate::Strict<$name> {
                /// See [`Strict`](crate::Strict) (rule S13): an unrecognized
                /// key, here or in any nested object, is a
                /// `serde::de::Error::unknown_field` rather than being ignored.
                fn deserialize<D>(deserializer: D) -> Result<$crate::Strict<$name>, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    <$name as $crate::object::StrictDeserialize>::deserialize_with(deserializer, true)
                        .map($crate::Strict)
                }
            }
        };
    };
}
