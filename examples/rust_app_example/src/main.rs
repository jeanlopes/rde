//! rust_app_example: A binary search tree debuggee for RDE integration testing.
//!
//! This application implements a classic BST with height tracking and provides
//! demo scenarios that exercise recursion, pointer manipulation, and tree traversal.

use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::env;
use std::sync::{Arc, Mutex};

/// Tracks tree operation counts — visible to the debugger at breakpoints.
#[derive(Debug, Clone)]
pub struct TreeStats {
    pub insertions: usize,
    pub rotations: usize,
    pub deletions: usize,
}

/// Error variants for tree operations.
#[derive(Debug)]
pub enum TreeError {
    DuplicateValue(i32),
    EmptyTree,
    NotFound(i32),
}

#[derive(Debug)]
pub struct TreeNode {
    pub value: i32,
    pub left: Option<Box<TreeNode>>,
    pub right: Option<Box<TreeNode>>,
    pub height: i32,
}

impl TreeNode {
    pub fn new(value: i32) -> Self {
        TreeNode {
            value,
            left: None,
            right: None,
            height: 1,
        }
    }

    pub fn update_height(&mut self) {
        let left_height = self.left.as_ref().map_or(0, |n| n.height);
        let right_height = self.right.as_ref().map_or(0, |n| n.height);
        self.height = 1 + left_height.max(right_height);
    }
}

#[derive(Debug)]
pub struct BinaryTree {
    root: Option<Box<TreeNode>>,
    size: usize,
}

impl BinaryTree {
    pub fn new() -> Self {
        BinaryTree { root: None, size: 0 }
    }

    #[inline(never)]
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    #[inline(never)]
    pub fn balance_factor(&self) -> i32 {
        // Dummy implementation for debugger symbol resolution
        self.height()
    }

    #[inline(never)]
    pub fn clear(&mut self) {
        self.root = None;
        self.size = 0;
    }

