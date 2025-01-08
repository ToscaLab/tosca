pub mod error;
pub mod info;
pub mod ok;
pub mod serial;

use ascot_library::hazards::{Hazard, Hazards};
use ascot_library::input::Inputs;
use ascot_library::route::{RestKind, Route};

use axum::{handler::Handler, Router};

use tracing::info;

#[rustfmt::skip]
macro_rules! all_the_tuples {
    ($name:ident) => {
        $name!([], );
        $name!([], T1);
        $name!([T1], T2);
        $name!([T1, T2], T3);
        $name!([T1, T2, T3], T4);
        $name!([T1, T2, T3, T4], T5);
        $name!([T1, T2, T3, T4, T5], T6);
        $name!([T1, T2, T3, T4, T5, T6], T7);
        $name!([T1, T2, T3, T4, T5, T6, T7], T8);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8], T9);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9], T10);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10], T11);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11], T12);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12], T13);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13], T14);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14], T15);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15], T16);
    };
}

pub(super) use all_the_tuples;

fn build_get_route(route: &str, inputs: &Inputs) -> String {
    let mut route = String::from(route);
    for input in inputs.iter() {
        route.push_str(&format!("/{{{}}}", input.name()));
    }
    info!("Build GET route: {}", route);
    route
}

#[derive(Debug)]
pub struct DeviceAction {
    // Router.
    pub(crate) router: Router,
    // Route
    pub(crate) route: Route,
}

impl DeviceAction {
    /// Checks whether an action does not define the given [`Hazard`].
    #[inline]
    pub fn miss_hazard(&self, hazard: Hazard) -> bool {
        !self.route.hazards().contains(hazard)
    }

    /// Checks whether an action does not define the given [`Hazard`]s.
    #[inline]
    pub fn miss_hazards(&self, hazards: &'static [Hazard]) -> bool {
        !hazards
            .iter()
            .all(|hazard| self.route.hazards().contains(*hazard))
    }

    /// Returns the [`Hazards`] collection associated with an action.
    #[inline]
    pub fn hazards(&self) -> &Hazards {
        &self.route.hazards()
    }

    #[inline]
    pub(crate) fn stateless<H, T>(route: Route, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        Self {
            router: Self::create_router(route.route(), route.kind(), route.inputs(), handler, ()),
            route,
        }
    }

    #[inline]
    pub(crate) fn stateful<H, T, S>(route: Route, handler: H, state: S) -> Self
    where
        H: Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static,
    {
        Self {
            router: Self::create_router(
                route.route(),
                route.kind(),
                route.inputs(),
                handler,
                state,
            ),
            route,
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            router: Router::new(),
            route: Route::get(""),
        }
    }

    #[inline]
    fn create_router<H, T, S>(
        route: &str,
        route_kind: RestKind,
        inputs: &Inputs,
        handler: H,
        state: S,
    ) -> Router
    where
        H: Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static,
    {
        // Create the GET route for the axum architecture.
        let route = if let RestKind::Get = route_kind {
            &build_get_route(route, inputs)
        } else {
            route
        };

        Router::new()
            .route(
                route,
                match route_kind {
                    RestKind::Get => axum::routing::get(handler),
                    RestKind::Put => axum::routing::put(handler),
                    RestKind::Post => axum::routing::post(handler),
                    RestKind::Delete => axum::routing::delete(handler),
                },
            )
            .with_state(state)
    }
}

/// A mandatory [`DeviceAction`].
pub struct MandatoryAction<const SET: bool> {
    pub(crate) device_action: DeviceAction,
}

impl MandatoryAction<false> {
    #[inline(always)]
    pub(crate) fn empty() -> Self {
        Self {
            device_action: DeviceAction::empty(),
        }
    }

    pub(super) const fn new(device_action: DeviceAction) -> Self {
        Self { device_action }
    }
}

impl MandatoryAction<true> {
    /// Returns a [`DeviceAction`] reference.
    pub const fn action_as_ref(&self) -> &DeviceAction {
        &self.device_action
    }

    pub(crate) const fn init(device_action: DeviceAction) -> Self {
        Self { device_action }
    }
}

#[cfg(test)]
mod tests {
    use ascot_library::input::Input;

    use super::{build_get_route, Route};

    #[test]
    fn test_build_get_route() {
        let route = Route::get("/route")
            .description("A GET route.")
            .with_inputs([
                Input::rangeu64_with_default("rangeu64", (0, 20, 1), 5),
                Input::rangef64("rangef64", (0., 20., 0.1)),
            ]);

        assert_eq!(
            &build_get_route(route.route(), route.inputs()),
            "/route/{rangeu64}/{rangef64}"
        );
    }
}
