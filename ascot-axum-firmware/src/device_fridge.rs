use alloc::sync::Arc;

use ascot_axum::{
    axum::{extract::Path, Extension, Json},
    device::ResponseError,
};
use async_lock::Mutex;

#[derive(Clone)]
pub(super) struct Fridge {
    pub(super) temperature: f64,
}

impl Default for Fridge {
    fn default() -> Self {
        Self::init(4.0)
    }
}

impl Fridge {
    pub(super) const fn init(temperature: f64) -> Self {
        Self {
            temperature,
        }
    }

    pub(super) fn increase_temperature(&mut self, increment: f64) {
        self.temperature += increment;
        println!("Fridge temperature increased to {}", self.temperature);
    }

    pub(super) fn decrease_temperature(&mut self, decrement: f64) {
        self.temperature -= decrement;
        println!("Fridge temperature decreased to {}", self.temperature);
    }
}

#[derive(Clone, Default)]
pub(crate) struct FridgeState(Arc<Mutex<Fridge>>);

impl FridgeState {
    pub(crate) fn new(fridge: Fridge) -> Self {
        Self(Arc::new(Mutex::new(fridge)))
    }
}

impl core::ops::Deref for FridgeState {
    type Target = Arc<Mutex<Fridge>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for FridgeState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub(crate) async fn increase_temperature(
    Path(increment): Path<f64>,
    Extension(state): Extension<FridgeState>,
) -> Result<Json<f64>, ResponseError> {
    let mut fridge = state.lock().await;
    fridge.increase_temperature(increment);

    Ok(Json(fridge.temperature))
}

pub(crate) async fn decrease_temperature(
    Path(decrement): Path<f64>,
    Extension(state): Extension<FridgeState>,
) -> Result<Json<f64>, ResponseError> {
    let mut fridge = state.lock().await;
    fridge.decrease_temperature(decrement);

    Ok(Json(fridge.temperature))
}