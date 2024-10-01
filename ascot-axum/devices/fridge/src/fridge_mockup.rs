#[derive(Clone)]
pub(crate) struct FridgeMockup {
    pub(crate) temperature: f64,
}

impl Default for FridgeMockup {
    fn default() -> Self {
        Self::init(2.0)
    }
}

impl FridgeMockup {
    pub(crate) const fn init(temperature: f64) -> Self {
        Self { temperature }
    }

    pub(crate) fn increase_temperature(&mut self, increment: f64) {
        self.temperature += increment;
        println!("Run increase temperature with increment={increment}");
    }

    pub(crate) fn decrease_temperature(&mut self, decrement: f64) {
        self.temperature -= decrement;
        println!("Run decrease temperature with decrement={decrement}");
    }
}
