use std::cmp::Eq;
use std::collections::hash_map::Keys;
use std::collections::{HashMap, LinkedList};
use std::hash::Hash;
use std::mem::MaybeUninit;
use std::sync::Mutex;

pub type PooledCacheIndex = (usize, usize);

struct CachePool<const C: usize, T: Sized> {
    count: usize,
    data: [Option<T>; C],
}
impl<const C: usize, T: Sized> CachePool<C, T> {
    fn new(item: T) -> CachePool<C, T> {
        let mut data: [MaybeUninit<Option<T>>; C] = unsafe { MaybeUninit::uninit().assume_init() };

        for p in &mut data[..] {
            p.write(None);
        }
        let elem = &mut data[0];
        *elem = MaybeUninit::new(Some(item));

        CachePool { count: 1, data: unsafe { MaybeUninit::array_assume_init(data) } }
    }

    fn add(&mut self, item: T) -> usize {
        let index = self.count;
        self.data[index] = Some(item);
        self.count += 1;
        index
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut T> { self.data[index].as_mut() }

    fn get(&self, index: usize) -> Option<&T> { self.data[index].as_ref() }
}

pub struct PooledCache<const C: usize, K: Eq + Hash, T> {
    map: HashMap<K, PooledCacheIndex>,
    data: LinkedList<CachePool<C, T>>,
    lock: Mutex<()>,
}
impl<const C: usize, K: Eq + Hash, T> PooledCache<C, K, T> {
    pub fn new() -> PooledCache<C, K, T> {
        PooledCache { map: HashMap::new(), data: LinkedList::new(), lock: Mutex::new(()) }
    }

    pub fn get(&self, file: &K) -> Option<&T> {
        let _lock = self.lock.lock().unwrap();
        if let Some(i) = self.map.get(file) {
            let pool = self.data.iter().nth(i.0).unwrap();
            return pool.get(i.1);
        }
        None
    }

    pub fn get_mut(&mut self, file: &K) -> Option<&mut T> {
        let _lock = self.lock.lock().unwrap();
        if let Some(i) = self.map.get_mut(file) {
            let pool = self.data.iter_mut().nth(i.0).unwrap();
            return pool.get_mut(i.1);
        }
        None
    }

    pub fn keys(&self) -> Keys<'_, K, PooledCacheIndex> { self.map.keys() }

    pub fn exists(&self, file: &K) -> bool { self.map.contains_key(file) }

    pub fn insert(&mut self, file: K, data: T) {
        let _lock = self.lock.lock().unwrap();
        for (i, pool) in self.data.iter_mut().enumerate() {
            if pool.count < C {
                let index = pool.add(data);
                self.map.insert(file, (i, index));
                return;
            }
        }
        let pool = CachePool::new(data);
        self.data.push_back(pool);
        self.map.insert(file, (self.data.len() - 1, 0));
    }
}
