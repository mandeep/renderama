use std::collections::HashMap;


/// Add a push_into method on Vec so that we can avoid calling .into() on every push
pub trait PushInto<T> {
    fn push_into(&mut self, item: impl Into<T>);
}

impl<T> PushInto<T> for Vec<T> {
    fn push_into(&mut self, item: impl Into<T>) {
        self.push(item.into());
    }
}

/// Add an insert_into on HashMap so that we can avoid calling .into() on every insertion
pub trait InsertInto<K, V> {
    fn insert_into(&mut self, key: impl Into<K>, value: impl Into<V>);
}

impl<K: Eq + std::hash::Hash, V> InsertInto<K, V> for HashMap<K, V> {
    fn insert_into(&mut self, key: impl Into<K>, value: impl Into<V>) {
        self.insert(key.into(), value.into());
    }
}