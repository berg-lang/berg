use compiler::source_data::ByteIndex;
use std::ops::Index;
use fnv::FnvHashMap;
use indexed_vec::IndexedVec;
use indexed_vec::IndexType;
use std::str;

const DEFAULT_CAPACITY: usize = 1024;

#[derive(Debug)]
pub struct InternPool<Ind: IndexType> {
    pub strings: StringPool<Ind>,
    // We use FnvHashMap because the hashing function is faster than the default
    pub indices: FnvHashMap<String,Ind>,
}

#[derive(Debug)]
pub struct StringPool<Ind: IndexType>(IndexedVec<String,Ind>);

pub trait Pool<Ind: IndexType> {
    fn add(&mut self, string: &str) -> Ind;
    unsafe fn add_utf8_unchecked(&mut self, buffer: &[u8], start: ByteIndex, end: ByteIndex) -> Ind {
        let string = str::from_utf8_unchecked(&buffer[usize::from(start)..usize::from(end)]);
        self.add(string)
    }
}

impl<Ind: IndexType> Default for InternPool<Ind> {
    fn default() -> Self { InternPool::with_capacity(DEFAULT_CAPACITY) }
}

impl<Ind: IndexType> Default for StringPool<Ind> {
    fn default() -> Self { StringPool::with_capacity(DEFAULT_CAPACITY) }
}

impl<Ind: IndexType> InternPool<Ind> {
    pub fn with_capacity(initial_capacity: usize) -> Self {
        let strings = StringPool::with_capacity(initial_capacity);
        let indices = FnvHashMap::with_capacity_and_hasher(initial_capacity, Default::default());
        InternPool { strings, indices }
    }
    pub fn len(&self) -> Ind {
        self.strings.len()
    }
}

impl<Ind: IndexType> Index<Ind> for InternPool<Ind> {
    type Output = str;
    fn index(&self, index: Ind) -> &str { &self.strings[index] }
}

impl<Ind: IndexType> StringPool<Ind> {
    pub fn len(&self) -> Ind { self.0.len() }
    pub fn with_capacity(initial_capacity: usize) -> Self { StringPool(IndexedVec::with_capacity(initial_capacity)) }
}

impl<Ind: IndexType> Index<Ind> for StringPool<Ind> {
    type Output = str;
    fn index(&self, index: Ind) -> &str { &self.0[index] }
}

impl<Ind: IndexType> Pool<Ind> for InternPool<Ind> {
    fn add(&mut self, string: &str) -> Ind {
        if let Some(index) = self.indices.get(string) {
            return *index;
        }

        let index = self.strings.add(string);
        self.indices.insert(string.to_string(), index);
        index
    }
}

impl<Ind: IndexType> Pool<Ind> for StringPool<Ind> {
    fn add(&mut self, string: &str) -> Ind {
        let index = self.len();
        self.0.push(string.to_string());
        index
    }
}
