//! TODO

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::collections::HashMap;

use ascot_library::device::DeviceEnvironment;
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

/// Parameters for `POST`, `PUT`, and `DELETE` requests.
#[derive(Debug, PartialEq)]
pub struct Parameters {
    /// Final route.
    pub route: String,
    /// Request parameters.
    // The insertion order is not important, so a simple HashMap can be used.
    pub params: HashMap<String, String>,
}

impl Parameters {
    fn empty(route: String) -> Self {
        Self {
            route,
            params: HashMap::new(),
        }
    }
}

/// All supported **_REST_** requests.
#[derive(Debug, PartialEq)]
pub enum Request {
    /// A `GET` request.
    Get(String),
    /// A `POST` request.
    Post(Parameters),
    /// A `PUT` request.
    Put(Parameters),
    /// A `DELETE` request.
    Delete(Parameters),
}

/// A **_REST_** request generator.
///
/// It builds a **_REST_** request starting from the route parameters
/// passed as input.
#[derive(Debug, PartialEq)]
pub struct RequestGenerator {
    device_environment: DeviceEnvironment,
    rest_kind: RestKind,
    // A base route is a route without parameters, formed by the composition of
    // the main address, the main route, and the effective route.
    base: String,
    // Use an index map to respect the insertion order, important to construct
    // requests.
    inputs: IndexMap<String, InputValue>,
}

impl RequestGenerator {
    /// Creates a [`RequestGenerator`].
    #[must_use]
    pub fn new(
        address: &str,
        device_environment: DeviceEnvironment,
        main_route: &str,
        route_config: &RouteConfig,
    ) -> Self {
        let address = slash_end(address);
        let main_route = slash_start_end(main_route);
        let route = slash_start_end(&route_config.data.name);

        let inputs = if route_config.data.inputs.is_empty() {
            IndexMap::new()
        } else {
            init_inputs(&route_config.data.inputs)
        };

        Self {
            device_environment,
            rest_kind: route_config.rest_kind,
            base: format!("{address}/{main_route}/{route}"),
            inputs,
        }
    }

    /// Checks whether a route has inputs.
    #[must_use]
    pub fn has_inputs(&self) -> bool {
        !self.inputs.is_empty()
    }

    /// Builds a [`Request`].
    #[must_use]
    pub fn build_request(&self) -> Request {
        let route = String::from(&self.base);

        match self.rest_kind {
            RestKind::Get => Request::Get(route),
            RestKind::Post => Request::Post(Parameters::empty(route)),
            RestKind::Put => Request::Put(Parameters::empty(route)),
            RestKind::Delete => Request::Delete(Parameters::empty(route)),
        }
    }

    /// Builds a [`Request`] having the given [`InputValue`] as
    /// route input parameter.
    ///
    /// If [`None`], the route input parameter does not exist or the
    /// [`InputValue`] type is not correct.
    pub fn build_request_with_input(
        &self,
        route_input: &str,
        value: InputValue,
    ) -> Option<Request> {
        let Some(input_value) = self.inputs.get(route_input) else {
            error!("{route_input} does not exist");
            return None;
        };

        if !value.same_type(input_value) {
            error!("{:?} does not have the same {:?} type", value, input_value);
            return None;
        }

        Some(match self.rest_kind {
            RestKind::Get => Request::Get(self.get_request(route_input, value)),
            RestKind::Post => Request::Post(self.param_request(route_input, value)),
            RestKind::Put => Request::Put(self.param_request(route_input, value)),
            RestKind::Delete => Request::Delete(self.param_request(route_input, value)),
        })
    }

    fn get_request(&self, route_input: &str, value: InputValue) -> String {
        match self.device_environment {
            DeviceEnvironment::Os => self.axum_get(route_input, value),
            // The server does not accept arguments.
            DeviceEnvironment::Esp32 => String::from(&self.base),
        }
    }

    fn param_request(&self, route_input: &str, value: InputValue) -> Parameters {
        let route = String::from(&self.base);

        let params = if self.inputs.is_empty() {
            HashMap::new()
        } else {
            self.build_params(route_input, value)
        };

        Parameters { route, params }
    }

    // Axum parameters: hello/{{1}}/{{2}}
    //                  hello/0.5/1
    fn axum_get(&self, route_input: &str, value: InputValue) -> String {
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

    fn build_params(&self, route_input: &str, value: InputValue) -> HashMap<String, String> {
        let mut params = HashMap::new();
        for input in &self.inputs {
            if input.0 == route_input {
                params.insert(route_input.into(), format!("{value}"));
            } else {
                params.insert(input.0.into(), format!("{}", input.1));
            }
        }
        params
    }
}

#[cfg(test)]
mod tests {
    use ascot_library::input::Input;
    use ascot_library::route::Route;

