//! Shared schema primitives.
//!
//! Ports the observable behaviour of `packages/schema/src/schema.ts`: optional
//! keys omit `undefined` while encoding, and `FiniteFromString` decodes a JSON
//! string into a finite number and encodes it back to a string.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A finite number that travels on the wire as a string.
///
/// Decoding accepts either a JSON string (`"1"`) or a number (`1`); encoding
/// always emits the canonical decimal string form (`"1"`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FiniteFromString(pub f64);

impl Serialize for FiniteFromString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let text = if self.0.fract() == 0.0 && self.0.is_finite() {
            format!("{}", self.0 as i64)
        } else {
            format!("{}", self.0)
        };
        serializer.serialize_str(&text)
    }
}

impl<'de> Deserialize<'de> for FiniteFromString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrNumber {
            String(String),
            Number(f64),
        }

        match StringOrNumber::deserialize(deserializer)? {
            StringOrNumber::Number(value) => Ok(FiniteFromString(value)),
            StringOrNumber::String(value) => value
                .parse::<f64>()
                .map(FiniteFromString)
                .map_err(serde::de::Error::custom),
        }
    }
}
