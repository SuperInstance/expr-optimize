# Expression Optimizer

**Expression optimization** is the process of transforming an abstract syntax tree (AST) into a semantically equivalent but more efficient form through constant folding, algebraic simplification, and dead-branch elimination.

## Why It Matters

Every compiler and query planner performs optimization passes. Without them, `x + 0` would waste a CPU cycle, `0 * x` would compute a pointless product, and `true ? a : b` would evaluate both branches. In database engines like PostgreSQL and query frameworks like Apache Calcite, algebraic identities such as `x * 1 = x` and boolean short-circuits reduce evaluation cost by orders of magnitude on large datasets. This crate implements these classical optimizations in a standalone, zero-dependency Rust library, making the techniques transparent and auditable.

## How It Works

The optimizer performs a **bottom-up rewrite** of the expression tree. Each subtree is optimized before its parent, so simplifications cascade upward: folding `2 + 3 * 4` first collapses `3 * 4 → 12`, then `2 + 12 → 14`.

### Constant Folding

When both operands of a binary node are literals, the optimizer evaluates them at compile time:

```
Lit(a) op Lit(b) → Lit(a op b)     // O(1) per node
```

### Algebraic Identities

The optimizer checks for **annihilators** and **identities** — elements that reduce an expression to a simpler form without full evaluation:

| Identity | Result |
|----------|--------|
| `x + 0`  | `x`    |
| `0 + x`  | `x`    |
| `x * 1`  | `x`    |
| `x * 0`  | `0`    |
| `x - 0`  | `x`    |
| `x / 1`  | `x`    |

### Boolean Short-Circuit

For logical operators, known operands trigger immediate collapse:
- `false && x → false` (left annihilator)
- `true && x → x` (left identity)
- `true || x → true` (left annihilator)
- `x || false → x` (right identity)

### Double Negation and Ternary

`!!x → x` eliminates redundant negation. For ternary expressions `(c ? t : e)`, if the condition is a known boolean the optimizer selects the branch directly; if both branches are structurally identical, the condition alone suffices.

### Complexity

Each pass is **O(n)** in the number of AST nodes — a single traversal with no fixpoint iteration needed for these peephole rewrites.

## Quick Start

```rust
// See src/main.rs for the full implementation.
// Run the demo:
// $ cargo run

fn main() {
    // The optimizer transforms:
    //   "x + 0"       → "x"
    //   "x * 1"       → "x"
    //   "2 + 3 * 4"   → "14"
    //   "0 * x"       → "0"
    //   "!!true"      → "true"
    //   "true ? a : b" → "a"
}
```

## API

| Type / Function | Description |
|----------------|-------------|
| `Expr` | AST enum: `Lit(f64)`, `Bool(bool)`, `Var(String)`, `Binary`, `Not`, `Ternary` |
| `BinOp` | Binary operator enum: `Add`, `Sub`, `Mul`, `Div`, `And`, `Or` |
| `optimize(expr: Expr) → Expr` | Apply all optimization passes recursively |
| `display(expr: &Expr) → String` | Pretty-print an expression tree |

## Architecture Notes

This crate is part of the **SuperInstance** expression pipeline: `expr-parser` produces the AST, `expr-typecheck` validates it, and `expr-optimize` reduces it before evaluation. Together they implement the correctness-efficiency duality **γ + η = C** — where γ (type safety) and η (optimization) combine to produce correct, efficient computation.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for the full system design.

## References

1. Aho, Lam, Sethi, Ullman. *Compilers: Principles, Techniques, and Tools* (Dragon Book), 2nd ed., Chapter 8.
2. Click, C. "Combining Analyses, Combining Optimizations." *ACM TOPLAS*, 1995.
3. Tate, Stepp, Tatlock, Lerner. "Equality Saturation: A New Approach to Optimization." *POPL 2009*.

## License

MIT
