// FIXME: Remove once passing by value will be supported in serde.
#![allow(clippy::trivially_copy_pass_by_ref)]

use alloc::borrow::Cow;
use alloc::string::String;

use hashbrown::DefaultHashBuilder;

use indexmap::map::{IndexMap, Iter};

use serde::{Deserialize, Serialize};

use crate::macros::map;

fn is_u8_max(value: &u8) -> bool {
    *value == u8::MAX
}

fn is_u8_min(value: &u8) -> bool {
    *value == u8::MIN
}

fn u8_max() -> u8 {
    u8::MAX
}

fn is_u16_max(value: &u16) -> bool {
    *value == u16::MAX
}

fn is_u16_min(value: &u16) -> bool {
    *value == u16::MIN
}

fn u16_max() -> u16 {
    u16::MAX
}

fn is_u32_max(value: &u32) -> bool {
    *value == u32::MAX
}

fn is_u32_min(value: &u32) -> bool {
    *value == u32::MIN
}

fn u32_max() -> u32 {
    u32::MAX
}

fn is_u64_max(value: &u64) -> bool {
    *value == u64::MAX
}

fn is_u64_min(value: &u64) -> bool {
    *value == u64::MIN
}

fn u64_max() -> u64 {
    u64::MAX
}

fn is_f32_max(value: &f32) -> bool {
    *value == f32::MAX
}

fn is_f32_min(value: &f32) -> bool {
    *value == f32::MIN
}

fn f32_min() -> f32 {
    f32::MIN
}

fn f32_max() -> f32 {
    f32::MAX
}

fn is_f64_max(value: &f64) -> bool {
    *value == f64::MAX
}

fn is_f64_min(value: &f64) -> bool {
    *value == f64::MIN
}

fn f64_min() -> f64 {
    f64::MIN
}

fn f64_max() -> f64 {
    f64::MAX
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// All route input parameters identifiers.
pub enum ParameterId {
    /// A [`bool`] value.
    Bool,
    /// An [`u8`] value.
    U8,
    /// An [`u16`] value.
    U16,
    /// An [`u32`] value.
    U32,
    /// An [`u64`] value.
    U64,
    /// A range of [`u64`].
    RangeU64,
    /// A [`f32`] value.
    F32,
    /// A [`f64`] value.
    F64,
    /// A range of [`f64`].
    RangeF64,
    /// A characters sequence.
    CharsSequence,
}

impl ParameterId {
    /// Converts a [`ParameterKind`] into a [`ParameterId`].
    #[must_use]
    pub const fn from_parameter_kind(parameter_kind: &ParameterKind) -> Self {
        match parameter_kind {
            ParameterKind::Bool { .. } => Self::Bool,
            ParameterKind::U8 { .. } => Self::U8,
            ParameterKind::U16 { .. } => Self::U16,
            ParameterKind::U32 { .. } => Self::U32,
            ParameterKind::U64 { .. } => Self::U64,
            ParameterKind::RangeU64 { .. } => Self::RangeU64,
            ParameterKind::F32 { .. } => Self::F32,
            ParameterKind::F64 { .. } => Self::F64,
            ParameterKind::RangeF64 { .. } => Self::RangeF64,
            ParameterKind::CharsSequence { .. } => Self::CharsSequence,
        }
    }

    /// Converts a [`ParameterValue`] into a [`ParameterId`].
    #[must_use]
    pub const fn from_parameter_value(parameter_value: &ParameterValue) -> Self {
        match parameter_value {
            ParameterValue::Bool(_) => Self::Bool,
            ParameterValue::U8(_) => Self::U8,
            ParameterValue::U16(_) => Self::U16,
            ParameterValue::U32(_) => Self::U32,
            ParameterValue::U64(_) => Self::U64,
            ParameterValue::F32(_) => Self::F32,
            ParameterValue::F64(_) => Self::F64,
            ParameterValue::CharsSequence(_) => Self::CharsSequence,
        }
    }

    /// Shows a [`ParameterId`] as a [`&str`].
    #[must_use]
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Bool => "Bool",
            Self::U8 => "U8",
            Self::U16 => "U16",
            Self::U32 => "U32",
            Self::U64 => "U64",
            Self::RangeU64 => "RangeU64",
            Self::F32 => "F32",
            Self::F64 => "F64",
            Self::RangeF64 => "RangeF64",
            Self::CharsSequence => "String",
        }
    }

    /// Prints the type associated with a [`ParameterId`] formatted
    /// as a [`&str`].
    #[must_use]
    pub const fn as_type(&self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 | Self::RangeU64 => "u64",
            Self::F32 => "f32",
            Self::F64 | Self::RangeF64 => "f64",
            Self::CharsSequence => "String",
        }
    }
}

