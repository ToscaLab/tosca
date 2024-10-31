use ascot_library::device::{DeviceData, DeviceKind, DeviceSerializer};
use ascot_library::route::{RouteConfigs, RoutesHazards};

use axum::Router;

use tracing::info;

use crate::actions::DeviceAction;

// Default main route for a device.
const DEFAULT_MAIN_ROUTE: &str = "/device";

/// A general device.
#[derive(Debug)]
pub struct Device<S = ()>
where
    S: Clone + Send + Sync + 'static,
{
    // Main device route.
    pub(crate) main_route: &'static str,
    // Router.
    pub(crate) router: Router,
    // State.
    pub(crate) state: S,
    // Kind.
    kind: DeviceKind,
    // All device routes and their hazards.
    routes_hazards: RoutesHazards,
}

impl Default for Device<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Device<()> {
    /// Creates an unknown [`Device`] without a state.
    #[inline(always)]
    pub fn new() -> Self {
        Self::with_state(())
    }
}

impl<S> DeviceSerializer for Device<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn serialize_data(&self) -> DeviceData {
        let mut route_configs = RouteConfigs::empty();
        for route_hazards in self.routes_hazards.iter() {
            info!(
                "Device route: [{}, \"{}{}\"]",
                route_hazards.route.kind(),
                self.main_route,
                route_hazards.route.route()
            );

            route_configs.add(route_hazards.serialize_data());
        }

        DeviceData {
            kind: self.kind,
            main_route: self.main_route,
            route_configs,
        }
    }
}

