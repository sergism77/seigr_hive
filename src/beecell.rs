use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub struct BeeCell {
    data: Vec<u8>,
    sealed: bool,
}

impl Hash for BeeCell {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
        self.sealed.hash(state);
    }
}

impl BeeCell {
    pub fn new() -> Self {
        BeeCell {
            data: Vec::new(),
            sealed: false,
        }
    }

    pub fn add_data(&mut self, data: Vec<u8>) {
        if !self.sealed {
            self.data.extend(data);
        } else {
            panic!("Cannot add data to a sealed bee cell");
        }
    }

    pub fn seal(&mut self) {
        self.sealed = true;
    }

    pub fn is_sealed(&self) -> bool {
        self.sealed
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn beecell_id(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn previous_beecell_id(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn next_beecell_id(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish().to_string()
    }

}
