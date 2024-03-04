use heapless::{FnvIndexSet, IndexSetIter};
use serde::{Deserialize, Serialize};

use crate::MAXIMUM_ELEMENTS;

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

/// Input data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputData<'a> {
    /// Name.
    pub name: &'a str,
    /// Type.
    #[serde(rename = "type")]
    pub datatype: InputType,
}

impl<'a> InputData<'a> {
    fn new(input: Input) -> Self {
        Self {
            name: input.name,
            datatype: input.datatype,
        }
    }
}

impl<'a> core::cmp::PartialEq for InputData<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<'a> core::cmp::Eq for InputData<'a> {}

impl<'a> core::hash::Hash for InputData<'a> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

/// A collection of [`InputData`]s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputsData<'a>(#[serde(borrow)] FnvIndexSet<InputData<'a>, MAXIMUM_ELEMENTS>);

impl<'a> InputsData<'a> {
    /// Initializes a new [`InputsData`] collection.
    pub fn init() -> Self {
        Self(FnvIndexSet::new())
    }

    /// Initializes a new [`InputsData`] collection from [`Inputs`].
    pub fn from_inputs(inputs: &Inputs) -> Self {
        let mut inputs_data = Self::init();
        for input in inputs.iter() {
            let _ = inputs_data.0.insert(InputData::new(*input));
        }
        inputs_data
    }

    /// Adds a new [`InputData`] to the [`InputsData`] collection.
    pub fn add(&mut self, input_data: InputData<'a>) {
        let _ = self.0.insert(input_data);
    }

    /// Whether the [`InputsData`] collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over [`InputData`]s.
    pub fn iter(&self) -> IndexSetIter<'_, InputData> {
        self.0.iter()
    }
}

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
    #[inline(always)]
    pub fn rangeu64(name: &'static str, range: (u64, u64, u64, u64)) -> Self {
        Self {
            name,
            datatype: InputType::RangeU64(Self::range(range)),
        }
    }

    /// Creates a new [`f64`] range.
    #[inline(always)]
    pub fn rangef64(name: &'static str, range: (f64, f64, f64, f64)) -> Self {
        Self {
            name,
            datatype: InputType::RangeF64(Self::range(range)),
        }
    }

    /// Creates a new [`bool`] range.
    #[inline(always)]
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
#[derive(Debug)]
pub struct Inputs(FnvIndexSet<Input, MAXIMUM_ELEMENTS>);

impl Inputs {
    /// Initializes a new [`Inputs`] collection.
    pub fn init() -> Self {
        Self(FnvIndexSet::new())
    }

    /// Adds a new [`Input`] to the [`Inputs`] collection.
    pub fn add(&mut self, input: Input) {
        let _ = self.0.insert(input);
    }

    /// Whether the [`Inputs`] collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Checks whether an [`Input`] is contained into
    /// the [`Inputs`] collection.
    pub fn contains(&self, input: &Input) -> bool {
        self.0.contains(input)
    }

    /// Returns an iterator over [`Input`]s.
    pub fn iter(&self) -> IndexSetIter<'_, Input> {
        self.0.iter()
    }
}