/// All supported kinds of route input parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterKind {
    /// A [`bool`] value.
    Bool {
        /// The initial [`bool`] value, but also the default one
        /// in case of missing input parameter.
        default: bool,
    },
    /// An [`u8`] value.
    U8 {
        /// The initial [`u8`] value, but also the default one
        /// in case of a missing input parameter.
        default: u8,
        /// The minimum [`u8`] value allowed.
        #[serde(skip_serializing_if = "is_u8_max")]
        #[serde(default)]
        min: u8,
        /// The maximum [`u8`] value allowed.
        #[serde(skip_serializing_if = "is_u8_min")]
        #[serde(default = "u8_max")]
        max: u8,
    },
    /// An [`u16`] value.
    U16 {
        /// The initial [`u16`] value, but also the default one
        /// in case of a missing input parameter.
        default: u16,
        /// The minimum [`u16`] value allowed.
        #[serde(skip_serializing_if = "is_u16_max")]
        #[serde(default)]
        min: u16,
        /// The maximum [`u16`] value allowed.
        #[serde(skip_serializing_if = "is_u16_min")]
        #[serde(default = "u16_max")]
        max: u16,
    },
    /// An [`u32`] value.
    U32 {
        /// The initial [`u32`] value, but also the default one
        /// in case of a missing input parameter.
        default: u32,
        /// The minimum [`u32`] value allowed.
        #[serde(skip_serializing_if = "is_u32_max")]
        #[serde(default)]
        min: u32,
        /// The maximum [`u32`] allowed value.
        #[serde(skip_serializing_if = "is_u32_min")]
        #[serde(default = "u32_max")]
        max: u32,
    },
    /// An [`u64`] value.
    U64 {
        /// The initial [`u64`] value, but also the default one
        /// in case of a missing input parameter.
        default: u64,
        /// The minimum [`u64`] value allowed.
        #[serde(skip_serializing_if = "is_u64_max")]
        #[serde(default)]
        min: u64,
        /// The maximum [`u64`] allowed value.
        #[serde(skip_serializing_if = "is_u64_min")]
        #[serde(default = "u64_max")]
        max: u64,
    },
    /// A [`f32`] value.
    F32 {
        /// The initial [`f32`] value, but also the default one
        /// in case of a missing input parameter.
        default: f32,
        /// The minimum [`f32`] value allowed.
        #[serde(skip_serializing_if = "is_f32_max")]
        #[serde(default = "f32_min")]
        min: f32,
        /// The maximum [`f32`] allowed value.
        #[serde(skip_serializing_if = "is_f32_min")]
        #[serde(default = "f32_max")]
        max: f32,
        /// The decimal step associated with the [`f32`] value.
        #[serde(skip_serializing_if = "is_f32_min")]
        #[serde(default)]
        step: f32,
    },
    /// A [`f64`] value.
    F64 {
        /// The initial [`f64`] value, but also the default one
        /// in case of a missing input.
        default: f64,
        /// The minimum [`f64`] value allowed.
        #[serde(skip_serializing_if = "is_f64_max")]
        #[serde(default = "f64_min")]
        min: f64,
        /// The maximum [`f64`] allowed value.
        #[serde(skip_serializing_if = "is_f64_min")]
        #[serde(default = "f64_max")]
        max: f64,
        /// The decimal step associated with the [`f64`] value.
        #[serde(skip_serializing_if = "is_f64_min")]
        #[serde(default)]
        step: f64,
    },
    /// A range of [`u64`] values.
    RangeU64 {
        /// Minimum [`u64`] value allowed.
        min: u64,
        /// Maximum [`u64`] value allowed.
        max: u64,
        /// The [`u64`] step necessary to pass from one allowed value
        /// to another one in the range.
        step: u64,
        /// Initial [`u64`] range value.
        default: u64,
    },
    /// A range of [`f64`] values.
    RangeF64 {
        /// The minimum [`f64`] value allowed.
        min: f64,
        /// Maximum [`u64`] value allowed.
        max: f64,
        /// The [`f64`] step necessary to pass from one allowed value
        /// to another one in the range. It is always a positive value.
        step: f64,
        /// Initial [`f64`] range value.
        default: f64,
    },
    /// A characters sequence.
    ///
    /// This kind of input parameter can contain an unknown or a precise
    /// sequence of characters.
    CharsSequence {
        /// Initial characters sequence, which also represents the default
        /// value.
        default: Cow<'static, str>,
        /// The character sequence length.
        length: usize,
    },
}

