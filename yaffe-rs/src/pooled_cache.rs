
use std::collections::{HashMap, LinkedList};
use std::cmp::Eq;
use std::hash::Hash;
use std::mem::MaybeUninit;
use std::sync::Mutex;
use std::collections::linked_list::Iter;

pub type PooledCacheIndex = (usize, usize);

struct CachePool<const C: usize, T: Sized> {
    count: usize,
    data: [Option<T>; C]
}
impl<const C: usize, T: Sized> CachePool<C, T> {
    fn new(item: T) -> CachePool<C, T> {
        let mut data: [MaybeUninit<Option<T>>; C] = unsafe { MaybeUninit::uninit().assume_init() };
        
        for p in &mut data[..] {
            p.write(None);
        }
        let elem = &mut data[0];
        *elem = MaybeUninit::new(Some(item));

        CachePool { 
            count: 1, 
            data: unsafe { MaybeUninit::array_assume_init(data) },
        }
    }

    fn add(&mut self, item: T) -> usize {
        let index = self.count;
        self.data[index] = Some(item);
        self.count += 1;
        index
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut T> { 
        self.data[index].as_mut()
    }
    fn get(&self, index: usize) -> Option<&T> {
        self.data[index].as_ref()
    }
}
pub struct PooledCache<const C: usize, K: Eq + Hash, T> {
    map: HashMap<K, PooledCacheIndex>,
    data: LinkedList<CachePool<C, T>>,
    lock: Mutex<()>,
}
impl<const C: usize, K: Eq + Hash, T> PooledCache<C, K, T> {
    pub fn new() -> PooledCache<C, K, T> {
        PooledCache { 
            map: HashMap::new(),
            data: LinkedList::new(),
            lock: Mutex::new(()),
        }
    }

    pub fn get_mut(&mut self, file: &K) -> Option<&mut T> {
        let _lock = self.lock.lock().unwrap();
        if let Some(i) = self.map.get_mut(file) {
            let pool = self.data.iter_mut().skip(i.0).next().unwrap();
            return pool.get_mut(i.1);
        }
        None
    }

    pub fn get(&self, file: &K) -> Option<&T> {
        let _lock = self.lock.lock().unwrap();
        if let Some(i) = self.map.get(file) {
            let pool = self.data.iter().skip(i.0).next().unwrap();
            return pool.get(i.1);
        }
        None
    }

    pub fn contains(&self, file: &K) -> bool {
        self.map.get(file).is_some()
    }

    pub fn remove_at(&mut self, index: PooledCacheIndex) {
        let _lock = self.lock.lock().unwrap();
        let pool = self.data.iter_mut().skip(index.0).next().unwrap();
        pool.data[index.1] = None;
    }

    pub fn iter(&self) -> PooledCacheIter<C, T> {
        PooledCacheIter { pools: self.data.iter(), curr: None, pool_index: 0, index: 0 }
    }

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

pub struct PooledCacheIter<'a, const C: usize, T> {
    pools: Iter<'a, CachePool<C, T>>,
    curr: Option<&'a CachePool<C, T>>,
    pool_index: usize,
    index: usize,
}
impl<'a, const C: usize, T> Iterator for PooledCacheIter<'a, C, T> {
    type Item = (PooledCacheIndex, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        if self.curr.is_none() || self.index >= self.curr.unwrap().count {
            self.curr = self.pools.next();
            self.index = 0;
            self.pool_index += 1;
        }
        match self.curr {
            None => None,
            Some(p) => {
                let index = (self.pool_index - 1, self.index);
                let data = p.get(self.index);
                if let None = data {
                    return None; //TODO this isnt quite right
                } else {
                    return Some((index, data.unwrap()));
                }
            },
        }
    }
}