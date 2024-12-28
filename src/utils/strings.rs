use core::str::FromStr;

use heapless::String;

use serde::{Deserialize, Serialize};

use crate::error::{Error, ErrorKind, Result};

macro_rules! impl_string {
    ($name:ident, $lenght:tt) => {
        impl core::fmt::Write for $name {
            fn write_str(&mut self, s: &str) -> core::result::Result<(), core::fmt::Error> {
                self.push(s).map_err(|_| core::fmt::Error)
            }

            fn write_char(&mut self, c: char) -> core::result::Result<(), core::fmt::Error> {
                self.push_char(c).map_err(|_| core::fmt::Error)
            }
        }

        impl $name {
            #[doc = concat!("Creates an empty [`", stringify!($name), "`].")]
            #[must_use]
            pub const fn empty() -> Self {
                Self(String::<$lenght>::new())
            }

            #[doc = concat!("Creates a [`", stringify!($name), "`].")]
            #[doc = ""]
            #[doc = "# Errors"]
            #[doc = concat!("If the input text is greater than ", stringify!($lenght), " bytes, an error is returned.")]
            pub fn new(text: &str) -> Result<Self> {
                Ok(Self(String::from_str(text).map_err(|()| {
                    Error::new(
                        ErrorKind::FixedText,
                        "Impossible to create a new stack string. Characters might not be UTF-8 or its length is wrong.",
                    )
                })?))
            }

            #[doc = concat!("Creates an infallible [`", stringify!($name), "`].")]
            #[doc = ""]
            #[doc = concat!("When an error occurs, an empty [`", stringify!($name), "`] is returned.")]
            #[must_use]
            pub fn infallible(text: &str) -> Self {
                Self::new(text).unwrap_or(Self::empty())
            }

            #[doc = concat!("Checks whether a [`", stringify!($name), "`] is empty.")]
            #[must_use]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            #[doc = "Returns the associated string slice."]
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            #[doc = concat!("Adds a string slice to [`", stringify!($name), "`].")]
            #[doc = ""]
            #[doc = "# Errors"]
            #[doc = concat!("If the input text is greater than ", stringify!($lenght), " bytes, an error is returned.")]
            pub fn push(&mut self, text: &str) -> Result<()> {
                self.0.push_str(text).map_err(|()| {
                    Error::new(
                        ErrorKind::FixedText,
                        "Impossible to add another stack string at the end of the current one.",
                    )
                })?;
                Ok(())
            }

            #[doc = concat!("Adds a character to [`", stringify!($name), "`].")]
            #[doc = ""]
            #[doc = "# Errors"]
            #[doc = concat!("If the input character causes the [`", stringify!($name), "`] to go beyond ", stringify!($lenght), " bytes, an error is returned.")]
            pub fn push_char(&mut self, c: char) -> Result<()> {
                self.0.push(c).map_err(|()| {
                    Error::new(
                        ErrorKind::FixedText,
                        "Impossible to add a char at the end of the stack string.",
                    )
                })?;
                Ok(())
            }
        }
    };
}

/// Minimal string data structure.
///
/// It can be used to save very short texts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniString(String<32>);

impl_string!(MiniString, 32);

/// Short string data structure.
///
/// It can be used to save short texts such as names.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortString(String<64>);

impl_string!(ShortString, 64);

/// Long string data structure.
///
/// It can be used to save long texts such as descriptions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongString(String<128>);

impl_string!(LongString, 128);
