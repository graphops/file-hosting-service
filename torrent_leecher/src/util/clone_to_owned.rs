use std::collections::HashMap;

pub trait CloneToOwned {
    type Target;

    fn clone_to_owned(&self) -> Self::Target;
}

impl<T> CloneToOwned for Option<T>
where
    T: CloneToOwned,
{
    type Target = Option<<T as CloneToOwned>::Target>;

    fn clone_to_owned(&self) -> Self::Target {
        self.as_ref().map(|i| i.clone_to_owned())
    }
}

impl<T> CloneToOwned for Vec<T>
where
    T: CloneToOwned,
{
    type Target = Vec<<T as CloneToOwned>::Target>;

    fn clone_to_owned(&self) -> Self::Target {
        self.iter().map(|i| i.clone_to_owned()).collect()
    }
}

impl CloneToOwned for u8 {
    type Target = u8;

    fn clone_to_owned(&self) -> Self::Target {
        *self
    }
}

impl CloneToOwned for u32 {
    type Target = u32;

    fn clone_to_owned(&self) -> Self::Target {
        *self
    }
}

impl<K, V> CloneToOwned for HashMap<K, V>
where
    K: CloneToOwned,
    <K as CloneToOwned>::Target: std::hash::Hash + Eq,
    V: CloneToOwned,
{
    type Target = HashMap<<K as CloneToOwned>::Target, <V as CloneToOwned>::Target>;

    fn clone_to_owned(&self) -> Self::Target {
        let mut result = HashMap::with_capacity(self.capacity());
        for (k, v) in self {
            result.insert(k.clone_to_owned(), v.clone_to_owned());
        }
        result
    }
}
