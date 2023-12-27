use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use crate::beecell::BeeCell;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Debug)]
pub struct Database {
    files: HashMap<String, Vec<u8>>,
    file_links: HashMap<String, HashSet<String>>,
}

#[derive(Default, Debug, Clone)]
pub struct File {
    pub filename: String,
    pub beeid: String,
    pub data: Vec<u8>,
    pub previous_beecell_id: String,
    pub next_beecell_id: String,
}

impl File {
    pub fn new(filename: String, beeid: String, data: Vec<u8>, previous_beecell_id: String, next_beecell_id: String) -> Self {
        File {
            filename,
            beeid,
            data,
            previous_beecell_id,
            next_beecell_id,
        }
    }
}

impl Hash for File {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.filename.hash(state);
        self.data.hash(state);
        self.previous_beecell_id.hash(state);
        self.next_beecell_id.hash(state);
    }
}

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename &&
            self.data == other.data &&
            self.previous_beecell_id == other.previous_beecell_id &&
            self.next_beecell_id == other.next_beecell_id
    }
}

impl Eq for File {}

impl Database {
    pub fn new() -> Self {
        Database {
            files: HashMap::new(),
            file_links: HashMap::new(),
        }
    }

    pub fn store_file(&mut self, filename: String, data: Vec<u8>) {
        self.files.insert(filename.clone(), data);
    
        // Store file links for slicing
        let file_links = self.file_links.entry(filename.clone()).or_insert(HashSet::new());
        // Add links to previous and next cells
        // (assuming beecell IDs are strings)
        let previous_id = Self::previous_beecell_id(&filename);
        let next_id = Self::next_beecell_id(&filename);
        file_links.insert(previous_id);
        file_links.insert(next_id);
    }

    pub fn retrieve_file(&self, filename: &str) -> Option<&Vec<u8>> {
        self.files.get(filename)
    }

    pub fn get_file_links(&self, filename: &str) -> Option<&HashSet<String>> {
        self.file_links.get(filename)
    }


    pub fn slice_file(&self, filename: &str, start: usize, end: usize) -> Option<Vec<u8>> {
        let file = self.retrieve_file(filename)?;
        let file_links = self.get_file_links(filename)?;

        let mut sliced_file = Vec::new();

        // Add previous cells
        for link in file_links.iter().take(start) {
            let cell = self.retrieve_file(link)?;
            sliced_file.extend(cell);
        }

        // Add current cell
        let cell = self.retrieve_file(filename)?;
        sliced_file.extend(cell);

        // Add next cells
        for link in file_links.iter().skip(end) {
            let cell = self.retrieve_file(link)?;
            sliced_file.extend(cell);
        }

        Some(sliced_file)
    }

    pub fn previous_beecell_id(beecell_id: &str) -> String {
        let mut hasher = DefaultHasher::new();
        beecell_id.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn next_beecell_id(beecell_id: &str) -> String {
        let mut hasher = DefaultHasher::new();
        beecell_id.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn beecell_id(data: &[u8]) -> String {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn beecell_id_from_previous(previous_beecell_id: &str, data: &[u8]) -> String {
        let mut hasher = DefaultHasher::new();
        previous_beecell_id.hash(&mut hasher);
        data.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn beecell_id_from_next(next_beecell_id: &str, data: &[u8]) -> String {
        let mut hasher = DefaultHasher::new();
        next_beecell_id.hash(&mut hasher);
        data.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn beecell_id_from_previous_and_next(previous_beecell_id: &str, next_beecell_id: &str, data: &[u8]) -> String {
        let mut hasher = DefaultHasher::new();
        previous_beecell_id.hash(&mut hasher);
        next_beecell_id.hash(&mut hasher);
        data.hash(&mut hasher);
        hasher.finish().to_string()
    }
}