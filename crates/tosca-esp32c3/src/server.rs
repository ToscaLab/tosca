use core::fmt::{Debug, Display};
use core::net::{Ipv4Addr, SocketAddr};
use core::pin::Pin;

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::format;
use alloc::str::SplitTerminator;
use alloc::string::ToString;
use alloc::vec::Vec;

use tosca::parameters::{
    ParameterId, ParameterKind, ParameterValue, ParametersValues as ToscaParametersValues,
};
use tosca::route::{RestKind, RouteConfig};

use edge_http::io::Body;
use edge_http::io::server::{Connection, Handler, Server as EdgeServer};
use edge_http::{Headers, Method};
use edge_nal::TcpBind;
use edge_nal_embassy::{Tcp, TcpBuffers};

use embassy_executor::Spawner;
use embassy_net::Stack;

use embedded_io_async::{Read, Write};

use log::{error, info};

use crate::device::Device;
use crate::error::Error;
use crate::mdns::Mdns;
use crate::parameters::ParametersValues;
use crate::response::{ErrorResponse, InfoResponse, OkResponse, Response, SerialResponse};
use crate::state::{State, ValueFromRef};

// Default HTTP address.
//
// The entire local network is considered, so the Ipv4 unspecified address is
// used.
const DEFAULT_HTTP_ADDRESS: Ipv4Addr = Ipv4Addr::UNSPECIFIED;

// Default port.
const DEFAULT_SERVER_PORT: u16 = 80;

// Number of sockets used for the HTTP server
const SERVER_SOCKETS: usize = 1;

// Maximum request size in bytes.
const MAXIMUM_REQUEST_SIZE: usize = 128;

pub(crate) type OkFn = Box<
    dyn Fn(
            ParametersValues,
        ) -> Pin<
            Box<dyn Future<Output = Result<OkResponse, ErrorResponse>> + Send + Sync + 'static>,
        > + Send
        + Sync
        + 'static,
>;

pub(crate) type OkStateFn<S> = Box<
    dyn Fn(
            State<S>,
            ParametersValues,
        ) -> Pin<
            Box<dyn Future<Output = Result<OkResponse, ErrorResponse>> + Send + Sync + 'static>,
        > + Send
        + Sync
        + 'static,
>;

pub(crate) type SerialFn = Box<
    dyn Fn(
            ParametersValues,
        ) -> Pin<
            Box<dyn Future<Output = Result<SerialResponse, ErrorResponse>> + Send + Sync + 'static>,
        > + Send
        + Sync
        + 'static,
>;

pub(crate) type SerialStateFn<S> = Box<
    dyn Fn(
            State<S>,
            ParametersValues,
        ) -> Pin<
            Box<dyn Future<Output = Result<SerialResponse, ErrorResponse>> + Send + Sync + 'static>,
        > + Send
        + Sync
        + 'static,
>;

pub(crate) type InfoFn = Box<
    dyn Fn(
            ParametersValues,
        ) -> Pin<
            Box<dyn Future<Output = Result<InfoResponse, ErrorResponse>> + Send + Sync + 'static>,
        > + Send
        + Sync
        + 'static,
>;

pub(crate) type InfoStateFn<S> = Box<
    dyn Fn(
            State<S>,
            ParametersValues,
        ) -> Pin<
            Box<dyn Future<Output = Result<InfoResponse, ErrorResponse>> + Send + Sync + 'static>,
        > + Send
        + Sync
        + 'static,
>;

pub(crate) type Functions<S> = (
    Vec<OkFn>,
    Vec<OkStateFn<S>>,
    Vec<SerialFn>,
    Vec<SerialStateFn<S>>,
    Vec<InfoFn>,
    Vec<InfoStateFn<S>>,
);

#[derive(Clone, Copy)]
pub(crate) enum FuncType {
    OkStateless,
    OkStateful,
    SerialStateless,
    SerialStateful,
    InfoStateless,
    InfoStateful,
}

#[derive(Clone, Copy)]
pub(crate) struct FuncIndex {
    func_type: FuncType,
    index: usize,
}

impl FuncIndex {
    pub(crate) const fn new(func_type: FuncType, index: usize) -> Self {
        Self { func_type, index }
    }
}

/// The `tosca` server.
pub struct Server<
    const TX_SIZE: usize,
    const RX_SIZE: usize,
    const MAXIMUM_HEADERS_COUNT: usize,
    const TIMEOUT: u32,
    S,
> where
    S: ValueFromRef + Send + Sync + 'static,
{
    // HTTP address.
    address: Ipv4Addr,
    // Server port.
    port: u16,
    // HTTP handler.
    handler: ServerHandler<S>,
    // mDNS
    mdns: Mdns,
}

