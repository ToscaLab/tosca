use ascot_library::input::{InputStructure, InputsData};
use ascot_library::route::{RestKind, RouteConfig};

use indexmap::IndexMap;

use tracing::error;

fn slash_end(s: &str) -> &str {
    if s.len() > 1 && s.ends_with('/') {
        &s[..s.len() - 1]
    } else {
        s
    }
}

fn slash_start(s: &str) -> &str {
    if s.len() > 1 && s.starts_with('/') {
        &s[1..]
    } else {
        s
    }
}

fn slash_start_end(s: &str) -> &str {
    slash_start(slash_end(s))
}

fn init_inputs(data: &InputsData) -> IndexMap<String, InputValue> {
    let mut inputs = IndexMap::with_capacity(data.len());
    for input in data.iter() {
        let value = match input.structure {
            InputStructure::Bool { default } => InputValue::Bool(default),
            InputStructure::U8 { default } => InputValue::U8(default),
            InputStructure::RangeU64 {
                min: _,
                max: _,
                step: _,
                default,
            } => InputValue::U64(default),
            InputStructure::RangeF64 {
                min: _,
                max: _,
                step: _,
                default,
            } => InputValue::F64(default),
        };
        inputs.insert(input.name.to_string(), value);
    }
    inputs
}

/// All supported input values needed by the [`RequestGenerator`] to generate
/// **_REST_** requests.
#[derive(Clone, Copy, PartialEq)]
pub enum InputValue {
    /// [`bool`].
    Bool(bool),
    /// [`u8`].
    U8(u8),
    /// [`u64`].
    U64(u64),
    /// [`f64`].
    F64(f64),
}

impl InputValue {
    fn same_type(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Bool(_), Self::Bool(_))
                | (Self::U8(_), Self::U8(_))
                | (Self::U64(_), Self::U64(_))
                | (Self::F64(_), Self::F64(_))
        )
    }
}

impl std::fmt::Debug for InputValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::Bool(v) => format!("Bool[{v}]"),
            Self::U8(v) => format!("U8[{v}]"),
            Self::U64(v) => format!("U64[{v}]"),
            Self::F64(v) => format!("F64[{v}]"),
        };
        s.fmt(f)
    }
}

impl std::fmt::Display for InputValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::Bool(v) => v.to_string(),
            Self::U8(v) => v.to_string(),
            Self::U64(v) => v.to_string(),
            Self::F64(v) => v.to_string(),
        };
        s.fmt(f)
    }
}

/// All supported **_REST_** requests.
#[derive(Debug, PartialEq)]
pub enum Request {
    /// A `GET` request.
    Get(String),
    /*Post(Post),
    Put(Put),
    Delete(Delete),*/
}

/// A **_REST_** request generator.
///
/// It builds a **_REST_** request starting from the route parameters
/// passed as input.
#[derive(Debug, PartialEq)]
pub struct RequestGenerator {
    rest_kind: RestKind,
    // A base route is a route without parameters, formed by the composition of
    // the main address, the main route, and the effective route.
    base: String,
    // Use an index map to respect the insertion order, important to construct
    // requests.
    inputs: IndexMap<String, InputValue>,
}

impl RequestGenerator {
    /// Creates a [`RestGenerator`].
    #[must_use]
    pub fn new(address: &str, main_route: &str, route_config: &RouteConfig) -> Self {
        let address = slash_end(address);
        let main_route = slash_start_end(main_route);
        let route = slash_start_end(&route_config.data.name);

        let inputs = if route_config.data.inputs.is_empty() {
            IndexMap::new()
        } else {
            init_inputs(&route_config.data.inputs)
        };

        Self {
            rest_kind: route_config.rest_kind,
            base: format!("{address}/{main_route}/{route}"),
            inputs,
        }
    }

    /// Constructs a **_REST_** request using the [`InputValue`] passed as
    /// input for the route input parameter.
    ///
    /// If [`None`], the route input parameter does not exist or the
    /// [`InputValue`] type is not correct.
    pub fn construct_request(&self, route_input: &str, value: InputValue) -> Option<Request> {
        let Some(input_value) = self.inputs.get(route_input) else {
            error!("{route_input} does not exist");
            return None;
        };

        if !value.same_type(input_value) {
            error!("{:?} does not have the same {:?} type", value, input_value);
            return None;
        }

        Some(match self.rest_kind {
            _ => Request::Get(self.get_request(route_input, value)),
            /*RestKind::Post => Request::Post(self.post_request(route_input, value)),
            RestKind::Put => Request::Put(self.put_request(route_input, value)),
            RestKind::Delete => Request::Delete(self.delete_request(route_input, value)),*/
        })
    }

    fn get_request(&self, route_input: &str, value: InputValue) -> String {
        // TODO: Consider different architectures to join values in the ascot-library
        // - standard: i.e. ?hello=0 - ?hello=3.5
        // - axum: i.e. /0. - /3.5
        let mut route = String::from(&self.base);
        for input in &self.inputs {
            let input_value = if input.0 == route_input {
                &value
            } else {
                input.1
            };
            route.push_str(&format!("/{input_value}"));
        }

        route
    }
}

#[cfg(test)]
mod tests {
    use ascot_library::input::Input;
    use ascot_library::route::Route;

    use super::{IndexMap, InputValue, Request, RequestGenerator, RestKind};

    #[test]
    fn create_route() {
        let route = Route::get("/route")
            .description("A GET route")
            .with_inputs([
                Input::rangeu64_with_default("rangeu64", (0, 20, 1), 5),
                Input::rangef64("rangef64", (0., 20., 0.1)),
            ])
            .serialize_data();

        let generator = RequestGenerator::new("http://hello.local/", "light/", &route);

        let mut inputs = IndexMap::with_capacity(2);
        inputs.insert("rangeu64".into(), InputValue::U64(5));
        inputs.insert("rangef64".into(), InputValue::F64(0.));

        assert_eq!(
            generator,
            RequestGenerator {
                rest_kind: RestKind::Get,
                base: "http://hello.local/light/route".into(),
                inputs
            }
        );

        // Wrong value.
        assert_eq!(
            generator.construct_request("wrong", InputValue::U64(0)),
            None
        );

        // Wrong input type.
        assert_eq!(
            generator.construct_request("rangeu64", InputValue::F64(0.)),
            None
        );

        assert_eq!(
            generator.construct_request("rangeu64", InputValue::U64(3)),
            Some(Request::Get("http://hello.local/light/route/3/0".into())),
        );
    }
}
