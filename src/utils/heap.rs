use core::hash::Hash;

use indexmap::map::IndexMap;
use indexmap::set::IndexSet;

use serde::{Deserialize, Serialize};

/// A set of elements for internal storage.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Set<T: PartialEq + Eq + Hash>(IndexSet<T>);

/// A serializable set of elements.
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct SerialSet<T: PartialEq + Eq + Hash>(IndexSet<T>);

/// A serializable and deserializable set of elements.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct OutputSet<T: PartialEq + Eq + Hash>(IndexSet<T>);

macro_rules! from_set {
    ($for:ident) => {
        impl<T, K> From<Set<K>> for $for<T>
        where
            T: Clone + PartialEq + Eq + Hash + From<K>,
            K: Clone + PartialEq + Eq + Hash,
        {
            fn from(set: Set<K>) -> Self {
                let mut elements = Self::new();
                for other_element in set.iter() {
                    elements.0.insert(T::from(other_element.clone()));
                }
                elements
            }
        }
    };
}

macro_rules! set_implementation {
    ($impl:ident) => {
        impl<T> IntoIterator for $impl<T>
        where
            T: Clone + PartialEq + Eq + Hash,
        {
            type Item = T;
            type IntoIter = indexmap::set::IntoIter<T>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        impl<'a, T> IntoIterator for &'a $impl<T>
        where
            T: Clone + PartialEq + Eq + Hash,
        {
            type Item = &'a T;
            type IntoIter = indexmap::set::Iter<'a, T>;

            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }

        impl<T> $impl<T>
        where
            T: Clone + PartialEq + Eq +  Hash,
        {
            #[doc = concat!("Creates a [`", stringify!($impl), "`].")]
            #[must_use]
            #[inline]
            pub fn new() -> Self {
                Self(IndexSet::default())
            }

            #[doc = concat!("Initializes a [`", stringify!($impl), "`] with a determined element.")]
            #[must_use]
            #[inline]
            pub fn init(element: T) -> Self {
                Self::new().insert(element)
            }

            #[doc = concat!("Inserts an element to a [`", stringify!($impl), "`].")]
            #[must_use]
            #[inline]
            pub fn insert(mut self, element: T) -> Self {
                self.0.insert(element);
                self
            }

            #[doc = concat!("Adds an element to a [`", stringify!($impl), "`].")]
            #[inline]
            pub fn add(&mut self, element: T) {
                self.0.insert(element);
            }

            #[doc = concat!("Checks whether the [`", stringify!($impl), "`] is empty.")]
            #[must_use]
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            #[doc = concat!("Returns the [`", stringify!($impl), "`] length.")]
            #[must_use]
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
            #[must_use]
            #[inline]
            pub fn iter(&self) -> indexmap::set::Iter<'_, T> {
                self.0.iter()
            }

            #[doc = concat!("Initializes [`", stringify!($impl), "`] with a list of elements.")]
            #[inline]
            pub fn init_with_elements(input_elements: &[T]) -> Self {
                let mut elements = Self::new();
                for element in input_elements.iter() {
                    elements.add(element.clone());
                }
                elements
            }

            #[doc = concat!("Merges all elements from another [`", stringify!($impl), "`] into this one.")]
            #[inline]
            pub fn merge(&mut self, element: &Self) {
                self.0 = self.0.union(&element.0).cloned().collect();
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

/// A map of elements for internal storage.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Map<K: PartialEq + Eq + Hash, V>(IndexMap<K, V>);

/// A serializable map of elements.
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct SerialMap<K: PartialEq + Eq + Hash, V>(IndexMap<K, V>);

/// A serializable and deserializable map of elements.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct OutputMap<K: PartialEq + Eq + Hash, V>(IndexMap<K, V>);

macro_rules! from_map {
    ($for:ident) => {
        impl<K, V, K1, V1> From<Map<K1, V1>> for $for<K, V>
        where
            K: Clone + PartialEq + Eq + Hash + From<K1>,
            V: Clone + PartialEq + Eq + Hash + From<V1>,
            K1: Clone + PartialEq + Eq + Hash,
            V1: Clone + PartialEq + Eq + Hash,
        {
            fn from(map: Map<K1, V1>) -> Self {
                let mut new_map = Self::new();
                for (key, value) in map.iter() {
                    new_map
                        .0
                        .insert(K::from(key.clone()), V::from(value.clone()));
                }
                new_map
            }
        }
    };
}

macro_rules! map_implementation {
    ($impl:ident) => {
        impl<K, V> IntoIterator for $impl<K, V>
        where
            K: Clone + PartialEq + Eq + Hash,
            V: Clone
        {
            type Item = (K, V);
            type IntoIter = indexmap::map::IntoIter<K, V>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        impl<'a, K, V> IntoIterator for &'a $impl<K, V>
        where
            K: Clone + PartialEq + Eq + Hash,
            V: Clone
        {
            type Item = (&'a K, &'a V);
            type IntoIter = indexmap::map::Iter<'a, K, V>;

            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }

        impl<K, V> $impl<K, V>
        where
            K: Clone + PartialEq + Eq +  Hash,
            V: Clone
        {
            #[doc = concat!("Creates a [`", stringify!($impl), "`].")]
            #[must_use]
            #[inline]
            pub fn new() -> Self {
                Self(IndexMap::default())
            }

            #[doc = concat!("Initializes a [`", stringify!($impl), "`] with a determined element.")]
            #[must_use]
            #[inline]
            pub fn init(key: K, value: V) -> Self {
                Self::new().insert(key, value)
            }

            #[doc = concat!("Inserts an element to a [`", stringify!($impl), "`].")]
            #[must_use]
            #[inline]
            pub fn insert(mut self, key: K, value: V) -> Self {
                self.0.insert(key, value);
                self
            }

            #[doc = concat!("Adds an element to a [`", stringify!($impl), "`].")]
            #[inline]
            pub fn add(&mut self, key: K, value: V) {
                self.0.insert(key, value);
            }

            #[doc = concat!("Checks whether the [`", stringify!($impl), "`] is empty.")]
            #[must_use]
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            #[doc = concat!("Returns the [`", stringify!($impl), "`] length.")]
            #[must_use]
            #[inline]
            pub fn len(&self) -> usize {
                self.0.len()
            }

            #[doc = concat!("Checks whether the [`", stringify!($impl), "`] contains the given key.")]
            #[inline]
            pub fn contains_key(&self, key: &K) -> bool {
                self.0.contains_key(key)
            }

            #[doc = concat!("Returns an iterator over the [`", stringify!($impl), "`].")]
            #[doc = ""]
            #[doc = "**It iterates in the insertion order.**"]
            #[must_use]
            #[inline]
            pub fn iter(&self) -> indexmap::map::Iter<'_, K, V> {
                self.0.iter()
            }

            #[doc = concat!("Initializes [`", stringify!($impl), "`] with a list of `(key, value)`.")]
            #[inline]
            pub fn init_with_elements(input_elements: &[(K, V)]) -> Self {
                let mut elements = Self::new();
                for (key, value) in input_elements.iter() {
                    elements.add(key.clone(), value.clone());
                }
                elements
            }


        }
    };
}

// Map implementation.
map_implementation!(Map);

// Serial map implementation.
map_implementation!(SerialMap);

// Output map implementation.
map_implementation!(OutputMap);

// Convert from map into serial map.
from_map!(SerialMap);
// Convert from map into output map.
from_map!(OutputMap);