/// Floating point decimal precision.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DecimalPrecision {
    /// One digit.
    OneDigit,
    /// Two digits.
    TwoDigits,
    /// Three digits.
    ThreeDigits,
    /// Any digits.
    Any,
}

impl DecimalPrecision {
    const fn to_f32(self) -> f32 {
        match self {
            Self::OneDigit => 0.1,
            Self::TwoDigits => 0.01,
            Self::ThreeDigits => 0.001,
            Self::Any => 0.,
        }
    }

    const fn to_f64(self) -> f64 {
        match self {
            Self::OneDigit => 0.1,
            Self::TwoDigits => 0.01,
            Self::ThreeDigits => 0.001,
            Self::Any => 0.,
        }
    }
}

map! {
  /// A map of serializable and deserializable [`Parameters`] data.
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct ParametersData(IndexMap<String, ParameterKind, DefaultHashBuilder>);
}

impl ParametersData {
    /// Retrieves the value associated with the specified key
    /// from [`ParametersData`].
    #[must_use]
    #[inline]
    pub fn get(&self, key: &str) -> Option<&ParameterKind> {
        self.0.get(key)
    }
}

/// Route input parameters.
#[derive(Debug, Clone)]
pub struct Parameters(IndexMap<&'static str, ParameterKind, DefaultHashBuilder>);

impl Default for Parameters {
    fn default() -> Self {
        Self::new()
    }
}

impl Parameters {
    /// Creates a [`Parameters`].
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self(IndexMap::with_hasher(DefaultHashBuilder::default()))
    }

    /// Adds a [`bool`] parameter.
    #[must_use]
    #[inline]
    pub fn bool(self, name: &'static str, default: bool) -> Self {
        self.create_parameter(name, ParameterKind::Bool { default })
    }

    /// Adds an [`u8`] parameter.
    #[must_use]
    #[inline]
    pub fn u8(self, name: &'static str, default: u8) -> Self {
        self.create_parameter(
            name,
            ParameterKind::U8 {
                default,
                min: u8::MAX,
                max: u8::MIN,
            },
        )
    }

    /// Adds an [`u8`] parameter with limits.
    #[must_use]
    #[inline]
    pub fn u8_with_limits(self, name: &'static str, default: u8, min: u8, max: u8) -> Self {
        self.create_parameter(name, ParameterKind::U8 { default, min, max })
    }

    /// Adds an [`u16`] parameter.
    #[must_use]
    #[inline]
    pub fn u16(self, name: &'static str, default: u16) -> Self {
        self.create_parameter(
            name,
            ParameterKind::U16 {
                default,
                min: u16::MAX,
                max: u16::MIN,
            },
        )
    }

    /// Adds an [`u16`] parameter with limits.
    #[must_use]
    #[inline]
    pub fn u16_with_limits(self, name: &'static str, default: u16, min: u16, max: u16) -> Self {
        self.create_parameter(name, ParameterKind::U16 { default, min, max })
    }

    /// Adds an [`u32`] parameter.
    #[must_use]
    #[inline]
    pub fn u32(self, name: &'static str, default: u32) -> Self {
        self.create_parameter(
            name,
            ParameterKind::U32 {
                default,
                min: u32::MAX,
                max: u32::MIN,
            },
        )
    }

    /// Adds an [`u32`] parameter with limits.
    #[must_use]
    #[inline]
    pub fn u32_with_limits(self, name: &'static str, default: u32, min: u32, max: u32) -> Self {
        self.create_parameter(name, ParameterKind::U32 { default, min, max })
    }

    /// Adds an [`u64`] parameter.
    #[must_use]
    #[inline]
    pub fn u64(self, name: &'static str, default: u64) -> Self {
        self.create_parameter(
            name,
            ParameterKind::U64 {
                default,
                min: u64::MAX,
                max: u64::MIN,
            },
        )
    }

    /// Adds an [`u64`] parameter with limits.
    #[must_use]
    #[inline]
    pub fn u64_with_limits(self, name: &'static str, default: u64, min: u64, max: u64) -> Self {
        self.create_parameter(name, ParameterKind::U64 { default, min, max })
    }

    /// Adds a [`f32`] parameter.
    #[must_use]
    #[inline]
    pub fn f32(self, name: &'static str, default: f32) -> Self {
        self.create_parameter(
            name,
            ParameterKind::F32 {
                default,
                min: f32::MAX,
                max: f32::MIN,
                step: 0.,
            },
        )
    }

    /// Adds a [`f32`] parameter with limits.
    #[must_use]
    #[inline]
    pub fn f32_with_limits(
        self,
        name: &'static str,
        default: f32,
        min: f32,
        max: f32,
        decimal_precision: DecimalPrecision,
    ) -> Self {
        self.create_parameter(
            name,
            ParameterKind::F32 {
                default,
                min,
                max,
                step: decimal_precision.to_f32(),
            },
        )
    }

    /// Adds a [`f64`] parameter.
    #[must_use]
    #[inline]
    pub fn f64(self, name: &'static str, default: f64) -> Self {
        self.create_parameter(
            name,
            ParameterKind::F64 {
                default,
                min: f64::MAX,
                max: f64::MIN,
                step: 0.,
            },
        )
    }

    /// Adds a [`f64`] parameter with limits.
    #[must_use]
    #[inline]
    pub fn f64_with_limits(
        self,
        name: &'static str,
        default: f64,
        min: f64,
        max: f64,
        decimal_precision: DecimalPrecision,
    ) -> Self {
        self.create_parameter(
            name,
            ParameterKind::F64 {
                default,
                min,
                max,
                step: decimal_precision.to_f64(),
            },
        )
    }

    /// Adds an [`u64`] range without a default value.
    #[must_use]
    #[inline]
    pub fn rangeu64(self, name: &'static str, range: (u64, u64, u64)) -> Self {
        self.rangeu64_with_default(name, range, 0)
    }

    /// Adds an [`u64`] range with a default value.
    #[must_use]
    #[inline]
    pub fn rangeu64_with_default(
        self,
        name: &'static str,
        range: (u64, u64, u64),
        default: u64,
    ) -> Self {
        self.create_parameter(
            name,
            ParameterKind::RangeU64 {
                min: range.0,
                max: range.1,
                step: range.2,
                default,
            },
        )
    }

    /// Adds a [`f64`] range without a default value.
    #[must_use]
    #[inline]
    pub fn rangef64(self, name: &'static str, range: (f64, f64, f64)) -> Self {
        self.rangef64_with_default(name, range, 0.0)
    }

    /// Adds a [`f64`] range with a default value.
    #[must_use]
    #[inline]
    pub fn rangef64_with_default(
        self,
        name: &'static str,
        range: (f64, f64, f64),
        default: f64,
    ) -> Self {
        self.create_parameter(
            name,
            ParameterKind::RangeF64 {
                min: range.0,
                max: range.1,
                step: range.2.abs(),
                default,
            },
        )
    }

    /// Adds a characters sequence with a determined length.
    #[must_use]
    #[inline]
    pub fn characters_sequence(
        self,
        name: &'static str,
        default: impl Into<Cow<'static, str>>,
    ) -> Self {
        let default = default.into();
        self.create_parameter(
            name,
            ParameterKind::CharsSequence {
                length: default.len(),
                default,
            },
        )
    }

    /// Serializes [`Parameters`] data.
    ///
    /// It consumes the data.
    #[must_use]
    #[inline]
    pub fn serialize_data(self) -> ParametersData {
        let mut data = ParametersData::new();
        for (key, value) in self.0 {
            data.add(key.into(), value);
        }
        data
    }

    fn create_parameter(mut self, name: &'static str, parameter_kind: ParameterKind) -> Self {
        self.0.insert(name, parameter_kind);
        self
    }
}

