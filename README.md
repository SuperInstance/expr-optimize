# expr-optimize

**AST-level expression optimization** in pure Rust: constant folding, algebraic simplification identities (x+0, x×1, x×0, x−0, x/1), boolean short-circuit elimination, double-negation removal, and dead-branch pruning for ternary conditionals. A minimal compiler optimization pass.

## Why It Matters

Every modern compiler (LLVM, GCC, rustc) performs constant folding and algebraic simplification in its early optimization passes. These transformations reduce code size and improve runtime performance without changing semantics — the optimized AST evaluates to the same value as the original.

This crate implements these optimizations for a small expression language with arithmetic (`+`, `−`, `×`, `÷`), boolean (`&&`, `||`, `!`), and ternary (`?:`) operators. It demonstrates the core techniques that production compilers use:

1. **Constant folding**: Evaluate sub-expressions at compile time when all operands are literals.
2. **Algebraic identities**: Replace expressions with simpler equivalents (x + 0 → x).
3. **Boolean simplification**: Short-circuit evaluation eliminates unreachable branches (false && x → false).
4. **Ternary pruning**: If the condition is known, eliminate the dead branch.

## How It Works

### AST Representation

```rust
enum Expr {
    Lit(f64),               // Numeric literal
    Bool(bool),             // Boolean literal
    Var(String),            // Variable reference
    Binary { op, left, right },  // Binary operation
    Not(Box<Expr>),         // Logical negation
    Ternary { cond, then, else_ }, // Conditional expression
}
```

### Optimization: Recursive Bottom-Up Rewriting

The `optimize()` function recursively optimizes children before applying local rewrite rules. This bottom-up traversal ensures that simplifications compose: if a child simplifies to a literal, the parent can then fold.

**Complexity**: O(n) per pass, where n = number of AST nodes. Each node is visited exactly once. The recursive depth is O(d) where d = tree depth (stack usage).

### Rewrite Rules

#### Constant Folding (arithmetic)

When both operands are literals, evaluate at compile time:

| Expression | Result |
|-----------|--------|
| `Lit(a) + Lit(b)` | `Lit(a + b)` |
| `Lit(a) − Lit(b)` | `Lit(a − b)` |
| `Lit(a) × Lit(b)` | `Lit(a × b)` |
| `Lit(a) ÷ Lit(b)` | `Lit(a / b)` if b ≠ 0 |

#### Algebraic Identities

| Rule | Simplification | Justification |
|------|---------------|---------------|
| `x + 0` | `x` | Additive identity |
| `0 + x` | `x` | Additive identity (commutative) |
| `x × 1` | `x` | Multiplicative identity |
| `1 × x` | `x` | Multiplicative identity (commutative) |
| `x × 0` | `0` | Annihilator |
| `0 × x` | `0` | Annihilator (commutative) |
| `x − 0` | `x` | Subtractive identity |
| `x ÷ 1` | `x` | Division identity |

#### Boolean Short-Circuit

| Rule | Result | Reason |
|------|--------|--------|
| `false && x` | `false` | False dominates AND |
| `true && x` | `x` | Identity for AND |
| `true \|\| x` | `true` | True dominates OR |
| `false \|\| x` | `x` | Identity for OR |

#### Negation and Ternary

| Rule | Result |
|------|--------|
| `!!x` | `x` (double negation) |
| `!(true/false)` | `false/true` (constant) |
| `true ? a : b` | `a` (dead branch elimination) |
| `false ? a : b` | `b` (dead branch elimination) |
| `c ? a : a` | `c` (identical branches, side-effect free) |

### Soundness

All transformations are semantics-preserving: the optimized expression evaluates to the same value as the original for all variable bindings. The proof is by case analysis on each rewrite rule — each rule is an instance of a well-known algebraic law (identity, annihilator, dominance, idempotence).

### Division by Zero

The optimizer deliberately does *not* fold `Lit(a) / Lit(0.0)` — it preserves the expression so that runtime evaluation produces `Infinity` or `NaN` as expected, rather than silently producing a wrong value at compile time.

## Quick Start

```rust
// The library is currently structured as a binary with a optimize() function.
// See src/main.rs for the full implementation.

// Example transformations:
// "x + 0"     → "x"
// "x * 1"     → "x"
// "2 + 3 * 4" → "14"     (constant folding: 2 + 12 = 14)
// "0 * x"     → "0"      (annihilator)
// "!!true"    → "true"   (double negation)
// "true ? a : b" → "a"   (dead branch pruning)
// "x && false"   → "false" (short-circuit)
// "y / 1"     → "y"      (division identity)
```

## API

### `Expr` (AST)
- `Lit(f64)` — Numeric literal
- `Bool(bool)` — Boolean literal
- `Var(String)` — Variable reference
- `Binary { op: BinOp, left: Box<Expr>, right: Box<Expr> }` — Binary operation
- `Not(Box<Expr>)` — Logical negation
- `Ternary { cond, then_br, else_br }` — Conditional

### `BinOp`
`Add` | `Sub` | `Mul` | `Div` | `And` | `Or`

### Functions
- `optimize(expr: Expr) -> Expr` — Apply all optimization passes recursively
- `display(expr: &Expr) -> String` — Pretty-print an expression

## Architecture Notes

This crate provides the optimization pass for expression ASTs in the SuperInstance stack. It connects to:

- **expr-parser** — Produces the unoptimized AST that this crate optimizes
- **cuda-oxide** — Uses optimized ASTs for GPU code generation

The conservation link γ + η = C applies: γ (reduced expressions) + η (eliminated operations) = C (total semantic content). Optimization preserves C exactly — no information is lost, only representation changes.

See the full architecture: [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md)

## References

1. Aho, A.V., Lam, M.S., Sethi, R., & Ullman, J.D. (2006). *Compilers: Principles, Techniques, and Tools,* 2nd ed. Pearson. Chapter 8 (code optimization).
2. Appel, A.W. (2004). *Modern Compiler Implementation.* Cambridge University Press.
3. LLVM Project. "LLVM Language Reference Manual: Constant Folding." [llvm.org/docs](https://llvm.org/docs/LangRef.html)
4. Kildall, G.A. (1973). "A Unified Approach to Global Program Optimization." *POPL '73.* — Lattice-based optimization framework.

## License

MIT
