macro_rules! map {
    (
        /// $desc:expr
        $(#[$attrs:meta])*
        pub struct $name:ident(IndexMap<$key:ty, $value:ty, DefaultHashBuilder>);
    ) => {
        ///
        $(#[$attrs])*
        pub struct $name(IndexMap<$key, $value, DefaultHashBuilder>);

        impl<'a> IntoIterator for &'a $name {
            type Item = (&'a $key, &'a $value);
            type IntoIter = Iter<'a, $key, $value>;

            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $name {
            #[doc = concat!("Creates an empty [`", stringify!($name), "`].")]
            #[must_use]
            #[inline]
            pub fn new() -> Self {
                Self(IndexMap::with_hasher(DefaultHashBuilder::default()))
            }

            #[doc = concat!("Initializes [`", stringify!($name), "`] with a specific element.")]
            #[must_use]
            #[inline]
            pub fn init(key: $key, value: $value) -> Self {
                Self::new().insert(key, value)
            }

            #[doc = concat!("Inserts a new element into [`", stringify!($name), "`].")]
            #[must_use]
            #[inline]
            pub fn insert(mut self, key: $key, value: $value) -> Self {
                self.0.insert(key, value);
                self
            }

            #[doc = concat!("Adds a new element into [`", stringify!($name), "`].")]
            #[doc = ""]
            #[doc = concat!("Unlike [`Self::insert`], this method does not return a modified [`", stringify!($name), "`].")]
            #[inline]
            pub fn add(&mut self, key: $key, value: $value) {
                self.0.insert(key, value);
            }

            #[doc = concat!("Checks if [`", stringify!($name), "`] is empty.")]
            #[must_use]
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            #[doc = concat!("Provides the number of elements in [`", stringify!($name), "`].")]
            #[must_use]
            #[inline]
            pub fn len(&self) -> usize {
                self.0.len()
            }

            #[doc = concat!("Retrieves the value associated with the specified key from [`", stringify!($name), "`].")]
            #[inline]
            pub fn get<Q>(&self, key: &Q) -> Option<&$value>
            where
                Q: ?Sized + core::hash::Hash + indexmap::Equivalent<$key>,
            {
                self.0.get(key)
            }

            #[doc = concat!("Returns an iterator over [`", stringify!($name), "`].")]
            #[doc = ""]
            #[doc = "**Iterates over the elements in the order they were inserted.**"]
            #[must_use]
            #[inline]
            pub fn iter(&self) -> Iter<'_, $key, $value> {
                self.0.iter()
            }
        }
    };
}

pub(crate) use map;
