use alloc::format;

use tosca::parameters::{ParameterValue, ParametersValues as ToscaParametersValues};

use crate::response::ErrorResponse;
use crate::server::invalid_data;

/// A container for storing route parameters.
pub struct ParametersValues(pub(crate) ToscaParametersValues<'static>);

impl ParametersValues {
    /// Retrieves the [`bool`] value associated with the given parameter name.
    ///
    /// # Errors
    ///
    /// An [`ErrorResponse`] is returned in the following cases:
    ///
    /// - When the given parameter is not found
    /// - When the given parameter has an incorrect type
    #[inline]
    pub fn bool(&self, name: &'static str) -> Result<bool, ErrorResponse> {
        self.insert(name, |value| match value {
            ParameterValue::Bool(v) => Ok(*v),
            _ => Err(invalid_data(&format!("`{name}` is not a bool kind"))),
        })
    }

    /// Retrieves the [`u8`] value associated with the given parameter name.
    ///
    /// # Errors
    ///
    /// An [`ErrorResponse`] is returned in the following cases:
    ///
    /// - When the given parameter is not found
    /// - When the given parameter has an incorrect type
    #[inline]
    pub fn u8(&self, name: &'static str) -> Result<u8, ErrorResponse> {
        self.insert(name, |value| match value {
            ParameterValue::U8(v) => Ok(*v),
            _ => Err(invalid_data(&format!("`{name}` is not a u8 kind"))),
        })
    }

    /// Retrieves the [`u16`] value associated with the given parameter name.
    ///
    /// # Errors
    ///
    /// An [`ErrorResponse`] is returned in the following cases:
    ///
    /// - When the given parameter is not found
    /// - When the given parameter has an incorrect type
    #[inline]
    pub fn u16(&self, name: &'static str) -> Result<u16, ErrorResponse> {
        self.insert(name, |value| match value {
            ParameterValue::U16(v) => Ok(*v),
            _ => Err(invalid_data(&format!("`{name}` is not a u16 kind"))),
        })
    }

    /// Retrieves the [`u32`] value associated with the given parameter name.
    ///
    /// # Errors
    ///
    /// An [`ErrorResponse`] is returned in the following cases:
    ///
    /// - When the given parameter is not found
    /// - When the given parameter has an incorrect type
    #[inline]
    pub fn u32(&self, name: &'static str) -> Result<u32, ErrorResponse> {
        self.insert(name, |value| match value {
            ParameterValue::U32(v) => Ok(*v),
            _ => Err(invalid_data(&format!("`{name}` is not a u32 kind"))),
        })
    }

    /// Retrieves the [`u64`] value associated with the given parameter name.
    ///
    /// # Errors
    ///
    /// An [`ErrorResponse`] is returned in the following cases:
    ///
    /// - When the given parameter is not found
    /// - When the given parameter has an incorrect type
    #[inline]
    pub fn u64(&self, name: &'static str) -> Result<u64, ErrorResponse> {
        self.insert(name, |value| match value {
            ParameterValue::U64(v) => Ok(*v),
            _ => Err(invalid_data(&format!("`{name}` is not a u64 kind"))),
        })
    }

    /// Retrieves the [`f32`] value associated with the given parameter name.
    ///
    /// # Errors
    ///
    /// An [`ErrorResponse`] is returned in the following cases:
    ///
    /// - When the given parameter is not found
    /// - When the given parameter has an incorrect type
    #[inline]
    pub fn f32(&self, name: &'static str) -> Result<f32, ErrorResponse> {
        self.insert(name, |value| match value {
            ParameterValue::F32(v) => Ok(*v),
            _ => Err(invalid_data(&format!("`{name}` is not a f32 kind"))),
        })
    }

    /// Retrieves the [`f64`] value associated with the given parameter name.
    ///
    /// # Errors
    ///
    /// An [`ErrorResponse`] is returned in the following cases:
    ///
    /// - When the given parameter is not found
    /// - When the given parameter has an incorrect type
    #[inline]
    pub fn f64(&self, name: &'static str) -> Result<f64, ErrorResponse> {
        self.insert(name, |value| match value {
            ParameterValue::F64(v) => Ok(*v),
            _ => Err(invalid_data(&format!("`{name}` is not a f64 kind"))),
        })
    }

    /// Retrieves the characters sequence associated with
    /// the given parameter name.
    ///
    /// # Errors
    ///
    /// An [`ErrorResponse`] is returned in the following cases:
    ///
    /// - When the given parameter is not found
    /// - When the given parameter has an incorrect type
    #[inline]
    pub fn chars_sequence<'a>(&'a self, name: &'static str) -> Result<&'a str, ErrorResponse> {
        self.insert(name, |value| match value {
            ParameterValue::CharsSequence(v) => Ok(v.as_ref()),
            _ => Err(invalid_data(&format!("`{name}` is not a chars sequence"))),
        })
    }

    #[inline]
    fn insert<'a, T, F>(&'a self, name: &'static str, func: F) -> Result<T, ErrorResponse>
    where
        F: FnOnce(&'a ParameterValue) -> Result<T, ErrorResponse>,
    {
        let value = self
            .0
            .get(name)
            .ok_or(invalid_data(&format!("`{name}` not found.")))?;

        func(value)
    }
}