    use super::{
        DeviceEnvironment, HashMap, IndexMap, InputValue, Parameters, Request, RequestGenerator,
        RestKind,
    };

    const COMPLETE_ROUTE: &str = "http://hello.local/light/route";

    fn generator<F>(
        route: Route,
        rest_kind: RestKind,
        inputs: IndexMap<String, InputValue>,
        has_inputs: bool,
        compare_request: F,
    ) where
        F: FnOnce(RequestGenerator),
    {
        let route = route.serialize_data();

        let generator = RequestGenerator::new(
            "http://hello.local/",
            DeviceEnvironment::Os,
            "light/",
            &route,
        );

        assert_eq!(
            generator,
            RequestGenerator {
                device_environment: DeviceEnvironment::Os,
                rest_kind,
                base: COMPLETE_ROUTE.into(),
                inputs,
            }
        );

        // No input values.
        assert_eq!(generator.has_inputs(), has_inputs);

        // Wrong value.
        assert_eq!(
            generator.build_request_with_input("wrong", InputValue::U64(0)),
            None
        );

        // Wrong input type.
        assert_eq!(
            generator.build_request_with_input("rangeu64", InputValue::F64(0.)),
            None
        );

        compare_request(generator);
    }

    fn build_generator<F>(route: Route, rest_kind: RestKind, compare_request: F)
    where
        F: FnOnce(RequestGenerator),
    {
        generator(route, rest_kind, IndexMap::new(), false, compare_request);
    }

    fn build_generator_with_inputs<F>(route: Route, rest_kind: RestKind, compare_request: F)
    where
        F: FnOnce(RequestGenerator),
    {
        let route = route.with_inputs([
            Input::rangeu64_with_default("rangeu64", (0, 20, 1), 5),
            Input::rangef64("rangef64", (0., 20., 0.1)),
        ]);

        let mut inputs = IndexMap::with_capacity(2);
        inputs.insert("rangeu64".into(), InputValue::U64(5));
        inputs.insert("rangef64".into(), InputValue::F64(0.));

        generator(route, rest_kind, inputs, true, compare_request);
    }

    macro_rules! request {
        ($route:expr, $request:ident) => {
            build_generator_with_inputs($route, RestKind::$request, |generator| {
                let request_params = Parameters {
                    route: COMPLETE_ROUTE.into(),
                    params: HashMap::new(),
                };

                assert_eq!(generator.build_request(), Request::$request(request_params),);
            });
        };
    }

    macro_rules! request_with_inputs {
        ($route:expr, $request:ident) => {
            build_generator_with_inputs($route, RestKind::$request, |generator| {
                let mut params = HashMap::new();
                params.insert("rangeu64".into(), format!("{}", InputValue::U64(3)));
                params.insert("rangef64".into(), format!("{}", InputValue::F64(0.)));

                let request_params = Parameters {
                    route: COMPLETE_ROUTE.into(),
                    params,
                };

                assert_eq!(
                    generator.build_request_with_input("rangeu64", InputValue::U64(3)),
                    Some(Request::$request(request_params)),
                );
            });
        };
    }

    #[test]
    fn create_os_get_request() {
        build_generator(
            Route::get("/route").description("A GET route"),
            RestKind::Get,
            |generator| {
                assert_eq!(
                    generator.build_request(),
                    Request::Get(COMPLETE_ROUTE.into()),
                );
            },
        );
    }

    #[test]
    fn create_os_get_request_with_inputs() {
        build_generator_with_inputs(
            Route::get("/route").description("A GET route"),
            RestKind::Get,
            |generator| {
                assert_eq!(
                    generator.build_request_with_input("rangeu64", InputValue::U64(3)),
                    Some(Request::Get("http://hello.local/light/route/3/0".into())),
                );
            },
        );
    }

    #[test]
    fn create_os_post_request() {
        request!(Route::post("/route").description("A POST route."), Post);
    }

    #[test]
    fn create_os_put_request() {
        request!(Route::put("/route").description("A PUT route."), Put);
    }

    #[test]
    fn create_os_delete_request() {
        request!(
            Route::delete("/route").description("A DELETE route."),
            Delete
        );
    }

    #[test]
    fn create_os_post_request_with_inputs() {
        request_with_inputs!(Route::post("/route").description("A POST route."), Post);
    }

    #[test]
    fn create_os_put_request_with_inputs() {
        request_with_inputs!(Route::put("/route").description("A PUT route."), Put);
    }

    #[test]
    fn create_os_delete_request_with_inputs() {
        request_with_inputs!(
            Route::delete("/route").description("A DELETE route."),
            Delete
        );
    }
}
