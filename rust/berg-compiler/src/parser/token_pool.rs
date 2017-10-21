use fnv::FnvHashMap;
use indexed_vec::IndexedVec;
use indexed_vec::IndexType;

const DEFAULT_CAPACITY: usize = 1024;

#[derive(Debug)]
pub(crate) struct TokenPool<Ind: IndexType> {
    pub strings: IndexedVec<String,Ind>,
    // We use FnvHashMap because the hashing function is faster than the default
    pub indices: FnvHashMap<String,Ind>,
}

impl<Ind: IndexType> Default for TokenPool<Ind> {
    fn default() -> Self { TokenPool::with_capacity(DEFAULT_CAPACITY) }
}

impl<Ind: IndexType> TokenPool<Ind> {
    pub fn with_capacity(initial_capacity: usize) -> Self {
        let strings = IndexedVec::with_capacity(initial_capacity);
        let indices = FnvHashMap::with_capacity_and_hasher(initial_capacity, Default::default());
        TokenPool { strings, indices }
    }
    pub fn intern(&mut self, string: &str) -> Ind {
        if let Some(index) = self.indices.get(string) {
            return *index;
        }

        let index = self.strings.len();
        self.strings.push(string.to_string());
        self.indices.insert(string.to_string(), index);
        index
    }
}