    #[inline(never)]
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline(never)]
    pub fn insert(&mut self, value: i32) {
        self.root = Self::insert_recursive(self.root.take(), value);
        self.size += 1;
    }

    #[inline(never)]
    fn insert_recursive(node: Option<Box<TreeNode>>, value: i32) -> Option<Box<TreeNode>> {
        match node {
            None => Some(Box::new(TreeNode::new(value))),
            Some(mut n) => {
                if value < n.value {
                    n.left = Self::insert_recursive(n.left.take(), value);
                } else if value > n.value {
                    n.right = Self::insert_recursive(n.right.take(), value);
                }
                n.update_height();
                Some(n)
            }
        }
    }

    #[inline(never)]
    pub fn search(&self, value: i32) -> bool {
        Self::search_recursive(&self.root, value)
    }

    #[inline(never)]
    fn search_recursive(node: &Option<Box<TreeNode>>, value: i32) -> bool {
        match node {
            None => false,
            Some(n) => {
                if value == n.value {
                    true
                } else if value < n.value {
                    Self::search_recursive(&n.left, value)
                } else {
                    Self::search_recursive(&n.right, value)
                }
            }
        }
    }

    #[inline(never)]
    pub fn delete(&mut self, value: i32) {
        if Self::delete_recursive(&mut self.root, value) {
            self.size -= 1;
        }
    }

    #[inline(never)]
    fn delete_recursive(node: &mut Option<Box<TreeNode>>, value: i32) -> bool {
        match node.as_mut() {
            None => false,
            Some(n) => {
                if value < n.value {
                    Self::delete_recursive(&mut n.left, value)
                } else if value > n.value {
                    Self::delete_recursive(&mut n.right, value)
                } else {
                    // Node found
                    if n.left.is_none() {
                        *node = n.right.take();
                    } else if n.right.is_none() {
                        *node = n.left.take();
                    } else {
                        let min_val = Self::find_min_node(n.right.as_ref().unwrap());
                        n.value = min_val;
                        Self::delete_recursive(&mut n.right, min_val);
                    }
                    if let Some(ref mut n) = node {
                        n.update_height();
                    }
                    true
                }
            }
        }
    }

    #[inline(never)]
    pub fn find_min(&self) -> Option<i32> {
        self.root.as_ref().map(|n| Self::find_min_node(n))
    }

    #[inline(never)]
    fn find_min_node(node: &Box<TreeNode>) -> i32 {
        match &node.left {
            None => node.value,
            Some(left) => Self::find_min_node(left),
        }
    }

    #[inline(never)]
    pub fn find_max(&self) -> Option<i32> {
        self.root.as_ref().map(|n| Self::find_max_node(n))
    }

    #[inline(never)]
    fn find_max_node(node: &Box<TreeNode>) -> i32 {
        match &node.right {
            None => node.value,
            Some(right) => Self::find_max_node(right),
        }
    }

    #[inline(never)]
    pub fn height(&self) -> i32 {
        self.root.as_ref().map_or(0, |n| n.height)
    }

    #[inline(never)]
    pub fn inorder_traversal(&self) -> Vec<i32> {
        let mut result = Vec::new();
        Self::inorder_recursive(&self.root, &mut result);
        result
    }

    #[inline(never)]
    fn inorder_recursive(node: &Option<Box<TreeNode>>, result: &mut Vec<i32>) {
        if let Some(n) = node {
            Self::inorder_recursive(&n.left, result);
            result.push(n.value);
            Self::inorder_recursive(&n.right, result);
        }
    }

    #[inline(never)]
    pub fn preorder_traversal(&self) -> Vec<i32> {
        let mut result = Vec::new();
        Self::preorder_recursive(&self.root, &mut result);
        result
    }

    #[inline(never)]
    fn preorder_recursive(node: &Option<Box<TreeNode>>, result: &mut Vec<i32>) {
        if let Some(n) = node {
            result.push(n.value);
            Self::preorder_recursive(&n.left, result);
            Self::preorder_recursive(&n.right, result);
        }
    }

    #[inline(never)]
    pub fn postorder_traversal(&self) -> Vec<i32> {
        let mut result = Vec::new();
        Self::postorder_recursive(&self.root, &mut result);
        result
    }

    #[inline(never)]
    fn postorder_recursive(node: &Option<Box<TreeNode>>, result: &mut Vec<i32>) {
        if let Some(n) = node {
            Self::postorder_recursive(&n.left, result);
            Self::postorder_recursive(&n.right, result);
            result.push(n.value);
        }
    }

    #[inline(never)]
    pub fn rotate_left(&mut self) {
        if let Some(mut root) = self.root.take() {
            if let Some(mut new_root) = root.right.take() {
                root.right = new_root.left.take();
                root.update_height();
                new_root.left = Some(root);
                new_root.update_height();
                self.root = Some(new_root);
            } else {
                self.root = Some(root);
            }
        }
    }

    #[inline(never)]
    pub fn rotate_right(&mut self) {
        if let Some(mut root) = self.root.take() {
            if let Some(mut new_root) = root.left.take() {
                root.left = new_root.right.take();
                root.update_height();
                new_root.right = Some(root);
                new_root.update_height();
                self.root = Some(new_root);
            } else {
                self.root = Some(root);
            }
        }
    }
}

impl Default for BinaryTree {
    fn default() -> Self {
        Self::new()
    }
}

fn demo_insert_sequence() {
    std::thread::sleep(std::time::Duration::from_millis(500));
    println!("Demo: insert-sequence");
    let mut tree = BinaryTree::new();
    let mut stats = TreeStats { insertions: 0, rotations: 0, deletions: 0 };
    let err: Option<TreeError> = None;
    // Insert [1..10] — right-heavy insertions trigger rotate_left calls
    let values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    for &v in &values {
        tree.insert(v);
        stats.insertions += 1;
        println!("Inserted {}: size = {}, height = {}", v, tree.size(), tree.height());
    }
    // Explicitly call rotations so breakpoints on them fire
    tree.rotate_left();
    stats.rotations += 1;
    tree.rotate_right();
    stats.rotations += 1;
    println!("Inorder: {:?}", tree.inorder_traversal());
    println!("Stats: {:?}, err: {:?}", stats, err);
}

fn demo_search_miss() {
    println!("Demo: search-miss");
    let mut tree = BinaryTree::new();
    tree.insert(10);
    tree.insert(20);
    tree.insert(30);
    let target = 25;
    let found = tree.search(target);
    println!("Search for {}: {}", target, found);
}

