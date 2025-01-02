use serde::{Deserialize, Serialize};

use crate::collections::OutputCollection;

/// All possible hazards for a device task.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Hazard {
    /// The execution may release toxic gases.
    AirPoisoning,
    /// The execution may cause oxygen deficiency by gaseous substances.
    Asphyxia,
    /// The execution authorises the app to record and save a video with audio on persistent storage.
    AudioVideoRecordAndStore,
    /// The execution authorises the app to obtain a video stream with audio.
    AudioVideoStream,
    /// The execution enables a device that consumes electricity.
    ElectricEnergyConsumption,
    /// The execution may cause an explosion.
    Explosion,
    /// The execution may cause fire.
    FireHazard,
    /// The execution enables a device that consumes gas.
    GasConsumption,
    /// The execution authorises the app to get and save information about the app's energy impact on the device the app runs on.
    LogEnergyConsumption,
    /// The execution authorises the app to get and save information about the app's duration of use.
    LogUsageTime,
    /// The execution authorises the app to use payment information and make a periodic payment.
    PaySubscriptionFee,
    /// The execution may cause an interruption in the supply of electricity.
    PowerOutage,
    /// The execution may lead to exposure to high voltages.
    PowerSurge,
    /// The execution authorises the app to get and save user inputs.
    RecordIssuedCommands,
    /// The execution authorises the app to get and save information about the user's preferences.
    RecordUserPreferences,
    /// The execution authorises the app to use payment information and make a payment transaction.
    SpendMoney,
    /// The execution may lead to rotten food.
    SpoiledFood,
    /// The execution authorises the app to read the display output and take screenshots of it.
    TakeDeviceScreenshots,
    /// The execution authorises the app to use a camera and take photos.
    TakePictures,
    /// The execution disables a protection mechanism and unauthorised individuals may physically enter home.
    UnauthorisedPhysicalAccess,
    /// The execution enables a device that consumes water.
    WaterConsumption,
    /// The execution allows water usage which may lead to flood.
    WaterFlooding,
}

impl core::convert::AsRef<Self> for Hazard {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl core::fmt::Debug for Hazard {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.name().fmt(f)
    }
}

impl core::fmt::Display for Hazard {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.name().fmt(f)
    }
}

impl Hazard {
    /// Returns an [`Hazard`] name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::AirPoisoning => "Air Poisoning",
            Self::Asphyxia => "Asphyxia",
            Self::AudioVideoRecordAndStore => "Audio Video Record And Store",
            Self::AudioVideoStream => "Audio Video Stream",
            Self::ElectricEnergyConsumption => "Electric Energy Consumption",
            Self::Explosion => "Explosion",
            Self::FireHazard => "Fire Hazard",
            Self::GasConsumption => "Gas Consumption",
            Self::LogEnergyConsumption => "Log Energy Consumption",
            Self::LogUsageTime => "Log Usage Time",
            Self::PaySubscriptionFee => "Pay Subscription Fee",
            Self::PowerOutage => "Power Outage",
            Self::PowerSurge => "Power Surge",
            Self::RecordIssuedCommands => "Record Issued Commands",
            Self::RecordUserPreferences => "Record User Preferences",
            Self::SpendMoney => "Spend Money",
            Self::SpoiledFood => "Spoiled Food",
            Self::TakeDeviceScreenshots => "Take Device Screenshots",
            Self::TakePictures => "Take Pictures",
            Self::UnauthorisedPhysicalAccess => "Unauthorised Physical Access",
            Self::WaterConsumption => "Water Consumption",
            Self::WaterFlooding => "Water Flooding",
        }
    }

    /// Returns an [`Hazard`] description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::AirPoisoning => "The execution may release toxic gases.",
            Self::Asphyxia => "The execution may cause oxygen deficiency by gaseous substances.",
            Self::AudioVideoRecordAndStore => "The execution authorises the app to record and save a video with audio on persistent storage.",
            Self::AudioVideoStream => "The execution authorises the app to obtain a video stream with audio.",
            Self::ElectricEnergyConsumption => "The execution enables a device that consumes electricity.",
            Self::Explosion => "The execution may cause an explosion.",
            Self::FireHazard => "The execution may cause fire.",
            Self::GasConsumption => "The execution enables a device that consumes gas.",
            Self::LogEnergyConsumption => "The execution authorises the app to get and save information about the app's energy impact on the device the app runs on.",
            Self::LogUsageTime => "The execution authorises the app to get and save information about the app's duration of use.",
            Self::PaySubscriptionFee => "The execution authorises the app to use payment information and make a periodic payment.",
            Self::PowerOutage => "The execution may cause an interruption in the supply of electricity.",
            Self::PowerSurge => "The execution may lead to exposure to high voltages.",
            Self::RecordIssuedCommands => "The execution authorises the app to get and save user inputs.",
            Self::RecordUserPreferences => "The execution authorises the app to get and save information about the user's preferences.",
            Self::SpendMoney => "The execution authorises the app to use payment information and make a payment transaction.",
            Self::SpoiledFood => "The execution may lead to rotten food.",
            Self::TakeDeviceScreenshots => "The execution authorises the app to read the display output and take screenshots of it.",
            Self::TakePictures => "The execution authorises the app to use a camera and take photos.",
            Self::UnauthorisedPhysicalAccess => "The execution disables a protection mechanism and unauthorised individuals may physically enter home.",
            Self::WaterConsumption => "The execution enables a device that consumes water.",
            Self::WaterFlooding => "The execution allows water usage which may lead to flood.",
        }
    }

    /// Returns the [`Category`] associated with an [`Hazard`].
    ///
    /// An hazard **must** be associated with **only** one category.
    #[must_use]
    pub const fn category(&self) -> Category {
        match self {
            Self::AirPoisoning
            | Self::Asphyxia
            | Self::Explosion
            | Self::FireHazard
            | Self::PowerOutage
            | Self::PowerSurge
            | Self::SpoiledFood
            | Self::UnauthorisedPhysicalAccess
            | Self::WaterFlooding => Category::Safety,
            Self::AudioVideoRecordAndStore
            | Self::AudioVideoStream
            | Self::LogEnergyConsumption
            | Self::RecordIssuedCommands
            | Self::RecordUserPreferences
            | Self::TakeDeviceScreenshots
            | Self::TakePictures
            | Self::LogUsageTime => Category::Privacy,
            Self::ElectricEnergyConsumption
            | Self::GasConsumption
            | Self::PaySubscriptionFee
            | Self::SpendMoney
            | Self::WaterConsumption => Category::Financial,
        }
    }

    /// Returns the identifier associated with an [`Hazard`].
    #[must_use]
    pub const fn id(&self) -> u16 {
        match self {
            Self::AirPoisoning => 0,
            Self::Asphyxia => 1,
            Self::AudioVideoRecordAndStore => 2,
            Self::AudioVideoStream => 3,
            Self::ElectricEnergyConsumption => 4,
            Self::Explosion => 5,
            Self::FireHazard => 6,
            Self::GasConsumption => 7,
            Self::LogEnergyConsumption => 8,
            Self::LogUsageTime => 9,
            Self::PaySubscriptionFee => 10,
            Self::PowerOutage => 11,
            Self::PowerSurge => 12,
            Self::RecordIssuedCommands => 13,
            Self::RecordUserPreferences => 14,
            Self::SpendMoney => 15,
            Self::SpoiledFood => 16,
            Self::TakeDeviceScreenshots => 17,
            Self::TakePictures => 18,
            Self::UnauthorisedPhysicalAccess => 19,
            Self::WaterConsumption => 20,
            Self::WaterFlooding => 21,
        }
    }

    /// Returns an [`Hazard`] from an integer identifier.
    ///
    /// The value is [`None`] whenever the identifier does not exist or
    /// it is not correct.
    #[must_use]
    pub const fn from_id(id: u16) -> Option<Self> {
        match id {
            0 => Some(Self::AirPoisoning),
            1 => Some(Self::Asphyxia),
            2 => Some(Self::AudioVideoRecordAndStore),
            3 => Some(Self::AudioVideoStream),
            4 => Some(Self::ElectricEnergyConsumption),
            5 => Some(Self::Explosion),
            6 => Some(Self::FireHazard),
            7 => Some(Self::GasConsumption),
            8 => Some(Self::LogEnergyConsumption),
            9 => Some(Self::LogUsageTime),
            10 => Some(Self::PaySubscriptionFee),
            11 => Some(Self::PowerOutage),
            12 => Some(Self::PowerSurge),
            13 => Some(Self::RecordIssuedCommands),
            14 => Some(Self::RecordUserPreferences),
            15 => Some(Self::SpendMoney),
            16 => Some(Self::SpoiledFood),
            17 => Some(Self::TakeDeviceScreenshots),
            18 => Some(Self::TakePictures),
            19 => Some(Self::UnauthorisedPhysicalAccess),
            20 => Some(Self::WaterConsumption),
            21 => Some(Self::WaterFlooding),
            _ => None,
        }
    }
}

