
use bjdatasets::treedstructure::Block;
use bjdatasets::treedstructure::IndexTrees;
use bjdatasets::treedstructure::TermRole;
use wasm_bindgen::prelude::*;


// We write one impl block per function to make it easier to debug (if we have
// a compile error, the compiler says there is an error in the whole impl block,
// splitting in different impl blocks help identifying the bugged function)


#[wasm_bindgen(js_name="TreedDataset")]
pub struct ExportedTrees {
    trees: IndexTrees
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { trees: IndexTrees::new() }
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    /// Returns the number of quads
    pub fn size(&self) -> usize {
        self.trees.base_tree.1.len()
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    /// Adds the given quad
    pub fn add(&mut self, s: u32, p: u32, o: u32, g: u32) {
        self.trees.insert_by_index([s, p, o, g]);
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    /// Removes the given quad
    pub fn remove(&mut self, s: u32, p: u32, o: u32, g: u32) {
        self.trees.delete_by_index([s, p, o, g]);
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    /// Returns true if the tree has the specified quad
    pub fn has(&self, s: u32, p: u32, o: u32, g: u32) -> bool {
        self.trees.base_tree.0.contains(&self.trees.base_tree.1, &[s, p, o, g])
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    /// Returns a slice with every quad flattened
    pub fn get_all(&self, s: Option<u32>, p: Option<u32>, o: Option<u32>, g: Option<u32>) -> Box<[u32]> {
        // We return a Box<[u32]> because :
        // 1- wasm bindgen has a memory friendly way to return this data structure (no memory leak)
        // 2- memcpy-ing is stupidly fast

        let mut vector = vec!();

        for quad in self.trees.filter([s, p, o, g]) {
            vector.push(quad[0]);
            vector.push(quad[1]);
            vector.push(quad[2]);
            vector.push(quad[3]);
        }
        
        vector.into_boxed_slice()
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    /// Builds a new dataset which is built by filtering with the given s, p, o and g.
    pub fn new_from(&self, s: Option<u32>, p: Option<u32>, o: Option<u32>, g: Option<u32>) -> Self {
        let mut new_tree = ExportedTrees::new();
        self.trees
            .filter([s, p, o, g])
            .for_each(|quad| { new_tree.trees.insert_by_index(quad); } );
        new_tree
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    pub fn insert_all_from_slice(&mut self, encoded_quads: &[u32]) {
        for i in 0..(encoded_quads.len() / 4) {
            self.add(
                encoded_quads[i * 4 + 0],
                encoded_quads[i * 4 + 1],
                encoded_quads[i * 4 + 2],
                encoded_quads[i * 4 + 3]
            );
        }
    }


    /// Builds a TreeDataset from a slice of 4 x u32
    /// 
    /// If you have previously extracted a slice from get_all, you can easily build a new dataset with this function
    pub fn new_from_slice(encoded_quads: &[u32]) -> Self {
        let mut new_tree = Self::new();
        new_tree.insert_all_from_slice(encoded_quads);
        new_tree
    }
}

// ==== RDF.JS Dataset backend implementation
// (https://rdf.js.org/dataset-spec/#dataset-interface)

impl ExportedTrees {
    /// Returns the number of optional trees that are currently instancied
    pub fn number_of_optional_built_trees(&self) -> usize {
        self.trees
            .optional_trees
            .iter()
            .filter(|pair| pair.1.get().is_some())
            .count()
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    /// Removes from the dataset the quads that matches the given pattern
    #[wasm_bindgen(js_name = deleteMatches)]
    pub fn delete_matches(&mut self, s: Option<u32>, p: Option<u32>, o: Option<u32>, g: Option<u32>) {
        // 1- Find quads that matches
        let quads = self.trees.search_all_matching_quads([s, p, o, g], false).collect::<Vec<[u32; 4]>>();

        let ratio_threshhold = 2 + self.number_of_optional_built_trees();

        if quads.len() < self.size() / ratio_threshhold {
            // 2- Remove quads if there are not a lot to remove

            for quad in quads {
                self.trees.delete_by_index(quad);
            }
        } else {
            // 2- If there are a lot, rebuild tree
            let spog = [s, p, o, g];

            // Delete every secondary tree. We do this first to let the new tree
            // eventually reuse the allocated memory of the former trees
            for optional_tree_tuple in self.trees.optional_trees.iter_mut() {
                optional_tree_tuple.1.take();
            }
    
            // Build the new filtered tree and replace the old one
            let new_tree = self.trees.base_tree.0.build_new_tree_by_filtering(&self.trees.base_tree.1, &spog);
            self.trees.base_tree.1 = new_tree;
        }
    }
}

impl ExportedTrees {
    pub fn are_trivially_mergeable_trees(lhs: &Self, rhs: &Self) -> bool {
        // TODO : static_assert
        // TODO : be able to merge indexes that are not the first from new
        let expected_block = [TermRole::Object, TermRole::Graph, TermRole::Predicate, TermRole::Subject];
        lhs.trees.base_tree.0.get_term_roles() == &expected_block && rhs.trees.base_tree.0.get_term_roles() == &expected_block
    }

    fn new_tree_from_fusion<'a, ITER>(iterator: ITER) -> Self
        where ITER: Iterator<Item=&'a Block<u32>> {
        let mut new_tree = Self::new();
        new_tree.trees.base_tree.1.extend(iterator);
        new_tree
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    #[wasm_bindgen]
    pub fn insersect(&self, other: &ExportedTrees) -> ExportedTrees {
        if ExportedTrees::are_trivially_mergeable_trees(self, other) {
            Self::new_tree_from_fusion(self.trees.base_tree.1.intersection(&other.trees.base_tree.1))
        } else {
            let mut new_tree = Self::new();

            for quad in self.trees.filter([None, None, None, None]) {
                if other.has(quad[0], quad[1], quad[2], quad[3]) {
                    new_tree.add(quad[0], quad[1], quad[2], quad[3]);
                }
            }

            new_tree
        }
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    #[wasm_bindgen(js_name = intersectSlice)]
    pub fn intersect_slice(&self, other: &[u32]) -> Self {
        let mut new_tree = Self::new();

        for i in 0..other.len() / 4 {
            if self.has(other[i * 4], other[i * 4 + 1], other[i * 4 + 2], other[i * 4 + 3]) {
                new_tree.add(other[i * 4], other[i * 4 + 1], other[i * 4 + 2], other[i * 4 + 3]);
            }
        }

        new_tree
    }

    #[wasm_bindgen(js_name = unionSlice)]
    pub fn union_slice(&self, other: &[u32]) -> Self {
        let mut new_tree = Self::new();

        for quad in self.trees.filter([None, None, None, None]) {
            new_tree.add(quad[0], quad[1], quad[2], quad[3]);
        }

        for i in 0..other.len() / 4 {
            new_tree.add(other[i * 4], other[i * 4 + 1], other[i * 4 + 2], other[i * 4 + 3]);
        }

        new_tree
    }

    #[wasm_bindgen(js_name = differenceSlice)]
    pub fn difference_slice(&self, other: &[u32]) -> Self {
        let mut new_tree = Self::new();

        for quad in self.trees.filter([None, None, None, None]) {
            new_tree.add(quad[0], quad[1], quad[2], quad[3]);
        }

        for i in 0..other.len() / 4 {
            new_tree.trees.delete_by_index([other[i * 4], other[i * 4 + 1], other[i * 4 + 2], other[i * 4 + 3]]);
        }

        new_tree
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    #[wasm_bindgen]
    pub fn union(&self, other: &ExportedTrees) -> Self {
        if Self::are_trivially_mergeable_trees(self, other) {
            Self::new_tree_from_fusion(self.trees.base_tree.1.union(&other.trees.base_tree.1))
        } else {
            let mut new_tree = Self::new();

            for quad in self.trees.filter([None, None, None, None]) {
                new_tree.add(quad[0], quad[1], quad[2], quad[3]);
            }

            for quad in other.trees.filter([None, None, None, None]) {
                new_tree.add(quad[0], quad[1], quad[2], quad[3]);
            }
            
            new_tree
        }
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    #[wasm_bindgen]
    pub fn difference(&self, other: &ExportedTrees) -> Self {
        if Self::are_trivially_mergeable_trees(self, other) {
            Self::new_tree_from_fusion(self.trees.base_tree.1.difference(&other.trees.base_tree.1))
        } else {
            let mut new_tree = Self::new();

            for quad in self.trees.filter([None, None, None, None]) {
                if !other.has(quad[0], quad[1], quad[2], quad[3]) {
                    new_tree.add(quad[0], quad[1], quad[2], quad[3]);
                }
            }
            
            new_tree
        }
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    #[wasm_bindgen]
    pub fn contains(&self, other: &ExportedTrees) -> bool {
        if Self::are_trivially_mergeable_trees(self, other) {
            self.trees.base_tree.1.is_superset(&other.trees.base_tree.1)
        } else {
            for quad in other.trees.filter([None, None, None, None]) {
                if !self.has(quad[0], quad[1], quad[2], quad[3]) {
                    return false;
                }
            }
            
            true
        }
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    #[wasm_bindgen(js_name = containsSlice)]
    pub fn contains_slice(&self, other: &[u32]) -> bool {
        assert!(other.len() % 4 == 0);
        let mut i = 0;
        while i != other.len() / 4 { 
            if !self.has(other[i * 4], other[i * 4 + 1], other[i * 4 + 2], other[i * 4 + 3]) {
                return false;
            }
            i = i + 1;
        }

        true
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    #[wasm_bindgen(js_name = equalsSlice)]
    pub fn equals_slice(&self, other: &[u32]) -> bool {
        if self.trees.base_tree.1.len() != other.len() / 4 {
            return false;
        }

        self.contains_slice(other)
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    #[wasm_bindgen]
    pub fn has_same_elements(&self, other: &ExportedTrees) -> bool {
        if self.trees.base_tree.1.len() != other.trees.base_tree.1.len() {
            return false;
        }

        self.contains(other)
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    /// Counts the number of quads that matches the given pattern
    #[wasm_bindgen]
    pub fn match_count(&self, s: Option<u32>, p: Option<u32>, o: Option<u32>, g: Option<u32>) -> usize {
        let spog = [s, p, o, g];
        return self.trees.search_all_matching_quads(spog, true).count();
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    /// Ensures the best tree to search quads matching the given pattern is built
    #[wasm_bindgen]
    pub fn ensure_has_index_for(&self, s: bool, p: bool, o: bool, g: bool) {
        let spog: [Option<u32>; 4] = [
            if s { Some(0) } else { None },
            if p { Some(0) } else { None },
            if o { Some(0) } else { None },
            if g { Some(0) } else { None },
        ];

        self.trees.search_all_matching_quads(spog, true);
    }
}

#[wasm_bindgen(js_class="TreedDataset")]
impl ExportedTrees {
    /// Counts the number of underlying trees
    #[wasm_bindgen]
    pub fn number_of_underlying_trees(&self) -> usize {
        1 + self.number_of_optional_built_trees()
    }
}