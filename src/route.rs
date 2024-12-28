use serde::{Deserialize, Serialize};

use crate::collections::{Collection, OutputCollection};
use crate::hazards::{Hazard, Hazards, HazardsData};
use crate::input::{Input, Inputs, InputsData};

use crate::MAXIMUM_ELEMENTS;

/// Route data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteData<'a> {
    /// Name.
    pub name: &'a str,
    /// Description.
    pub description: Option<&'a str>,
    /// Inputs associated with a route..
    #[serde(skip_serializing_if = "InputsData::is_empty")]
    pub inputs: InputsData<'a>,
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
    // `DELETE` API
    Delete,
}

impl core::fmt::Display for RestKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Get => "GET",
            Self::Put => "PUT",
            Self::Post => "POST",
            Self::Delete => "DELETE",
        }
        .fmt(f)
    }
}

/// A server route configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig<'a> {
    /// Route.
    #[serde(flatten, borrow)]
    pub data: RouteData<'a>,
    /// Kind of a `REST` API.
    #[serde(rename = "REST kind")]
    pub rest_kind: RestKind,
    /// Hazards data.
    #[serde(skip_serializing_if = "HazardsData::is_empty")]
    pub hazards: HazardsData<'a>,
}

impl PartialEq for RouteConfig<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.data.name.eq(other.data.name) && self.rest_kind == other.rest_kind
    }
}

impl Eq for RouteConfig<'_> {}

impl core::hash::Hash for RouteConfig<'_> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.data.name.hash(state);
        self.rest_kind.hash(state);
    }
}

/// A collection of [`RouteConfig`]s.
pub type RouteConfigs<'a> = OutputCollection<RouteConfig<'a>>;

/// A server route.
///
/// It represents a specific `REST` API which, when invoked, runs a task on
/// a remote device.
#[derive(Debug)]
pub struct Route {
    // Route.
    route: &'static str,
    // REST kind.
    rest_kind: RestKind,
    // Description.
    description: Option<&'static str>,
    // Inputs.
    inputs: Inputs,
}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.route == other.route && self.rest_kind == other.rest_kind
    }
}

impl Eq for Route {}

impl core::hash::Hash for Route {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.route.hash(state);
        self.rest_kind.hash(state);
    }
}

impl Route {
    /// Creates a new [`Route`] through a REST `GET` API.
    #[must_use]
    pub const fn get(route: &'static str) -> Self {
        Self::init(RestKind::Get, route)
    }

    /// Creates a new [`Route`] through a REST `PUT` API.
    #[must_use]
    pub const fn put(route: &'static str) -> Self {
        Self::init(RestKind::Put, route)
    }

    /// Creates a new [`Route`] through a REST `POST` API.
    #[must_use]
    pub const fn post(route: &'static str) -> Self {
        Self::init(RestKind::Post, route)
    }

    /// Sets the route description.
    #[must_use]
    pub const fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets a single [`Input`].
    #[must_use]
    #[inline]
    pub fn input(mut self, input: Input) -> Self {
        self.inputs.add(input);
        self
    }

    /// Sets more [`Input`]s.
    #[must_use]
    #[inline]
    pub fn inputs<const N: usize>(mut self, inputs: [Input; N]) -> Self {
        inputs.into_iter().take(MAXIMUM_ELEMENTS).for_each(|input| {
            self.inputs.add(input);
        });
        self
    }

    /// Returns route.
    #[must_use]
    pub fn route(&self) -> &str {
        self.route
    }

    /// Returns [`RestKind`].
    #[must_use]
    pub const fn kind(&self) -> RestKind {
        self.rest_kind
    }

    const fn init(rest_kind: RestKind, route: &'static str) -> Self {
        Self {
            route,
            rest_kind,
            description: None,
            inputs: Inputs::empty(),
        }
    }
}

/// A route with its associated hazards.
#[derive(Debug)]
pub struct RouteHazards {
    /// Route.
    pub route: Route,
    /// Hazards.
    pub hazards: Hazards,
}

impl core::cmp::PartialEq for RouteHazards {
    fn eq(&self, other: &Self) -> bool {
        self.route.eq(&other.route)
    }
}

impl core::cmp::Eq for RouteHazards {}

impl core::hash::Hash for RouteHazards {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.route.hash(state);
    }
}

impl RouteHazards {
    /// Creates a [`RouteHazards`] instance.
    #[must_use]
    pub const fn new(route: Route, hazards: Hazards) -> Self {
        Self { route, hazards }
    }

    /// Creates a [`RouteHazards`] without any hazard.
    #[must_use]
    pub const fn no_hazards(route: Route) -> Self {
        Self {
            route,
            hazards: Hazards::empty(),
        }
    }

    /// Initializes a [`RouteHazards`] with a single [`Hazard`].
    #[must_use]
    #[inline]
    pub fn single_hazard(route: Route, hazard: Hazard) -> Self {
        Self {
            route,
            hazards: Hazards::init(hazard),
        }
    }

    /// Initializes a [`RouteHazards`] with some [`Hazard`]s.
    #[must_use]
    #[inline]
    pub fn with_hazards(route: Route, hazards: &'static [Hazard]) -> Self {
        Self {
            route,
            hazards: Hazards::init_with_elements(hazards),
        }
    }

    /// Serializes [`RouteHazards`] data.
    #[must_use]
    #[inline]
    pub fn serialize_data(&self) -> RouteConfig {
        RouteConfig {
            rest_kind: self.route.rest_kind,
            hazards: HazardsData::from(&self.hazards),
            data: RouteData {
                name: self.route.route(),
                description: self.route.description,
                inputs: InputsData::from(&self.route.inputs),
            },
        }
    }
}

/// A collection of [`RouteHazards`]s.
pub type RoutesHazards = Collection<RouteHazards>;