impl<S> Device<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates an unknown [`Device`] with state.
    #[inline]
    pub fn with_state(state: S) -> Self {
        Self::init(DeviceKind::Unknown, state)
    }

    /// Sets a new main route.
    pub const fn main_route(mut self, main_route: &'static str) -> Self {
        self.main_route = main_route;
        self
    }

    /// Adds an action to the [`Device`].
    #[inline]
    pub fn add_action(self, device_action: impl FnOnce(S) -> DeviceAction) -> Self {
        let device_action = device_action(self.state.clone());
        self.add_device_action(device_action)
    }

    #[inline]
    pub(crate) fn init(kind: DeviceKind, state: S) -> Self {
        Self {
            main_route: DEFAULT_MAIN_ROUTE,
            router: Router::new(),
            kind,
            routes_hazards: RoutesHazards::empty(),
            state,
        }
    }

    #[inline]
    pub(crate) fn add_device_action(mut self, device_action: DeviceAction) -> Self {
        self.router = self.router.merge(device_action.router);
        self.routes_hazards.add(device_action.route_hazards);
        self
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::sync::Arc;

    use core::ops::{Deref, DerefMut};

    use ascot_library::device::DeviceInfo;
    use ascot_library::energy::Energy;
    use ascot_library::route::{Route, RouteHazards};

    use async_lock::Mutex;

    use axum::extract::{FromRef, Json, State};

    use serde::{Deserialize, Serialize};

    use crate::actions::info::{info_stateful, InfoPayload};
    use crate::actions::serial::{serial_stateful, serial_stateless, SerialPayload};
    use crate::actions::ActionError;

    use super::Device;

    #[derive(Clone)]
    struct DeviceState<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        state: S,
        info: DeviceInfoState,
    }

    impl DeviceState<()> {
        fn empty() -> Self {
            Self::new(())
        }
    }

    impl<S> DeviceState<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        fn new(state: S) -> Self {
            Self {
                state,
                info: DeviceInfoState::new(DeviceInfo::empty()),
            }
        }

        fn add_device_info(mut self, info: DeviceInfo) -> Self {
            self.info = DeviceInfoState::new(info);
            self
        }
    }

    #[derive(Clone)]
    struct SubState {}

    impl FromRef<DeviceState<SubState>> for SubState {
        fn from_ref(device_state: &DeviceState<SubState>) -> SubState {
            device_state.state.clone()
        }
    }

    #[derive(Clone)]
    struct DeviceInfoState {
        info: Arc<Mutex<DeviceInfo>>,
    }

    impl DeviceInfoState {
        fn new(info: DeviceInfo) -> Self {
            Self {
                info: Arc::new(Mutex::new(info)),
            }
        }
    }

    impl Deref for DeviceInfoState {
        type Target = Arc<Mutex<DeviceInfo>>;

        fn deref(&self) -> &Self::Target {
            &self.info
        }
    }

    impl DerefMut for DeviceInfoState {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.info
        }
    }

    impl<S> FromRef<DeviceState<S>> for DeviceInfoState
    where
        S: Clone + Send + Sync + 'static,
    {
        fn from_ref(device_state: &DeviceState<S>) -> DeviceInfoState {
            device_state.info.clone()
        }
    }

    #[derive(Deserialize)]
    struct Inputs {
        parameter: f64,
    }

    #[derive(Serialize)]
    struct DeviceResponse {
        parameter: f64,
    }

    async fn serial_action_with_state(
        State(_state): State<DeviceState<()>>,
        Json(inputs): Json<Inputs>,
    ) -> Result<SerialPayload<DeviceResponse>, ActionError> {
        Ok(SerialPayload::new(DeviceResponse {
            parameter: inputs.parameter,
        }))
    }

    async fn serial_action_with_substate1(
        State(_state): State<SubState>,
        Json(inputs): Json<Inputs>,
    ) -> Result<SerialPayload<DeviceResponse>, ActionError> {
        Ok(SerialPayload::new(DeviceResponse {
            parameter: inputs.parameter,
        }))
    }

    #[derive(Serialize)]
    struct DeviceInfoResponse {
        parameter: f64,
        device_info: DeviceInfo,
    }

    // This method is just a demonstration of this library flexibility,
    // but we do not recommend it because a DeviceInfo inside a SerialPayload
    // could be ignored as response by a receiver.
    async fn serial_action_with_substate2(
        State(state): State<DeviceInfoState>,
        Json(inputs): Json<Inputs>,
    ) -> Result<SerialPayload<DeviceInfoResponse>, ActionError> {
        // Retrieve internal state.
        let mut device_info = state.lock().await;

        // Change state.
        device_info.energy = Energy::empty();

        Ok(SerialPayload::new(DeviceInfoResponse {
            parameter: inputs.parameter,
            device_info: device_info.clone(),
        }))
    }

    async fn info_action_with_substate3(
        State(state): State<DeviceInfoState>,
    ) -> Result<InfoPayload, ActionError> {
        // Retrieve internal state.
        let mut device_info = state.lock().await;

        // Change state.
        device_info.energy = Energy::empty();

        Ok(InfoPayload::new(device_info.clone()))
    }

    async fn serial_action_without_state(
        Json(inputs): Json<Inputs>,
    ) -> Result<SerialPayload<DeviceResponse>, ActionError> {
        Ok(SerialPayload::new(DeviceResponse {
            parameter: inputs.parameter,
        }))
    }

    struct Routes {
        with_state_route: RouteHazards,
        without_state_route: RouteHazards,
    }

    #[inline]
    fn create_routes() -> Routes {
        Routes {
            with_state_route: RouteHazards::no_hazards(
                Route::put("/state-action").description("Run action with state."),
            ),
            without_state_route: RouteHazards::no_hazards(
                Route::post("/no-state-route").description("Run action without state."),
            ),
        }
    }

    #[test]
    fn with_state() {
        let routes = create_routes();

        let state = DeviceState::empty().add_device_info(DeviceInfo::empty());

        Device::with_state(state)
            .add_action(serial_stateful(
                routes.with_state_route,
                serial_action_with_state,
            ))
            .add_action(serial_stateless(
                routes.without_state_route,
                serial_action_without_state,
            ));

        assert!(true);
    }

    #[test]
    fn with_substates() {
        let routes = create_routes();

        let state = DeviceState::new(SubState {}).add_device_info(DeviceInfo::empty());

        Device::with_state(state)
            .add_action(serial_stateful(
                routes.with_state_route,
                serial_action_with_substate1,
            ))
            .add_action(serial_stateful(
                RouteHazards::no_hazards(
                    Route::put("/substate-action")
                        .description("Run a serial action with a substate."),
                ),
                serial_action_with_substate2,
            ))
            .add_action(info_stateful(
                RouteHazards::no_hazards(
                    Route::put("/substate-info")
                        .description("Run an informative action with a substate."),
                ),
                info_action_with_substate3,
            ))
            .add_action(serial_stateless(
                routes.without_state_route,
                serial_action_without_state,
            ));

        assert!(true);
    }

    #[test]
    fn without_state() {
        let routes = create_routes();

        Device::new().add_action(serial_stateless(
            routes.without_state_route,
            serial_action_without_state,
        ));

        assert!(true);
    }
}