impl<
    const TX_SIZE: usize,
    const RX_SIZE: usize,
    const MAXIMUM_HEADERS_COUNT: usize,
    const TIMEOUT: u32,
    S,
> Server<TX_SIZE, RX_SIZE, MAXIMUM_HEADERS_COUNT, TIMEOUT, S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    /// Creates a [`Server`].
    #[inline]
    pub fn new(device: Device<S>, mdns: Mdns) -> Self {
        Self {
            address: DEFAULT_HTTP_ADDRESS,
            port: DEFAULT_SERVER_PORT,
            handler: ServerHandler::new(device),
            mdns,
        }
    }

    /// Sets the `IPv4` address for the server.
    #[must_use]
    pub const fn address(mut self, address: Ipv4Addr) -> Self {
        self.address = address;
        self
    }

    /// Sets the port number for the server to listen on.
    #[must_use]
    pub const fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Runs the [`Server`] and the [`Mdns`] task.
    ///
    /// # Errors
    ///
    /// - Failure to bind TCP protocol buffers to the underlying socket
    /// - Failure to spawn the `mDNS` task
    /// - Failure to run the server
    pub async fn run(self, stack: Stack<'static>, spawner: Spawner) -> Result<(), Error> {
        let buffers = TcpBuffers::<SERVER_SOCKETS, TX_SIZE, RX_SIZE>::new();
        let tcp = Tcp::new(stack, &buffers);

        let socket = SocketAddr::new(self.address.into(), self.port);

        let acceptor = tcp.bind(socket).await?;

        let mut server = EdgeServer::<SERVER_SOCKETS, RX_SIZE, MAXIMUM_HEADERS_COUNT>::new();

        // Run mdns.
        //
        // NOTE: Use the same server port for the mDNS-SD service
        self.mdns.run(stack, self.port, spawner)?;

        info!(
            "Starting server on address `{}` and port `{}`",
            self.address, self.port
        );

        // Run server.
        server
            .run(Some(TIMEOUT), acceptor, self.handler)
            .await
            .map_err(core::convert::Into::into)
    }
}

const fn method_map(ascot_method: RestKind) -> Method {
    match ascot_method {
        RestKind::Get => Method::Get,
        RestKind::Put => Method::Put,
        RestKind::Post => Method::Post,
        RestKind::Delete => Method::Delete,
    }
}

#[inline]
fn error_response_with_error(description: &str, error: &str) -> Response {
    error!("{description}: {error}");
    ErrorResponse::internal_with_error(description, error).0
}

#[inline]
fn error_response(description: &str) -> Response {
    error!("{description}");
    ErrorResponse::internal(description).0
}

#[inline]
fn invalid_data_response(description: &str) -> Response {
    invalid_data(description).0
}

#[inline]
pub(crate) fn invalid_data(description: &str) -> ErrorResponse {
    error!("{description}");
    ErrorResponse::invalid_data(description)
}

struct RouteInfo {
    index: usize,
    parameters_values: ParametersValues,
}

impl RouteInfo {
    const fn new(index: usize, parameters_values: ToscaParametersValues<'static>) -> Self {
        Self {
            index,
            parameters_values: ParametersValues(parameters_values),
        }
    }
}

struct ServerHandler<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    device: Device<S>,
}

