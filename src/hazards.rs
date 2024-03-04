use heapless::{FnvIndexSet, IndexSetIter};

use serde::{Deserialize, Serialize};

use crate::MAXIMUM_ELEMENTS;

/// Hazard data.
#[derive(Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct HazardData<'a> {
    /// Identifier.
    pub id: u16,
    /// Name.
    pub name: &'a str,
    /// Description.
    pub description: &'a str,
    /// Category.
    pub category: CategoryData<'a>,
}

impl<'a> HazardData<'a> {
    fn new(id: u16, name: &'a str, description: &'a str, category: CategoryData<'a>) -> Self {
        Self {
            id,
            name,
            description,
            category,
        }
    }
}

impl<'a> core::cmp::PartialEq for HazardData<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// A collection of [`HazardData`]s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HazardsData<'a>(#[serde(borrow)] FnvIndexSet<HazardData<'a>, MAXIMUM_ELEMENTS>);

impl<'a> HazardsData<'a> {
    /// Initializes a new [`HazardsData`] collection.
    pub fn init() -> Self {
        Self(FnvIndexSet::new())
    }

    /// Initializes a new [`HazardsData`] collection from [`Hazards`].
    pub fn from_hazards(hazards: &Hazards) -> Self {
        let mut hazards_data = Self::init();
        for hazard in hazards.iter() {
            let hazard_data = HazardData::new(
                hazard.id(),
                hazard.name(),
                hazard.description(),
                CategoryData::new(*hazard),
            );

            let _ = hazards_data.0.insert(hazard_data);
        }
        hazards_data
    }

    /// Adds a new [`HazardData`] to the [`HazardsData`] collection.
    pub fn add(&mut self, hazard_data: HazardData<'a>) {
        let _ = self.0.insert(hazard_data);
    }

    /// Whether the [`HazardsData`] collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Checks whether a [`HazardData`] is contained into [`HazardsData`].
    pub fn contains(&self, hazard_data: &HazardData) -> bool {
        self.0.contains(hazard_data)
    }

    /// Returns an iterator over [`HazardData`]s.
    pub fn iter(&self) -> IndexSetIter<'_, HazardData> {
        self.0.iter()
    }

    /// Merges the collection with another [`HazardsData`].
    pub fn merge(&mut self, hazards_data: &Self) {
        self.0 = self.0.union(&hazards_data.0).cloned().collect();
    }
}

/// All possible hazards for a device task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hazard {
    /// A fire hazard can destroy a smart home.
    FireHazard,
    /// The device consumes lots of energy.
    EnergyConsumption,
}

impl core::fmt::Display for Hazard {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.name().fmt(f)
    }
}

impl Hazard {
    /// Returns an [`Hazard`] name.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::FireHazard => "Fire Hazard",
            Self::EnergyConsumption => "Energy Consumption",
        }
    }

    /// Returns an [`Hazard`] description.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::FireHazard => "A fire hazard can destroy a smart home.",
            Self::EnergyConsumption => "The device consumes lots of energy.",
        }
    }

    /// Returns the [`Category`] associated with an [`Hazard`].
    ///
    /// An hazard **must** be associated with **only** one category.
    pub const fn category(&self) -> Category {
        match self {
            Self::FireHazard => Category::Safety,
            Self::EnergyConsumption => Category::Financial,
        }
    }

    /// Returns the identifier associated with an [`Hazard`].
    pub const fn id(&self) -> u16 {
        match self {
            Self::FireHazard => 0,
            Self::EnergyConsumption => 1,
        }
    }

    /// Returns an [`Hazard`] from an integer identifier.
    ///
    /// The value is [`None`] whenever the identifier does not exist or
    /// it is not correct.
    pub const fn from_id(id: u16) -> Option<Self> {
        match id {
            0 => Some(Self::FireHazard),
            1 => Some(Self::EnergyConsumption),
            _ => None,
        }
    }
}

/// A collection of [`Hazard`]s.
#[derive(Debug, Clone)]
pub struct Hazards(FnvIndexSet<Hazard, MAXIMUM_ELEMENTS>);

impl Hazards {
    /// Initializes a new [`Hazards`] collection.
    pub fn init() -> Self {
        Self(FnvIndexSet::new())
    }

    /// Adds a new [`Hazard`] to the [`Hazards`] collection.
    pub fn add(&mut self, hazard: Hazard) {
        let _ = self.0.insert(hazard);
    }

    /// Whether the [`Hazards`] collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Checks whether an [`Hazard`] is contained into
    /// the [`Hazards`] collection.
    pub fn contains(&self, hazard: Hazard) -> bool {
        self.0.contains(&hazard)
    }

    /// Returns an iterator over [`Hazard`]s.
    pub fn iter(&self) -> IndexSetIter<'_, Hazard> {
        self.0.iter()
    }
}

/// Hazard category data.
#[derive(Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct CategoryData<'a> {
    /// Name.
    pub name: &'a str,
    /// Description.
    pub description: &'a str,
}

impl<'a> CategoryData<'a> {
    fn new(hazard: Hazard) -> Self {
        Self {
            name: hazard.category().name(),
            description: hazard.category().description(),
        }
    }
}

impl<'a> core::cmp::PartialEq for CategoryData<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// Hazard categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum Category {
    /// All the safety-related hazards.
    Safety,
    /// All the financial-related hazards.
    Financial,
}

impl core::fmt::Display for Category {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.name().fmt(f)
    }
}

impl Category {
    /// Returns a [`Category`] name.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Safety => "Safety",
            Self::Financial => "Financial",
        }
    }

    /// Returns a [`Category`] description.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Safety => "All the safety-related hazards.",
            Self::Financial => "All the financial-related hazards.",
        }
    }

    /// Returns all [`Hazard`]s associated with a [`Category`].
    pub const fn hazards(&self) -> &[Hazard] {
        match self {
            Self::Safety => &[Hazard::FireHazard],
            Self::Financial => &[Hazard::EnergyConsumption],
        }
    }
}
