pub trait CumlMap {
	type Key;
	type Value;

	fn insert(&mut self, Self::Key, Self::Value);
	fn get_cuml(&self, Self::Key) -> Self::Value;
	fn get_single(&self, Self::Key) -> Self::Value;
	fn get_quantile(&self, Self::Value) -> Self::Key;
}

struct CumlTable<V> {
	total: V,
	tables: Vec<Vec<V>>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
