use std::collections::HashMap;

use crate::key_gen::{Key, KeyGen};

pub trait HasId {
  fn id(&self) -> &str;
}

#[derive(Debug)]
pub struct KeyStore<T> {
  key_gen: KeyGen<T>,
  data: HashMap<Key<T>, T>,
}

impl<T> KeyStore<T> {
  pub fn new() -> Self {
    Self {
      key_gen: KeyGen::new(),
      data: HashMap::new(),
    }
  }

  pub fn from<I>(iter: I) -> Self
  where
    I: Iterator<Item = T>,
  {
    iter.fold(Self::new(), |mut store, item| {
      store.add(item);
      store
    })
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }

  pub fn keys(&self) -> impl Iterator<Item = &Key<T>> {
    self.data.keys()
  }

  pub fn contains_key(&self, key: Key<T>) -> bool {
    self.data.contains_key(&key)
  }

  pub fn get(&self, key: Key<T>) -> Option<&T> {
    self.data.get(&key)
  }

  pub fn get_mut(&mut self, key: Key<T>) -> Option<&mut T> {
    self.data.get_mut(&key)
  }

  pub fn add(&mut self, item: T) -> Key<T> {
    let key = self.key_gen.next();
    self.data.insert(key, item);
    key
  }

  pub fn first_key(&self) -> Option<Key<T>> {
    self.data.keys().min().cloned()
  }

  pub fn iter(&self) -> impl Iterator<Item = (Key<T>, &T)> {
    self.data.iter().map(|(key, value)| (*key, value))
  }

  pub fn remove(&mut self, key: Key<T>) -> Option<T> {
    self.data.remove(&key)
  }
}

impl<T> PartialEq for KeyStore<T>
where
  T: PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    self.data.eq(&other.data)
  }
}

#[derive(Debug)]
pub struct KeyStoreWithId<T> {
  key_store: KeyStore<T>,
  keys_by_id: HashMap<String, Key<T>>,
}

impl<T: HasId> KeyStoreWithId<T> {
  pub fn new() -> Self {
    Self {
      key_store: KeyStore::new(),
      keys_by_id: HashMap::new(),
    }
  }

  pub fn from<I>(iter: I) -> Self
  where
    I: Iterator<Item = T>,
  {
    iter.fold(Self::new(), |mut store, item| {
      store.add(item);
      store
    })
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.key_store.len()
  }

  #[inline]
  pub fn keys(&self) -> impl Iterator<Item = &Key<T>> {
    self.key_store.keys()
  }

  #[inline]
  pub fn contains_key(&self, key: Key<T>) -> bool {
    self.key_store.contains_key(key)
  }

  pub fn contains_id<'a, S: Into<&'a str>>(&self, id: S) -> bool {
    self.keys_by_id.contains_key(id.into())
  }

  pub fn key_from_id<'a, S: Into<&'a str>>(&self, id: S) -> Option<Key<T>> {
    self.keys_by_id.get(id.into()).cloned()
  }

  #[inline]
  pub fn get(&self, key: Key<T>) -> Option<&T> {
    self.key_store.get(key)
  }

  #[inline]
  pub fn get_mut(&mut self, key: Key<T>) -> Option<&mut T> {
    self.key_store.get_mut(key)
  }

  pub fn add(&mut self, item: T) -> Key<T> {
    let id = item.id().to_string();
    let key = self.key_store.add(item);
    self.keys_by_id.insert(id, key);
    key
  }

  #[inline]
  pub fn first_key(&self) -> Option<Key<T>> {
    self.key_store.keys().min().cloned()
  }

  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = (Key<T>, &T)> {
    self.key_store.iter().map(|(key, value)| (key, value))
  }

  pub fn remove(&mut self, key: Key<T>) -> Option<T> {
    let maybe_item = self.key_store.remove(key);
    if let Some(item) = maybe_item.as_ref() {
      self.keys_by_id.remove(item.id());
    }
    maybe_item
  }
}
