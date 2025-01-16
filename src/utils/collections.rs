use core::hash::Hash;

use heapless::{FnvIndexSet, IndexSetIter};

use serde::{Deserialize, Serialize};

use crate::MAXIMUM_ELEMENTS;

/// A collection of elements for internal storage.
#[derive(Debug, PartialEq, Clone)]
pub struct Collection<T: PartialEq + Eq + Hash>(FnvIndexSet<T, MAXIMUM_ELEMENTS>);

/// A serializable collection of elements.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct SerialCollection<T: PartialEq + Eq + Hash>(FnvIndexSet<T, MAXIMUM_ELEMENTS>);

/// A serializable and deserializable collection of elements.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct OutputCollection<T: PartialEq + Eq + Hash>(FnvIndexSet<T, MAXIMUM_ELEMENTS>);

macro_rules! from_collection {
    ($for:ident $(,$trait:ident)?) => {
        impl<T, K> From<Collection<K>> for $for<T>
        where
            T: Clone + $($trait +)?  PartialEq + Eq + Hash + From<K>,
            K: Clone + $($trait +)? PartialEq + Eq + Hash,
        {
            fn from(collection: Collection<K>) -> Self {
                let mut elements = Self::empty();
                for other_element in collection.iter() {
                    #[cfg(feature = "alloc")]
                    let _ = elements.0.insert(T::from(other_element.clone()));
                    #[cfg(not(feature = "alloc"))]
                    let _ = elements.0.insert(T::from(*other_element));
                }
                elements
            }
        }
    };
}

macro_rules! implementation {
    ($impl:ident $(,$trait:ident)?) => {
        impl<'a, T> IntoIterator for &'a $impl<T>
        where
            T: Clone + $($trait +)? PartialEq + Eq + Hash,
        {
            type Item = &'a T;
            type IntoIter = heapless::IndexSetIter<'a, T>;

            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }

        impl<T> $impl<T>
        where
            T: Clone + $($trait +)? PartialEq + Eq + Hash,
        {
            #[doc = concat!("Creates an empty [`", stringify!($impl), "`].")]
            #[must_use]
            pub const fn empty() -> Self {
                Self(FnvIndexSet::new())
            }

            #[doc = concat!("Initializes a [`", stringify!($impl), "`] with a determined element.")]
            #[must_use]
            #[inline]
            pub fn init(element: T) -> Self {
                let mut elements = Self::empty();
                elements.add(element);
                elements
            }

            #[doc = concat!("Inserts an element to a [`", stringify!($impl), "`].")]
            #[must_use]
            #[inline]
            pub fn insert(mut self, element: T) -> Self {
                let _ = self.0.insert(element);
                self
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

            #[doc = concat!("Returns the [`", stringify!($impl), "`] length.")]
            #[inline]
            pub fn len(&self) -> usize {
                self.0.len()
            }

            #[doc = concat!("Checks whether the [`", stringify!($impl), "`] contains the given element.")]
            #[inline]
            pub fn contains(&self, element: impl AsRef<T>) -> bool {
                self.0.contains(element.as_ref())
            }

            #[doc = concat!("Returns an iterator over the [`", stringify!($impl), "`].")]
            #[doc = ""]
            #[doc = "**It iterates in the insertion order.**"]
            #[inline]
            pub fn iter(&self) -> IndexSetIter<'_, T> {
                self.0.iter()
            }

            #[doc = concat!("Initializes [`", stringify!($impl), "`] with a list of elements.")]
            #[inline]
            pub fn init_with_elements(input_elements: &[T]) -> Self {
                let mut elements = Self::empty();
                for element in input_elements.iter() {
                    #[cfg(feature = "alloc")]
                    elements.add(element.clone());
                    #[cfg(not(feature = "alloc"))]
                    elements.add(*element);
                }
                elements
            }

            #[doc = concat!("Merges all elements from another [`", stringify!($impl), "`] into this one.")]
            #[inline]
            pub fn merge(&mut self, element: &Self) {
                #[cfg(feature = "alloc")]
                let a = self.0.union(&element.0).cloned().collect();
                #[cfg(not(feature = "alloc"))]
                let a = self.0.union(&element.0).copied().collect();

                self.0 = a;
            }

        }
    };
}

#[cfg(feature = "alloc")]
mod alloc {
    use super::{Collection, FnvIndexSet, Hash, IndexSetIter, OutputCollection, SerialCollection};

    // Collection implementation.
    implementation!(Collection);

    // Output collection implementation.
    implementation!(OutputCollection);

    // Serial collection implementation.
    implementation!(SerialCollection);

    // Convert from collection into serial collection.
    from_collection!(SerialCollection);
    // Convert from collection into output collection.
    from_collection!(OutputCollection);
}

#[cfg(not(feature = "alloc"))]
mod alloc {
    use super::{Collection, FnvIndexSet, Hash, IndexSetIter, OutputCollection, SerialCollection};

    // Collection implementation.
    implementation!(Collection, Copy);

    // Output collection implementation.
    implementation!(OutputCollection, Copy);

    // Serial collection implementation.
    implementation!(SerialCollection, Copy);

    // Convert from collection into serial collection.
    from_collection!(SerialCollection, Copy);

    // Convert from collection into output collection.
    from_collection!(OutputCollection, Copy);
}