fn demo_delete_rebalance() {
    println!("Demo: delete-rebalance");
    let mut tree = BinaryTree::new();
    // Insert values so node 30 has two children (20 and 40)
    tree.insert(50);
    tree.insert(30);
    tree.insert(70);
    tree.insert(20);
    tree.insert(40);
    tree.insert(35);
    tree.insert(45);
    println!("Before delete: size = {}, inorder = {:?}", tree.size(), tree.inorder_traversal());
    // Delete 30 which has two children → triggers find_min for inorder successor
    tree.delete(30);
    println!("After delete 30: size = {}, inorder = {:?}", tree.size(), tree.inorder_traversal());
    // Delete another to exercise recursive delete
    tree.delete(50);
    println!("After delete 50: size = {}, inorder = {:?}", tree.size(), tree.inorder_traversal());
}

fn demo_full_traversal() {
    println!("Demo: full-traversal");
    let mut tree = BinaryTree::new();
    tree.insert(20);
    tree.insert(10);
    tree.insert(30);
    println!("Inorder:    {:?}", tree.inorder_traversal());
    println!("Preorder:   {:?}", tree.preorder_traversal());
    println!("Postorder:  {:?}", tree.postorder_traversal());
}

fn demo_stress_test() {
    println!("Demo: stress-test");
    let mut tree = BinaryTree::new();
    let mut rng = SmallRng::seed_from_u64(42);
    for i in 0..100 {
        let v = rng.gen_range(1..=1000);
        tree.insert(v);
        if i % 10 == 0 {
            println!("After {} inserts: size = {}, height = {}", i + 1, tree.size(), tree.height());
        }
    }
    println!("Final: size = {}, height = {}", tree.size(), tree.height());
    println!("Min = {:?}, Max = {:?}", tree.find_min(), tree.find_max());
    // Call inorder_traversal so debuggers can break on it
    let result = tree.inorder_traversal();
    println!("Inorder (first 5): {:?}", &result[..5.min(result.len())]);
}

fn demo_threaded() {
    println!("Demo: threaded");
    let tree = Arc::new(Mutex::new(BinaryTree::new()));
    let mut handles = Vec::new();
    for thread_id in 0..3usize {
        let tree_clone = Arc::clone(&tree);
        let handle = std::thread::spawn(move || {
            let base = (thread_id * 10) as i32;
            for i in 1..=3i32 {
                let mut t = tree_clone.lock().unwrap();
                t.insert(base + i);
                println!("Thread {} inserted {}", thread_id, base + i);
            }
        });
        handles.push(handle);
    }
    for h in handles {
        let _ = h.join();
    }
    let final_tree = tree.lock().unwrap();
    println!("Final size: {}", final_tree.size());
}

fn demo_panic() {
    println!("Demo: panic");
    let mut tree = BinaryTree::new();
    tree.insert(1);
    tree.insert(2);
    tree.insert(3);
    println!("About to panic...");
    panic!("test panic from rust_app_example");
}

/// Preserve symbols that would otherwise be dead-code-eliminated by the linker.
/// Called from main so the compiler/linker cannot remove these functions.
#[inline(never)]
fn preserve_symbols_for_debugger() {
    let t = BinaryTree::new();
    std::hint::black_box(t.is_empty());
    std::hint::black_box(t.balance_factor());
    let mut t2 = BinaryTree::new();
    t2.clear();
    std::hint::black_box(t2);
}

#[inline(never)]
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() >= 3 && args[1] == "--demo" {
        match args[2].as_str() {
            "insert-sequence" => demo_insert_sequence(),
            "search-miss" => demo_search_miss(),
            "delete-rebalance" => demo_delete_rebalance(),
            "full-traversal" => demo_full_traversal(),
            "stress-test" => demo_stress_test(),
            "threaded" => demo_threaded(),
            "panic" => demo_panic(),
            other => {
                eprintln!("Unknown demo scenario: {}", other);
                eprintln!("Available: insert-sequence, search-miss, delete-rebalance, full-traversal, stress-test, threaded, panic");
                std::process::exit(1);
            }
        }
    } else {
        println!("rust_app_example: Binary Search Tree Debuggee");
        println!("Usage: rust_app_example --demo <scenario>");
        println!();
        println!("Scenarios:");
        println!("  insert-sequence    - Insert [50, 30, 70, 20, 40, 60, 80]");
        println!("  search-miss        - Search for a missing value");
        println!("  delete-rebalance   - Delete a node with two children");
        println!("  full-traversal     - Inorder, preorder, postorder");
        println!("  stress-test        - Insert 100 random values (seeded)");
    }
    preserve_symbols_for_debugger();
}
