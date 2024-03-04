use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Hazard data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HazardData {
    /// Identifier.
    pub id: u16,
    /// Name.
    pub name: Cow<'static, str>,
    /// Description.
    pub description: Cow<'static, str>,
    /// Category.
    pub category: CategoryData,
}

impl core::cmp::PartialEq for HazardData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl core::cmp::Eq for HazardData {}

impl core::hash::Hash for HazardData {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
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

    /// Serializes [`Hazard`] data.
    pub fn serialize_data(&self) -> HazardData {
        HazardData {
            id: self.id(),
            name: self.name().into(),
            description: self.description().into(),
            category: self.category().serialize_data(),
        }
    }
}

/// Hazard category data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryData {
    /// Name.
    pub name: Cow<'static, str>,
    /// Description.
    pub description: Cow<'static, str>,
}

impl core::cmp::PartialEq for CategoryData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl core::cmp::Eq for CategoryData {}

impl core::hash::Hash for CategoryData {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
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
    /// Returns an [`Category`] name.
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

    /// Serializes [`Category`] data.
    pub fn serialize_data(&self) -> CategoryData {
        CategoryData {
            name: self.name().into(),
            description: self.description().into(),
        }
    }
}
