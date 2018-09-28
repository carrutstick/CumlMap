/// Trait for building and querying mappings between keys and cumulative
/// values.
pub trait CumlMap {
    /// Type for the keys in this mapping.
    type Key;

    /// Type for the values in this mapping.
    type Value;

    /// Insert an entry into the mapping.
    fn insert(&mut self, Self::Key, Self::Value);

    /// Get the cumulative value up to and including
    /// the specified key.
    fn get_cuml(&self, Self::Key) -> Self::Value;

    /// Get the value at the specified key (not the cumulative value).
    fn get_single(&self, Self::Key) -> Self::Value;

    /// Get the first key at which the cumulative value equals or exceeds
    /// the specified value, if such a key exists.
    /// Note that if the result of this function is only defined if the
    /// cumulative value is non-decreasing. If you start putting negative
    /// values into your mappings, you will get strange results from this
    /// function.
    fn get_quantile(&self, Self::Value) -> Option<Self::Key>;
}
