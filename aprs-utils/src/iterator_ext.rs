pub trait IteratorExt: Sized {
    fn with_key<K, V, F>(self, mut f: F) -> impl Iterator<Item = (K, V)>
    where
        F: FnMut(&V) -> K,
        Self: Iterator<Item = V>,
    {
        self.map(move |v| {
            let k = f(&v);
            (k, v)
        })
    }

    fn with_value<K, V, F>(self, mut f: F) -> impl Iterator<Item = (K, V)>
    where
        F: FnMut(&K) -> V,
        Self: Iterator<Item = K>,
    {
        self.map(move |k| {
            let v = f(&k);
            (k, v)
        })
    }

    fn map_key<K1, V, F, K2>(self, mut f: F) -> impl Iterator<Item = (K2, V)>
    where
        F: FnMut(K1) -> K2,
        Self: Iterator<Item = (K1, V)>,
    {
        self.map(move |(k, v)| {
            let k = f(k);
            (k, v)
        })
    }

    fn map_value<K, V1, F, V2>(self, mut f: F) -> impl Iterator<Item = (K, V2)>
    where
        F: FnMut(V1) -> V2,
        Self: Iterator<Item = (K, V1)>,
    {
        self.map(move |(k, v)| {
            let v = f(v);
            (k, v)
        })
    }

    fn filter_key<K, V, F>(self, mut f: F) -> impl Iterator<Item = (K, V)>
    where
        F: FnMut(&K) -> bool,
        Self: Iterator<Item = (K, V)>,
    {
        self.filter(move |(k, _)| f(k))
    }

    fn filter_value<K, V, F>(self, mut f: F) -> impl Iterator<Item = (K, V)>
    where
        F: FnMut(&V) -> bool,
        Self: Iterator<Item = (K, V)>,
    {
        self.filter(move |(_, v)| f(v))
    }

    fn map_to_key<K, V>(self) -> impl Iterator<Item = K>
    where
        Self: Iterator<Item = (K, V)>,
    {
        self.map(|(k, _)| k)
    }

    fn map_to_value<K, V>(self) -> impl Iterator<Item = V>
    where
        Self: Iterator<Item = (K, V)>,
    {
        self.map(|(_, v)| v)
    }
}

impl<T: Iterator> IteratorExt for T {}
