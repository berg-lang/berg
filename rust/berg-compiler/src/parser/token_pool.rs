use fnv::FnvHashMap;
use indexed_vec::IndexedVec;
use parser::token::TokenIndex;

const DEFAULT_CAPACITY: usize = 1024;

#[derive(Debug)]
pub(crate) struct TokenPool {
    tokens: IndexedVec<String,TokenIndex>,
    // We use FnvHashMap because the hashing function is faster than the default
    indices: FnvHashMap<String,TokenIndex>,
}

impl Default for TokenPool {
    fn default() -> Self { TokenPool::with_capacity(DEFAULT_CAPACITY) }
}

impl TokenPool {
    pub fn with_capacity(initial_capacity: usize) -> Self {
        let tokens = IndexedVec::with_capacity(initial_capacity);
        let indices = FnvHashMap::with_capacity_and_hasher(initial_capacity, Default::default());
        TokenPool { tokens, indices }
    }
    pub fn intern(&mut self, string: &str) -> TokenIndex {
        if let Some(index) = self.indices.get(string) {
            return *index;
        }

        let index = self.tokens.len();
        self.tokens.push(string.to_string());
        self.indices.insert(string.to_string(), index);
        index
    }
    pub fn string(&self, index: TokenIndex) -> &str {
        &self.tokens[index]
    }
}
