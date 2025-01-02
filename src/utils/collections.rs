#![allow(clippy::iter_without_into_iter)]

use core::hash::Hash;

use heapless::{FnvIndexSet, IndexSetIter};

use serde::{Deserialize, Serialize};

use crate::MAXIMUM_ELEMENTS;

/// A collection of elements for internal storage.
#[derive(Debug, Clone)]
pub struct Collection<T: PartialEq + Eq + Hash>(FnvIndexSet<T, MAXIMUM_ELEMENTS>);

/// A serializable collection of elements.
#[derive(Debug, Clone, Serialize)]
pub struct SerialCollection<T: PartialEq + Eq + Hash>(FnvIndexSet<T, MAXIMUM_ELEMENTS>);

/// A serializable and deserializable collection of elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputCollection<T: PartialEq + Eq + Hash>(FnvIndexSet<T, MAXIMUM_ELEMENTS>);

macro_rules! from {
    ($from:ident, $for:ident) => {
        impl<T, K> From<&$from<K>> for $for<T>
        where
            T: Clone + PartialEq + Eq + Hash + From<K>,
            K: Clone + Copy + PartialEq + Eq + Hash,
        {
            fn from(other_collection: &$from<K>) -> Self {
                let mut elements = Self::empty();
                for other_element in other_collection.iter() {
                    let _ = elements.0.insert(T::from(*other_element));
                }
                elements
            }
        }
    };
}

macro_rules! implementation {
    ($impl:ident) => {
        impl<T> $impl<T>
        where
            T: PartialEq + Eq + Hash,
        {
            #[doc = concat!("Creates an empty [`", stringify!($impl), "`].")]
            #[must_use]
            pub const fn empty() -> Self {
                Self(FnvIndexSet::new())
            }

            #[doc = concat!("Initializes a [`", stringify!($impl), "`] with a determined element.")]
            #[inline]
            pub fn init(element: T) -> Self {
                let mut elements = Self::empty();
                elements.add(element);
                elements
            }

            #[doc = concat!("Adds an element to a [`", stringify!($impl), "`].")]
            #[inline]
            pub fn add(&mut self, element: T) {
                let _ = self.0.insert(element);
            }

            #[doc = concat!("Checks whether the [`", stringify!($impl), "`] is empty.")]
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            #[doc = concat!("Checks whether the [`", stringify!($impl), "`] contains the given element.")]
            #[inline]
            pub fn contains(&self, element: impl AsRef<T>) -> bool {
                self.0.contains(element.as_ref())
            }

            #[doc = concat!("Returns an iterator over the [`", stringify!($impl), "`].")]
            #[inline]
            pub fn iter(&self) -> IndexSetIter<'_, T> {
                self.0.iter()
            }
        }

        impl<T> $impl<T>
        where
            T: Clone + Copy + PartialEq + Eq + Hash,
        {
            #[doc = concat!("Initializes [`", stringify!($impl), "`] with a list of elements.")]
            #[inline]
            pub fn init_with_elements(input_elements: &[T]) -> Self {
                let mut elements = Self::empty();
                for element in input_elements.iter() {
                     elements.add(*element);
                }
                elements
            }
        }

        impl<T> $impl<T>
        where
            T: Clone + PartialEq + Eq + Hash,
        {
            #[doc = concat!("Merges all elements from another [`", stringify!($impl), "`] into this one.")]
            #[inline]
            pub fn merge(&mut self, element: &Self) {
                self.0 = self.0.union(&element.0).cloned().collect();
            }
        }
    };
}

// Collection implementation.
implementation!(Collection);

// Serial collection implementation.
implementation!(SerialCollection);

// Output collection implementation.
implementation!(OutputCollection);

// From macro traits.
from!(Collection, OutputCollection);
from!(Collection, SerialCollection);
from!(OutputCollection, OutputCollection);
from!(SerialCollection, SerialCollection);