/// All supported parameter values extracted from or
/// used to construct a request.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ParameterValue {
    /// A [`bool`] value.
    Bool(bool),
    /// A [`u8`] value.
    U8(u8),
    /// A [`u16`] value.
    U16(u16),
    /// A [`u32`] value.
    U32(u32),
    /// A [`u64`] value.
    U64(u64),
    /// A [`f32`] value.
    F32(f32),
    /// A [`f64`] value.
    F64(f64),
    /// A characters sequence.
    CharsSequence(Cow<'static, str>),
}

impl core::fmt::Display for ParameterValue {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Bool(v) => v.fmt(f),
            Self::U8(v) => v.fmt(f),
            Self::U16(v) => v.fmt(f),
            Self::U32(v) => v.fmt(f),
            Self::U64(v) => v.fmt(f),
            Self::F32(v) => v.fmt(f),
            Self::F64(v) => v.fmt(f),
            Self::CharsSequence(v) => v.fmt(f),
        }
    }
}

impl ParameterValue {
    /// Creates a [`ParameterValue`] from [`ParameterKind`].
    #[must_use]
    pub fn from_parameter_kind(parameter_kind: &ParameterKind) -> Self {
        match parameter_kind {
            ParameterKind::Bool { default } => Self::Bool(*default),
            ParameterKind::U8 { default, .. } => Self::U8(*default),
            ParameterKind::U16 { default, .. } => Self::U16(*default),
            ParameterKind::U32 { default, .. } => Self::U32(*default),
            ParameterKind::U64 { default, .. } | ParameterKind::RangeU64 { default, .. } => {
                Self::U64(*default)
            }
            ParameterKind::F32 { default, .. } => Self::F32(*default),
            ParameterKind::F64 { default, .. } | ParameterKind::RangeF64 { default, .. } => {
                Self::F64(*default)
            }
            ParameterKind::CharsSequence { default, .. } => Self::CharsSequence(default.clone()),
        }
    }
}

