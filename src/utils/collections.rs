use core::hash::Hash;

use heapless::{FnvIndexSet, IndexSetIter};

use serde::{Deserialize, Serialize};

use crate::MAXIMUM_ELEMENTS;

/// A collection of elements for internal storage.
#[derive(Debug, Clone)]
pub struct Collection<T: PartialEq + Eq + Hash>(FnvIndexSet<T, MAXIMUM_ELEMENTS>);

/// A collection of elements which need to be serialized and deserialized on
/// a support.
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
            #[inline(always)]
            pub fn add(&mut self, element: T) {
                let _ = self.0.insert(element);
            }

            #[doc = concat!("Checks whether the [`", stringify!($impl), "`] is empty.")]
            #[inline(always)]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            #[doc = concat!("Returns an iterator over the [`", stringify!($impl), "`].")]
            #[inline]
            pub fn iter(&self) -> IndexSetIter<'_, T> {
                self.0.iter()
            }
        }
    };
}

macro_rules! init_elements {
    ($impl:ident) => {
        impl<T> $impl<T>
        where
            T: Clone + Copy + PartialEq + Eq + Hash,
        {
            #[doc = concat!("Initializes [`", stringify!($impl), "`] with a list of elements.")]
            #[inline]
            pub fn init_with_elements(input_elements: &[T]) -> Self {
                let mut elements = Self::empty();
                input_elements.iter().for_each(|element| {
                    elements.add(*element);
                });
                elements
            }
        }
    };
}

macro_rules! merge_implementation {
    ($impl:ident) => {
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

// From macro traits.
from!(Collection, OutputCollection);
from!(OutputCollection, OutputCollection);

// Implementation macro.
implementation!(OutputCollection);

// Init elements macro.
init_elements!(OutputCollection);

// Merge macro.
merge_implementation!(OutputCollection);

impl<T> OutputCollection<T>
where
    T: Clone + PartialEq + Eq + Hash,
{
    /// Checks whether an [`OutputCollection`] contains the given element.
    #[inline(always)]
    pub fn contains(&self, element: &T) -> bool {
        self.0.contains(element)
    }
}

// Implementation macro.
implementation!(Collection);

// Init elements macro.
init_elements!(Collection);

// Merge macro.
merge_implementation!(Collection);

impl<T> Collection<T>
where
    T: Clone + Copy + PartialEq + Eq + Hash,
{
    /// Checks whether a [`Collection`] contains the given element.
    #[inline(always)]
    pub fn contains(&self, element: T) -> bool {
        self.0.contains(&element)
    }
}
