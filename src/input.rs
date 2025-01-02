use serde::{Deserialize, Serialize};

use crate::collections::Collection;

/// An [`InputType`] range defined as an interval.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Range<T> {
    /// Minimum.
    pub minimum: T,
    /// Maximum.
    pub maximum: T,
    /// Step.
    pub step: T,
    /// Default.
    pub default: T,
}

/// All supported [`Input`] types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InputType {
    /// A [`u64`] type with a specific **[minimum, maximum, step]** range.
    RangeU64(Range<u64>),
    /// A [`f64`] type with a specific **[minimum, maximum, step]** range.
    RangeF64(Range<f64>),
    /// A [`bool`] type.
    Bool(bool),
}

#[cfg(feature = "std")]
mod input_data {
    use alloc::string::String;

    use super::*;

    /// Input data.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InputData {
        /// Name.
        pub name: String,
        /// Type.
        #[serde(rename = "type")]
        pub datatype: InputType,
    }

    impl InputData {
        pub(super) fn new(input: Input) -> Self {
            Self {
                name: String::from(input.name),
                datatype: input.datatype,
            }
        }
    }

    /// A collection of [`InputData`]s.
    pub type InputsData = crate::collections::OutputCollection<InputData>;
}

#[cfg(not(feature = "std"))]
mod input_data {
    use super::*;

    /// Input data.
    #[derive(Debug, Clone, Serialize)]
    pub struct InputData {
        /// Name.
        pub name: &'static str,
        /// Type.
        #[serde(rename = "type")]
        pub datatype: InputType,
    }

    impl InputData {
        pub(super) fn new(input: Input) -> Self {
            Self {
                name: input.name,
                datatype: input.datatype,
            }
        }
    }

    /// A collection of [`InputData`]s.
    pub type InputsData = crate::collections::SerialCollection<InputData>;
}

impl core::cmp::PartialEq for input_data::InputData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl core::cmp::Eq for input_data::InputData {}

impl core::hash::Hash for input_data::InputData {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl From<Input> for input_data::InputData {
    fn from(input: Input) -> Self {
        Self::new(input)
    }
}

pub use input_data::InputData;
pub use input_data::InputsData;

/// All supported inputs.
#[derive(Debug, Clone, Copy)]
pub struct Input {
    // Name.
    pub(crate) name: &'static str,
    // Type.
    datatype: InputType,
}

impl core::cmp::PartialEq for Input {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl core::cmp::Eq for Input {}

impl core::hash::Hash for Input {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Input {
    /// Creates a new [`u64`] range.
    #[must_use]
    #[inline]
    pub fn rangeu64(name: &'static str, range: (u64, u64, u64, u64)) -> Self {
        Self {
            name,
            datatype: InputType::RangeU64(Self::range(range)),
        }
    }

    /// Creates a new [`f64`] range.
    #[must_use]
    #[inline]
    pub fn rangef64(name: &'static str, range: (f64, f64, f64, f64)) -> Self {
        Self {
            name,
            datatype: InputType::RangeF64(Self::range(range)),
        }
    }

    /// Creates a new [`bool`] range.
    #[must_use]
    #[inline]
    pub fn boolean(name: &'static str, default: bool) -> Self {
        Self {
            name,
            datatype: InputType::Bool(default),
        }
    }

    fn range<T>(range: (T, T, T, T)) -> Range<T> {
        Range {
            minimum: range.0,
            maximum: range.1,
            step: range.2,
            default: range.3,
        }
    }
}

/// A collection of [`Input`]s.
pub type Inputs = Collection<Input>;
