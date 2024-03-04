use alloc::borrow::Cow;

use heapless::FnvIndexSet;
use serde::{Deserialize, Serialize};

use crate::hazards::{Hazard, HazardData};
use crate::MAXIMUM_ELEMENTS;

/// An [`InputType`] range defined as an interval.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputType {
    /// A [`u64`] type with a specific **[minimum, maximum, step]** range.
    RangeU64(Range<u64>),
    /// A [`f64`] type with a specific **[minimum, maximum, step]** range.
    RangeF64(Range<f64>),
    /// A [`bool`] type.
    Bool(bool),
}

/// All supported input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input<'a> {
    /// Name.
    pub name: Cow<'a, str>,
    /// Type.
    #[serde(rename = "type")]
    pub datatype: InputType,
}

impl<'a> core::cmp::PartialEq for Input<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<'a> core::cmp::Eq for Input<'a> {}

impl<'a> core::hash::Hash for Input<'a> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<'a> Input<'a> {
    /// Creates a new [`u64`] range.
    #[inline(always)]
    pub fn rangeu64(name: &'static str, range: (u64, u64, u64, u64)) -> Self {
        Self {
            name: name.into(),
            datatype: InputType::RangeU64(Self::range(range)),
        }
    }

    /// Creates a new [`f64`] range.
    #[inline(always)]
    pub fn rangef64(name: &'static str, range: (f64, f64, f64, f64)) -> Self {
        Self {
            name: name.into(),
            datatype: InputType::RangeF64(Self::range(range)),
        }
    }

    /// Creates a new [`bool`] range.
    #[inline(always)]
    pub fn boolean(name: &'static str, default: bool) -> Self {
        Self {
            name: name.into(),
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

/// Route data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteData<'a> {
    /// Name.
    pub name: String,
    /// Description.
    pub description: Option<Cow<'static, str>>,
    /// Stateless parameter.
    ///
    /// The route does not modify the internal state of a device.
    ///
    /// The default value is [`false`].
    pub stateless: bool,
    /// Inputs datatype.
    #[serde(skip_serializing_if = "FnvIndexSet::is_empty")]
    pub inputs: FnvIndexSet<Input<'a>, MAXIMUM_ELEMENTS>,
}

impl<'a> RouteData<'a> {
    /// Serializes [`RouteData`].
    pub fn serialize_data(&self) -> Self {
        let mut name = self.name.to_owned();
        self.inputs.iter().for_each(|input| {
            name = name.replace(&format!(":{}", input.name), &format!("<{}>", input.name));
        });
        Self {
            name,
            description: self.description.to_owned(),
            stateless: self.stateless,
            inputs: self.inputs.to_owned(),
        }
    }
}

/// Kind of a `REST` API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RestKind {
    // `GET` API.
    Get,
    // `PUT` API.
    Put,
    // `POST` API.
    Post,
}

/// A server route configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig<'a> {
    /// Route.
    #[serde(flatten)]
    pub data: RouteData<'a>,
    /// Kind of a `REST` API.
    #[serde(rename = "REST kind")]
    pub rest_kind: RestKind,
    /// Hazards data.
    #[serde(skip_serializing_if = "FnvIndexSet::is_empty")]
    pub hazards: FnvIndexSet<HazardData, MAXIMUM_ELEMENTS>,
}

/// A route to activate a precise smart home device task.
///
/// It represents a specific `REST` API which is being invoked by an external
/// process and run on the device.
#[derive(Debug)]
pub struct Route<'a> {
    /// Route name.
    pub route: &'static str,
    /// Hazards.
    pub hazards: FnvIndexSet<Hazard, MAXIMUM_ELEMENTS>,
    /// Route configuration.
    pub config: RouteConfig<'a>,
}

impl<'a> Eq for Route<'a> {}

impl<'a> PartialEq for Route<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.config.data.name == other.config.data.name
            && self.config.rest_kind == other.config.rest_kind
    }
}

impl<'a> core::hash::Hash for Route<'a> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.config.rest_kind.hash(state);
        self.config.data.name.hash(state);
    }
}

impl<'a> Route<'a> {
    /// Creates a new [`Route`] through a REST `GET` API.
    pub fn get(route: &'static str) -> Self {
        Self::init(RestKind::Get, route)
    }

    /// Creates a new [`Route`] through a REST `PUT` API.
    pub fn put(route: &'static str) -> Self {
        Self::init(RestKind::Put, route)
    }

    /// Creates a new [`Route`] through a REST `POST` API.
    pub fn post(route: &'static str) -> Self {
        Self::init(RestKind::Post, route)
    }

    /// Sets the action description.
    pub fn description(mut self, description: &'static str) -> Self {
        self.config.data.description = Some(description.into());
        self
    }

    /// Sets the route as stateless.
    pub fn stateless(mut self) -> Self {
        self.config.data.stateless = true;
        self
    }

    /// Sets a single [`Input`].
    pub fn input(mut self, input: Input<'a>) -> Self {
        let _ = self.config.data.inputs.insert(input);
        self
    }

    /// Sets [`Input`]s.
    pub fn inputs<const N: usize>(mut self, inputs: [Input<'a>; N]) -> Self {
        self.config.data.inputs = inputs.into_iter().take(MAXIMUM_ELEMENTS).collect();
        self
    }

    fn init(rest_kind: RestKind, route: &'static str) -> Self {
        Self {
            config: RouteConfig {
                rest_kind,
                hazards: FnvIndexSet::new(),
                data: RouteData {
                    name: route.into(),
                    description: None,
                    stateless: false,
                    inputs: FnvIndexSet::new(),
                },
            },
            route,
            hazards: FnvIndexSet::new(),
        }
    }
}
