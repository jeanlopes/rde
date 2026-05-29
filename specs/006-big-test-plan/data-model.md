# Data Model: rust_app_example (Binary Tree Debuggee)

**Feature**: 006-big-test-plan
**Date**: 2026-05-28

---

## Entity: TreeNode

Represents a single node in the binary search tree.

```rust
pub struct TreeNode {
    pub value: i32,
    pub left: Option<Box<TreeNode>>,
    pub right: Option<Box<TreeNode>>,
    pub height: i32,
}
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `value` | `i32` | The key/value stored in this node. Used for BST ordering. |
| `left` | `Option<Box<TreeNode>>` | Left child (values < `self.value`). `None` if no child. |
| `right` | `Option<Box<TreeNode>>` | Right child (values > `self.value`). `None` if no child. |
| `height` | `i32` | Height of this node (1 + max(height(left), height(right))). 0 for leaf. |

### Invariants

- BST ordering: `left.value < self.value < right.value` (for all non-None children).
- No duplicates: insert of existing value may be rejected or ignored (implementation-defined).
- `height` is always consistent with children heights.

---

## Entity: BinaryTree

The main tree structure exposed to the debugger.

```rust
pub struct BinaryTree {
    root: Option<Box<TreeNode>>,
}
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `fn new() -> Self` | Creates an empty tree. |
| `insert` | `fn insert(&mut self, value: i32)` | Inserts a value into the tree. Recursive. May call `rotate_left`/`rotate_right` if exposed. |
| `search` | `fn search(&self, value: i32) -> bool` | Returns true if value exists in the tree. Recursive. |
| `delete` | `fn delete(&mut self, value: i32)` | Removes a value. Handles leaf, one-child, and two-child cases. Recursive. |
| `find_min` | `fn find_min(&self) -> Option<i32>` | Returns the minimum value in the tree. |
| `find_max` | `fn find_max(&self) -> Option<i32>` | Returns the maximum value in the tree. |
| `height` | `fn height(&self) -> i32` | Returns the height of the tree (0 for empty). |
| `size` | `fn size(&self) -> usize` | Returns the number of nodes. |
| `inorder_traversal` | `fn inorder_traversal(&self) -> Vec<i32>` | Returns values in sorted order. |
| `preorder_traversal` | `fn preorder_traversal(&self) -> Vec<i32>` | Returns values in pre-order. |
| `postorder_traversal` | `fn postorder_traversal(&self) -> Vec<i32>` | Returns values in post-order. |
| `is_empty` | `fn is_empty(&self) -> bool` | Returns true if root is None. |
| `clear` | `fn clear(&mut self)` | Removes all nodes (drops the tree). |
| `rotate_left` | `fn rotate_left(&mut self)` | Rotates the tree left around root (for testing step into). |
| `rotate_right` | `fn rotate_right(&mut self)` | Rotates the tree right around root (for testing step into). |

---

## Entity: DemoScenario

Maps CLI arguments to predefined tree operations.

```rust
enum DemoScenario {
    InsertSequence,
    SearchMiss,
    DeleteRebalance,
    FullTraversal,
    StressTest,
}
```

### Scenarios

| Scenario | Argument | Operations |
|----------|----------|------------|
| `InsertSequence` | `--demo insert-sequence` | Insert [50, 30, 70, 20, 40, 60, 80] in order. |
| `SearchMiss` | `--demo search-miss` | Insert [10, 20, 30], then search for 25 (not found). |
| `DeleteRebalance` | `--demo delete-rebalance` | Insert [50, 30, 70, 20, 40], then delete 30 (has two children). |
| `FullTraversal` | `--demo full-traversal` | Insert [20, 10, 30], then inorder/preorder/postorder. |
| `StressTest` | `--demo stress-test` | Insert 100 random values (seeded for determinism). |

### Determinism

- `StressTest` uses a fixed seed (`rand::SeedableRng::seed_from_u64(42)`) to ensure golden paths são reproduzíveis.
- Todos os outros scenarios usam sequências fixas.

---

## Relationships

```text
BinaryTree *-- TreeNode : root
TreeNode "0..1" --> "0..1" TreeNode : left
TreeNode "0..1" --> "0..1" TreeNode : right
BinaryTree --> DemoScenario : executes
```

---

## Validation Rules

1. **Insert ordering**: `value` must follow BST invariants after insertion.
2. **Delete completeness**: After `delete(value)`, `search(value)` must return `false`.
3. **Traversal correctness**: `inorder_traversal` must return strictly ascending sorted values.
4. **Height accuracy**: `height` of a single-node tree is 1; empty tree is 0.
5. **Memory safety**: `clear` must drop all nodes without leaks (verifiable via debugger inspection of heap).