/// A collection of [`Hazard`]s.
pub type Hazards = OutputCollection<Hazard>;

/// Hazard categories.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    /// Category which includes all the financial-related hazards.
    Financial,
    /// Category which includes all the privacy-related hazards.
    Privacy,
    /// Category which includes all the safety-related hazards.
    Safety,
}

impl core::fmt::Debug for Category {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.name().fmt(f)
    }
}

impl core::fmt::Display for Category {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.name().fmt(f)
    }
}

impl Category {
    /// Returns a [`Category`] name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Financial => "Financial",
            Self::Privacy => "Privacy",
            Self::Safety => "Safety",
        }
    }

    /// Returns a [`Category`] description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Financial => "Category which includes all financial-related hazards.",
            Self::Privacy => "Category which includes all privacy-related hazards.",
            Self::Safety => "Category which includes all safety-related hazards.",
        }
    }

    /// Returns all [`Hazard`]s associated with a [`Category`].
    #[must_use]
    pub const fn hazards(&self) -> &[Hazard] {
        match self {
            Self::Financial => &[
                Hazard::ElectricEnergyConsumption,
                Hazard::GasConsumption,
                Hazard::PaySubscriptionFee,
                Hazard::SpendMoney,
                Hazard::WaterConsumption,
            ],
            Self::Privacy => &[
                Hazard::AudioVideoRecordAndStore,
                Hazard::AudioVideoStream,
                Hazard::LogEnergyConsumption,
                Hazard::LogUsageTime,
                Hazard::RecordIssuedCommands,
                Hazard::RecordUserPreferences,
                Hazard::TakeDeviceScreenshots,
                Hazard::TakePictures,
            ],
            Self::Safety => &[
                Hazard::AirPoisoning,
                Hazard::Asphyxia,
                Hazard::Explosion,
                Hazard::FireHazard,
                Hazard::PowerOutage,
                Hazard::PowerSurge,
                Hazard::SpoiledFood,
                Hazard::UnauthorisedPhysicalAccess,
                Hazard::WaterFlooding,
            ],
        }
    }
}

#[cfg(test)]
mod tests {

    use serde_json::json;

    use super::*;

    #[test]
    fn test_hazard() {
        assert_eq!(
            serde_json::to_value(Hazard::AirPoisoning).unwrap(),
            json!("AirPoisoning")
        );
    }
}
