pub trait CumlMap {
    type Key;
    type Value;

    fn insert(&mut self, Self::Key, Self::Value);
    fn get_cuml(&self, Self::Key) -> Self::Value;
    fn get_single(&self, Self::Key) -> Self::Value;
    fn get_quantile(&self, Self::Value) -> Option<Self::Key>;
}
