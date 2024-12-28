use core::str::FromStr;

use heapless::String;

use serde::{Deserialize, Serialize};

use crate::error::{Error, ErrorKind, Result};

// Maximum string length for a mini string.
const MINI_STRING_LENGHT: usize = 32;

// Maximum string length for a short string.
const SHORT_STRING_LENGHT: usize = 64;

// Maximum string length for a long string.
const LONG_STRING_LENGHT: usize = 128;

fn create_string<const N: usize>(text: &str) -> Result<String<N>> {
    String::from_str(text).map_err(|()| {
        Error::new(
            ErrorKind::FixedText,
            "Impossible to create a new stack string. Characters might not be UTF-8 or its length is wrong.",
        )
    })
}

fn push_string<const N: usize>(string: &mut String<N>, text: &str) -> Result<()> {
    string.push_str(text).map_err(|()| {
        Error::new(
            ErrorKind::FixedText,
            "Impossible to add another stack string at the end of the current one.",
        )
    })
}

fn push_char<const N: usize>(string: &mut String<N>, c: char) -> Result<()> {
    string.push(c).map_err(|()| {
        Error::new(
            ErrorKind::FixedText,
            "Impossible to add a char at the end of the stack string.",
        )
    })
}

macro_rules! impl_write_trait {
    ($name:ident) => {
        impl core::fmt::Write for $name {
            fn write_str(&mut self, s: &str) -> core::result::Result<(), core::fmt::Error> {
                self.push(s).map_err(|_| core::fmt::Error)
            }

            fn write_char(&mut self, c: char) -> core::result::Result<(), core::fmt::Error> {
                self.push_char(c).map_err(|_| core::fmt::Error)
            }
        }
    };
}

/// Minimal string data structure.
///
/// It can be used to save very short texts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniString(String<MINI_STRING_LENGHT>);

impl_write_trait!(MiniString);

impl MiniString {
    /// Creates an empty [`MiniString`].
    #[must_use]
    pub const fn empty() -> Self {
        Self(String::<MINI_STRING_LENGHT>::new())
    }

    /// Creates a [`MiniString`].
    ///
    /// # Errors
    /// If the input text is greater than 32 bytes, an error is returned.
    pub fn new(text: &str) -> Result<Self> {
        Ok(Self(create_string::<MINI_STRING_LENGHT>(text)?))
    }

    /// Creates an infallible [`MiniString`].
    ///
    /// When an error occurs, an empty [`MiniString`] is returned.
    #[must_use]
    pub fn infallible(text: &str) -> Self {
        Self::new(text).unwrap_or(Self::empty())
    }

    /// Checks whether a [`MiniString`] is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the associated string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Adds a string slice to [`MiniString`].
    ///
    /// # Errors
    /// If the input text is greater than 32 bytes, an error is returned.
    pub fn push(&mut self, text: &str) -> Result<()> {
        push_string::<MINI_STRING_LENGHT>(&mut self.0, text)
    }

    /// Adds a character to [`MiniString`].
    ///
    /// # Errors
    /// If the input character causes the [`MiniString`] to go beyond 32 bytes,
    /// an error is returned.
    pub fn push_char(&mut self, c: char) -> Result<()> {
        push_char::<MINI_STRING_LENGHT>(&mut self.0, c)
    }
}

/// Short string data structure.
///
/// It can be used to save short texts such as names.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortString(String<SHORT_STRING_LENGHT>);

impl_write_trait!(ShortString);

impl ShortString {
    /// Creates an empty [`ShortString`].
    #[must_use]
    pub const fn empty() -> Self {
        Self(String::<SHORT_STRING_LENGHT>::new())
    }

    /// Creates a [`ShortString`].
    ///
    /// # Errors
    /// If the input text is greater than 64 bytes, an error is returned.
    pub fn new(text: &str) -> Result<Self> {
        Ok(Self(create_string::<SHORT_STRING_LENGHT>(text)?))
    }

    /// Creates an infallible [`ShortString`].
    ///
    /// When an error occurs, an empty [`ShortString`] is returned.
    #[must_use]
    pub fn infallible(text: &str) -> Self {
        Self::new(text).unwrap_or(Self::empty())
    }

    /// Checks whether a [`MiniString`] is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the associated string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Adds a string slice to [`ShortString`].
    ///
    /// # Errors
    /// If the input text is greater than 64 bytes, an error is returned.
    pub fn push(&mut self, text: &str) -> Result<()> {
        push_string::<SHORT_STRING_LENGHT>(&mut self.0, text)
    }

    /// Adds a character to [`ShortString`].
    ///
    /// # Errors
    /// If the input character causes the [`ShortString`] to go beyond 64 bytes,
    /// an error is returned.
    pub fn push_char(&mut self, c: char) -> Result<()> {
        push_char::<SHORT_STRING_LENGHT>(&mut self.0, c)
    }
}

/// Long string data structure.
///
/// It can be used to save long texts such as descriptions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongString(String<LONG_STRING_LENGHT>);

impl_write_trait!(LongString);

impl LongString {
    /// Creates an empty [`LongString`].
    #[must_use]
    pub const fn empty() -> Self {
        Self(String::<LONG_STRING_LENGHT>::new())
    }

    /// Creates a [`LongString`].
    ///
    /// # Errors
    /// If the input text is greater than 128 bytes, an error is returned.
    pub fn new(text: &str) -> Result<Self> {
        Ok(Self(create_string::<LONG_STRING_LENGHT>(text)?))
    }

    /// Creates an infallible [`LongString`].
    ///
    /// When an error occurs, an empty [`LongString`] is returned.
    #[must_use]
    pub fn infallible(text: &str) -> Self {
        Self::new(text).unwrap_or(Self::empty())
    }

    /// Checks whether a [`MiniString`] is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the associated string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Adds a string slice to [`LongString`].
    ///
    /// # Errors
    /// If the input text is greater than 128 bytes, an error is returned.
    pub fn push(&mut self, text: &str) -> Result<()> {
        push_string::<LONG_STRING_LENGHT>(&mut self.0, text)
    }

    /// Adds a character to [`LongString`].
    ///
    /// # Errors
    /// If the input character causes the [`LongString`] to go beyond 128 bytes,
    /// an error is returned.
    pub fn push_char(&mut self, c: char) -> Result<()> {
        push_char::<LONG_STRING_LENGHT>(&mut self.0, c)
    }
}