impl<S> ServerHandler<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    #[inline]
    fn new(device: Device<S>) -> Self {
        Self { device }
    }

    async fn analyze_route<const N: usize, T: Read>(
        &self,
        method: Method,
        path: &str,
        headers: &Headers<'_, N>,
        body: &mut Body<'_, T>,
    ) -> Result<RouteInfo, Response> {
        // If the last character of a path ends with '/', remove it.
        let path = path.strip_suffix('/').unwrap_or(path);

        info!("Complete path: {path}");

        // Create an iterator to parse the route in an iterative way. This
        // function removes the trailing '/' which might be present in route
        // definition.
        let mut route_iter = path.split_terminator('/');

        // Every time a `nth(0)` function is being called, the iterator
        // consumes the current element.
        //
        // The first 0-element of a route is always an empty path
        // because **each** path begins with a '/'.
        //
        // In case of error, return a not found route.
        let empty_path = route_iter.nth(0).ok_or_else(Response::not_found)?;

        // If the empty path is equal to the route path,
        // the route is not correct. This might happen when a route
        // path makes use of a wrong separator, both for route subpaths, but
        // also for its parameters.
        //
        // In case of error, return a not found route.
        if empty_path == path {
            return Err(Response::not_found());
        }

        // Retrieve the main route.
        let main_route_path = route_iter.nth(0).ok_or_else(Response::not_found)?;

        // If the subpath is not equal to the main route,
        // the route is not correct. Starts from the 1-index
        // in order to skip the "/" placed before the main route.
        if main_route_path != &self.device.main_route[1..] {
            return Err(Response::not_found());
        }

        let mut route_index = self.device.route_configs.len();
        for (index, route) in self.device.route_configs.iter().enumerate() {
            // If the request REST method is different from the route
            // method, skip to the next route.
            if method != method_map(route.rest_kind) {
                continue;
            }

            let route_path = &route.data.path[1..];

            // If the entire route path does not include the specific segment
            // that characterizes the route, proceed to the next route.
            if !path.contains(route_path) {
                continue;
            }

            info!("Route path: {route_path}");

            for _ in 0..route_path.split_terminator('/').count() {
                route_iter.nth(0).ok_or_else(Response::not_found)?;
            }

            // If the route has no parameters, return its index.
            //
            // Otherwise, save the index and break the loop,
            // as there are parameters to analyze.
            if route.data.parameters.is_empty() {
                return Ok(RouteInfo::new(index, ToscaParametersValues::new()));
            }

            route_index = index;
            break;
        }

        // Retrieve the route configuration.
        let route_config = self
            .device
            .route_configs
            .get_index(route_index)
            .ok_or_else(Response::not_found)?;

        match method {
            Method::Get => Self::parse_get_parameters(route_config, route_iter),
            // NOTE: We include the disallowed methods here as well, since
            // the check has already been performed earlier.
            _ => Self::parse_headers_parameters(route_config, headers, body).await,
        }
        .map(|parameters_values| RouteInfo::new(route_index, parameters_values))
    }

    #[inline]
    fn parse_get_parameters(
        route_config: &RouteConfig,
        mut route_iter: SplitTerminator<'_, char>,
    ) -> Result<ToscaParametersValues<'static>, Response> {
        // Create parameters values.
        let mut parameters_values = ToscaParametersValues::new();

        for (index, parameter) in route_config.data.parameters.iter().enumerate() {
            let parameter_value = route_iter.nth(0).ok_or_else(|| {
                invalid_data_response(&format!(
                    "Passed route path is too short, missing parameters: {:?}",
                    route_config
                        .data
                        .parameters
                        .iter()
                        .skip(index)
                        .map(|parameter| parameter.0.as_str())
                        .collect::<Vec<&str>>()
                ))
            })?;

            info!("Parameter value as string: {parameter_value}");
            let parameter_value = Self::parse_parameter_value(parameter_value, parameter.1)?;

            parameters_values.parameter_value(parameter.0.clone(), parameter_value);
        }

        // NOTE: We do not check whether a route path still contains other ,
        // as this is unnecessary since all parameters have been taken.
        Ok(parameters_values)
    }

    #[inline]
    async fn parse_headers_parameters<const N: usize, T: Read>(
        route_config: &RouteConfig,
        headers: &Headers<'_, N>,
        body: &mut Body<'_, T>,
    ) -> Result<ToscaParametersValues<'static>, Response> {
        info!("Headers: {headers:?}");

        let content_length = headers
            .get("Content-Length")
            .ok_or_else(|| invalid_data_response("No `Content-Length` found"))?;

        let content_length = content_length.parse::<usize>().map_err(|e| {
            error_response_with_error(
                "Unable to convert the `Content-Length` header into a number",
                &format!("{e}"),
            )
        })?;

        if content_length > MAXIMUM_REQUEST_SIZE {
            return Err(error_response(&format!(
                "The request exceeds the maximum allowed size of {MAXIMUM_REQUEST_SIZE} and cannot be processed"
            )));
        }

        let content_type = headers
            .content_type()
            .ok_or_else(|| invalid_data_response("No `Content-Type` found"))?;

        if content_type != "application/json" {
            return Err(invalid_data_response(
                "The request body does not have a JSON format as content type",
            ));
        }

        let mut bytes = [0; MAXIMUM_REQUEST_SIZE];
        body.read(&mut bytes).await.map_err(|e| {
            error_response_with_error("Error reading the request bytes", &format!("{e:?}"))
        })?;

        let route_parameters = serde_json::from_slice::<ToscaParametersValues>(
            &bytes[0..content_length],
        )
        .map_err(|e| {
            error_response_with_error(
                "Failed to convert bytes into a sequence of parameters",
                &format!("{e}"),
            )
        })?;

        info!("Route parameters: {route_parameters:?}");

        for parameter_config in &route_config.data.parameters {
            let route_parameter_value =
                route_parameters.get(parameter_config.0).ok_or_else(|| {
                    invalid_data_response(&format!("Parameter `{}` not found", parameter_config.0))
                })?;

            let route_parameter_value_id = ParameterId::from_parameter_value(route_parameter_value);
            let route_parameter_kind_id = ParameterId::from_parameter_kind(parameter_config.1);

            if route_parameter_value_id != route_parameter_kind_id {
                return Err(invalid_data_response(&format!(
                    "Found id `{}`, expected id `{}`",
                    route_parameter_value_id.to_str(),
                    route_parameter_kind_id.to_str()
                )));
            }
        }

        Ok(route_parameters)
    }

    fn parse_parameter_value(
        parameter_value: &str,
        parameter_kind: &ParameterKind,
    ) -> Result<ParameterValue, Response> {
        match parameter_kind {
            ParameterKind::Bool { .. } => {
                Self::into_value::<bool, _>(parameter_value, "bool", ParameterValue::Bool)
            }
            ParameterKind::U8 { .. } => {
                Self::into_value::<u8, _>(parameter_value, "u8", ParameterValue::U8)
            }
            ParameterKind::U16 { .. } => {
                Self::into_value::<u16, _>(parameter_value, "u16", ParameterValue::U16)
            }
            ParameterKind::U32 { .. } => {
                Self::into_value::<u32, _>(parameter_value, "u32", ParameterValue::U32)
            }
            ParameterKind::U64 { .. } | ParameterKind::RangeU64 { .. } => {
                Self::into_value::<u64, _>(parameter_value, "u64", ParameterValue::U64)
            }
            ParameterKind::F32 { .. } => {
                Self::into_value::<f32, _>(parameter_value, "f32", ParameterValue::F32)
            }
            ParameterKind::F64 { .. } | ParameterKind::RangeF64 { .. } => {
                Self::into_value::<f64, _>(parameter_value, "f64", ParameterValue::F64)
            }
            ParameterKind::CharsSequence { .. } => Ok(ParameterValue::CharsSequence(Cow::Owned(
                parameter_value.to_string(),
            ))),
        }
    }

    #[inline]
    fn into_value<T, F>(
        parameter_value: &str,
        type_msg: &str,
        parameter_value_generator: F,
    ) -> Result<ParameterValue, Response>
    where
        T: core::str::FromStr,
        <T as core::str::FromStr>::Err: Display,
        F: FnOnce(T) -> ParameterValue,
    {
        parameter_value
            .parse::<T>()
            .map(parameter_value_generator)
            .map_err(|e| {
                error_response_with_error(
                    &format!("Failed to parse `{parameter_value}` into `{type_msg}` type"),
                    &format!("{e}"),
                )
            })
    }

    #[inline]
    async fn run_function(&self, index: usize, parameters_values: ParametersValues) -> Response {
        let func_index = self.device.index_array[index];

        match func_index.func_type {
            FuncType::OkStateless => {
                let func = &self.device.routes_functions.0[func_index.index];
                func(parameters_values).await.into()
            }
            FuncType::OkStateful => {
                let func = &self.device.routes_functions.1[func_index.index];
                func(
                    State(S::value_from_ref(&self.device.state.0)),
                    parameters_values,
                )
                .await
                .into()
            }
            FuncType::SerialStateless => {
                let func = &self.device.routes_functions.2[func_index.index];
                func(parameters_values).await.into()
            }
            FuncType::SerialStateful => {
                let func = &self.device.routes_functions.3[func_index.index];
                func(
                    State(S::value_from_ref(&self.device.state.0)),
                    parameters_values,
                )
                .await
                .into()
            }
            FuncType::InfoStateless => {
                let func = &self.device.routes_functions.4[func_index.index];
                func(parameters_values).await.into()
            }
            FuncType::InfoStateful => {
                let func = &self.device.routes_functions.5[func_index.index];
                func(
                    State(S::value_from_ref(&self.device.state.0)),
                    parameters_values,
                )
                .await
                .into()
            }
        }
    }

    const fn is_method_allowed(method: Method) -> bool {
        !matches!(
            method,
            Method::Get | Method::Post | Method::Put | Method::Delete
        )
    }
}

impl<S: ValueFromRef + Send + Sync + 'static> Handler for ServerHandler<S> {
    type Error<E>
        = edge_http::io::Error<E>
    where
        E: Debug;

    async fn handle<T, const N: usize>(
        &self,
        _task_id: impl Display + Copy,
        conn: &mut Connection<'_, T, N>,
    ) -> Result<(), Self::Error<T::Error>>
    where
        T: Read + Write,
    {
        let (headers, body) = conn.split();

        if headers.path == "/" {
            return self.device.main_route_response.write_from_ref(conn).await;
        }

        if Self::is_method_allowed(headers.method) {
            return Response::not_allowed().write(conn).await;
        }

        let route_info = match self
            .analyze_route(headers.method, headers.path, &headers.headers, body)
            .await
        {
            Ok(index) => index,
            Err(response) => return response.write(conn).await,
        };

        let RouteInfo {
            index,
            parameters_values,
        } = route_info;

        let response = self.run_function(index, parameters_values).await;
        response.write(conn).await
    }
}