/// Route input parameters values.
#[derive(Debug, PartialEq, Deserialize)]
pub struct ParametersValues<'a>(IndexMap<Cow<'a, str>, ParameterValue, DefaultHashBuilder>);

impl Default for ParametersValues<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for &'a ParametersValues<'a> {
    type Item = (&'a Cow<'a, str>, &'a ParameterValue);
    type IntoIter = Iter<'a, Cow<'a, str>, ParameterValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> ParametersValues<'a> {
    /// Creates [`ParametersValues`].
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self(IndexMap::with_hasher(DefaultHashBuilder::default()))
    }

    /// Adds a [`ParameterValue`].
    #[inline]
    pub fn parameter_value(
        &mut self,
        name: impl Into<Cow<'a, str>>,
        parameter_value: ParameterValue,
    ) -> &mut Self {
        self.0.insert(name.into(), parameter_value);
        self
    }

    /// Adds a [`bool`] value.
    #[inline]
    pub fn bool(&mut self, name: impl Into<Cow<'a, str>>, value: bool) -> &mut Self {
        self.parameter_value(name, ParameterValue::Bool(value))
    }

    /// Adds an [`u8`] parameter.
    #[inline]
    pub fn u8(&mut self, name: impl Into<Cow<'a, str>>, value: u8) -> &mut Self {
        self.parameter_value(name, ParameterValue::U8(value))
    }

    /// Adds an [`u16`] parameter.
    #[inline]
    pub fn u16(&mut self, name: impl Into<Cow<'a, str>>, value: u16) -> &mut Self {
        self.parameter_value(name, ParameterValue::U16(value))
    }

    /// Adds an [`u32`] parameter.
    #[inline]
    pub fn u32(&mut self, name: impl Into<Cow<'a, str>>, value: u32) -> &mut Self {
        self.parameter_value(name, ParameterValue::U32(value))
    }

    /// Adds an [`u64`] parameter.
    #[inline]
    pub fn u64(&mut self, name: impl Into<Cow<'a, str>>, value: u64) -> &mut Self {
        self.parameter_value(name, ParameterValue::U64(value))
    }

    /// Adds a [`f32`] parameter.
    #[inline]
    pub fn f32(&mut self, name: impl Into<Cow<'a, str>>, value: f32) -> &mut Self {
        self.parameter_value(name, ParameterValue::F32(value))
    }

