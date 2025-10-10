use alloc::vec::Vec;

use ascot::route::RouteConfigs;

use crate::response::Response;
use crate::server::{FuncIndex, Functions};
use crate::state::{State, ValueFromRef};

/// A general device.
pub struct Device<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    pub(crate) main_route: &'static str,
    pub(crate) state: State<S>,
    pub(crate) routes_functions: Functions<S>,
    pub(crate) route_configs: RouteConfigs,
    pub(crate) index_array: Vec<FuncIndex>,
    pub(crate) main_route_response: Response,
}

impl<S> Device<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    #[inline]
    pub(crate) fn new(
        main_route: &'static str,
        state: State<S>,
        routes_functions: Functions<S>,
        index_array: Vec<FuncIndex>,
        main_route_response: Response,
        route_configs: RouteConfigs,
    ) -> Self {
        Self {
            main_route,
            state,
            routes_functions,
            route_configs,
            index_array,
            main_route_response,
        }
    }
}
