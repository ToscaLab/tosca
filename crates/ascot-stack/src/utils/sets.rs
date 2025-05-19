use core::hash::Hash;

use heapless::{FnvIndexSet, IndexSetIter};

use serde::{Deserialize, Serialize};

/// A set of elements for internal storage.
#[derive(Debug, Clone)]
pub struct Set<V: Eq + Hash, const N: usize>(FnvIndexSet<V, N>);

/// A serializable set of elements.
#[derive(Debug, Clone, Serialize)]
pub struct SerialSet<V: Eq + Hash, const N: usize>(FnvIndexSet<V, N>);

/// A serializable and deserializable set of elements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputSet<V: Eq + Hash, const N: usize>(FnvIndexSet<V, N>);

macro_rules! from_set {
    ($for:ident) => {
        impl<V, V1, const N: usize> From<Set<V1, N>> for $for<V, N>
        where
            V: Clone + Copy + Eq + Hash + From<V1>,
            V1: Clone + Copy + Eq + Hash,
        {
            fn from(set: Set<V1, N>) -> Self {
                let mut new_set = Self::new();
                for element in set.iter() {
                    let _ = new_set.0.insert(V::from(*element));
                }
                new_set
            }
        }
    };
}

macro_rules! set_implementation {
    ($impl:ident $(,$trait:ident)?) => {
        impl<'a, V, const N: usize> IntoIterator for &'a $impl<V, N>
        where
            V: Clone + Copy + Eq + Hash,
        {
            type Item = &'a V;
            type IntoIter = IndexSetIter<'a, V>;

            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }

        impl<V, const N: usize> Default for $impl<V, N>
        where
            V: Clone + Copy + Eq + Hash,
        {
            fn default() -> Self {
                Self::new()
            }
        }

        impl<V, const N: usize> $impl<V, N>
        where
            V: Clone + Copy + Eq + Hash,
        {
            #[doc = concat!("Creates a [`", stringify!($impl), "`].")]
            #[must_use]
            pub const fn new() -> Self {
                Self(FnvIndexSet::new())
            }

            #[doc = concat!("Initializes a [`", stringify!($impl), "`] with a determined element.")]
            #[must_use]
            #[inline]
            pub fn init(element: V) -> Self {
                let mut elements = Self::new();
                elements.add(element);
                elements
            }

            #[doc = concat!("Inserts an element to a [`", stringify!($impl), "`].")]
            #[must_use]
            #[inline]
            pub fn insert(mut self, element: V) -> Self {
                let _ = self.0.insert(element);
                self
            }

            #[doc = concat!("Adds an element to a [`", stringify!($impl), "`].")]
            #[inline]
            pub fn add(&mut self, element: V) {
                let _ = self.0.insert(element);
            }

            #[doc = concat!("Checks whether the [`", stringify!($impl), "`] is empty.")]
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            #[doc = concat!("Returns the [`", stringify!($impl), "`] length.")]
            #[inline]
            pub fn len(&self) -> usize {
                self.0.len()
            }

            #[doc = concat!("Checks whether the [`", stringify!($impl), "`] contains the given element.")]
            #[inline]
            pub fn contains(&self, element: &V) -> bool {
                self.0.contains(element)
            }

            #[doc = concat!("Returns an iterator over the [`", stringify!($impl), "`].")]
            #[doc = ""]
            #[doc = "**It iterates in the insertion order.**"]
            #[inline]
            pub fn iter(&self) -> IndexSetIter<'_, V> {
                self.0.iter()
            }

            #[doc = concat!("Initializes [`", stringify!($impl), "`] with a list of elements.")]
            #[inline]
            pub fn init_with_elements(input_elements: &[V]) -> Self {
                let mut elements = Self::new();
                for element in input_elements.iter() {
                    elements.add(*element);
                }
                elements
            }

            #[doc = concat!("Merges all elements from another [`", stringify!($impl), "`] into this one.")]
            #[inline]
            pub fn merge(&mut self, element: &Self) {
                self.0 = self.0.union(&element.0).copied().collect();
            }
        }
    };
}

// Set implementation.
set_implementation!(Set);

// Serial set implementation.
set_implementation!(SerialSet);

// Output set implementation.
set_implementation!(OutputSet);

// Convert from a set into a serial collection.
from_set!(SerialSet);
// Convert from a set into an output set.
from_set!(OutputSet);