    /// Adds a [`f64`] parameter.
    #[inline]
    pub fn f64(&mut self, name: impl Into<Cow<'a, str>>, value: f64) -> &mut Self {
        self.parameter_value(name, ParameterValue::F64(value))
    }

    /// Adds a characters sequence.
    #[inline]
    pub fn characters_sequence(
        &mut self,
        name: impl Into<Cow<'a, str>>,
        value: String,
    ) -> &mut Self {
        self.parameter_value(name, ParameterValue::CharsSequence(value.into()))
    }

    /// Retrieves the [`ParameterValue`] by name.
    ///
    /// If [`None`], the parameter does not exist.
    #[must_use]
    #[inline]
    pub fn get<'b>(&'b self, name: impl Into<Cow<'b, str>>) -> Option<&'b ParameterValue> {
        self.0.get(&name.into())
    }

    /// Returns an iterator over [`ParameterValue`]s.
    ///
    /// **Iterates over the elements in the order they were inserted.**
    #[must_use]
    #[inline]
    pub fn iter(&self) -> Iter<'_, Cow<'_, str>, ParameterValue> {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use crate::{deserialize, serialize};

    use super::{ParameterKind, Parameters, ParametersData, ParametersValues};

    fn expected_parameters_data() -> ParametersData {
        ParametersData::new()
            .insert("bool".into(), ParameterKind::Bool { default: true })
            .insert(
                "u8".into(),
                ParameterKind::U8 {
                    default: 0,
                    min: u8::MIN,
                    max: u8::MAX,
                },
            )
            .insert(
                "u16".into(),
                ParameterKind::U16 {
                    default: 0,
                    min: u16::MIN,
                    max: u16::MAX,
                },
            )
            .insert(
                "u32".into(),
                ParameterKind::U32 {
                    default: 0,
                    min: u32::MIN,
                    max: u32::MAX,
                },
            )
            .insert(
                "u64".into(),
                ParameterKind::U64 {
                    default: 0,
                    min: u64::MIN,
                    max: u64::MAX,
                },
            )
            .insert(
                "f32".into(),
                ParameterKind::F32 {
                    default: 0.,
                    min: f32::MIN,
                    max: f32::MAX,
                    step: 0.,
                },
            )
            .insert(
                "f64".into(),
                ParameterKind::F64 {
                    default: 0.,
                    min: f64::MIN,
                    max: f64::MAX,
                    step: 0.,
                },
            )
            .insert(
                "rangeu64".into(),
                ParameterKind::RangeU64 {
                    min: 0,
                    max: 20,
                    step: 1,
                    default: 5,
                },
            )
            .insert(
                "rangef64".into(),
                ParameterKind::RangeF64 {
                    min: 0.,
                    max: 20.,
                    step: 0.1,
                    default: 5.,
                },
            )
            .insert(
                "greeting".into(),
                ParameterKind::CharsSequence {
                    default: "hello".into(),
                    length: 5,
                },
            )
            .insert(
                "greeting2".into(),
                ParameterKind::CharsSequence {
                    default: "hello".into(),
                    length: 5,
                },
            )
    }

    #[test]
    fn test_parameters() {
        let parameters = Parameters::new()
            .bool("bool", true)
            .u8("u8", 0)
            .u16("u16", 0)
            .u32("u32", 0)
            .u64("u64", 0)
            .f32("f32", 0.)
            .f64("f64", 0.)
            .rangeu64_with_default("rangeu64", (0, 20, 1), 5)
            .rangef64_with_default("rangef64", (0., 20., 0.1), 5.)
            .characters_sequence("greeting", "hello")
            .characters_sequence("greeting2", String::from("hello"))
            // Adds a duplicate to see whether that value is maintained or
            // removed.
            .u16("u16", 0);

        assert_eq!(
            deserialize::<ParametersData>(serialize(parameters.serialize_data())),
            expected_parameters_data(),
        );
    }

    #[test]
    fn test_deserialize_parameters_values() {
        let mut parameters = ParametersValues::new();
        parameters.bool("one", true);
        parameters.u8("two", 8);
        parameters.f32("three", 3.0);

        let json_value = serde_json::json!({
            "one": true,
            "two": 8,
            "three": 3.0,
        });

        assert_eq!(deserialize::<ParametersValues>(json_value), parameters);
    }
}
