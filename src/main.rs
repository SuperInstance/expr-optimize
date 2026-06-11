//! expr-optimize — AST-level optimizations: constant folding, algebraic simplification, dead code elimination.

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq)]
enum BinOp { Add, Sub, Mul, Div, And, Or }

/// AST for boolean/arithmetic expressions.
#[derive(Debug, Clone, PartialEq)]
enum Expr {
    Lit(f64),
    Bool(bool),
    Var(String),
    Binary { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Not(Box<Expr>),
    Ternary { cond: Box<Expr>, then_br: Box<Expr>, else_br: Box<Expr> },
}

impl Expr {
    fn lit(v: f64) -> Self { Expr::Lit(v) }
    fn var(name: &str) -> Self { Expr::Var(name.into()) }
    fn bin(op: BinOp, l: Expr, r: Expr) -> Self { Expr::Binary { op, left: Box::new(l), right: Box::new(r) } }
    fn not(e: Expr) -> Self { Expr::Not(Box::new(e)) }
    fn ternary(c: Expr, t: Expr, e: Expr) -> Self { Expr::Ternary { cond: Box::new(c), then_br: Box::new(t), else_br: Box::new(e) } }
}

/// Optimize pass — applies constant folding and algebraic identities.
fn optimize(expr: Expr) -> Expr {
    match expr {
        Expr::Binary { op, left, right } => {
            let l = optimize(*left);
            let r = optimize(*right);

            // Constant folding
            if let (Expr::Lit(a), Expr::Lit(b)) = (&l, &r) {
                return Expr::Lit(match op {
                    BinOp::Add => a + b,
                    BinOp::Sub => a - b,
                    BinOp::Mul => a * b,
                    BinOp::Div if *b != 0.0 => a / b,
                    BinOp::Div => return Expr::bin(op, l, r),
                    _ => return Expr::bin(op, l, r),
                });
            }

            // x + 0 => x, 0 + x => x
            if op == BinOp::Add {
                if let Expr::Lit(0.0) = &r { return l; }
                if let Expr::Lit(0.0) = &l { return r; }
            }
            // x * 1 => x, 1 * x => x
            if op == BinOp::Mul {
                if let Expr::Lit(1.0) = &r { return l; }
                if let Expr::Lit(1.0) = &l { return r; }
                // x * 0 => 0, 0 * x => 0
                if let Expr::Lit(0.0) = &r { return Expr::Lit(0.0); }
                if let Expr::Lit(0.0) = &l { return Expr::Lit(0.0); }
            }
            // x - 0 => x
            if op == BinOp::Sub && matches!(&r, Expr::Lit(0.0)) { return l; }
            // x / 1 => x
            if op == BinOp::Div && matches!(&r, Expr::Lit(1.0)) { return l; }

            // Boolean short-circuit
            if op == BinOp::And && matches!(&l, Expr::Bool(false)) { return Expr::Bool(false); }
            if op == BinOp::And && matches!(&r, Expr::Bool(true)) { return l; }
            if op == BinOp::Or && matches!(&l, Expr::Bool(true)) { return Expr::Bool(true); }
            if op == BinOp::Or && matches!(&r, Expr::Bool(false)) { return l; }

            Expr::bin(op, l, r)
        }
        Expr::Not(inner) => {
            let inner = optimize(*inner);
            match inner {
                Expr::Bool(b) => Expr::Bool(!b),
                Expr::Not(e) => *e, // double negation
                _ => Expr::Not(Box::new(inner)),
            }
        }
        Expr::Ternary { cond, then_br, else_br } => {
            let c = optimize(*cond);
            let t = optimize(*then_br);
            let e = optimize(*else_br);
            match &c {
                Expr::Bool(true) => t,
                Expr::Bool(false) => e,
                _ if t == e => c, // both branches identical → just condition (side-effect free)
                _ => Expr::ternary(c, t, e),
            }
        }
        other => other,
    }
}

/// Pretty-print an expression.
fn display(expr: &Expr) -> String {
    match expr {
        Expr::Lit(n) => format!("{n}"),
        Expr::Bool(b) => b.to_string(),
        Expr::Var(v) => v.clone(),
        Expr::Binary { op, left, right } => format!("({} {} {})", display(left), match op {
            BinOp::Add => "+", BinOp::Sub => "-", BinOp::Mul => "*", BinOp::Div => "/",
            BinOp::And => "&&", BinOp::Or => "||",
        }, display(right)),
        Expr::Not(e) => format!("!{}", display(e)),
        Expr::Ternary { cond, then_br, else_br } => format!("({} ? {} : {})", display(cond), display(then_br), display(else_br)),
    }
}

fn main() {
    let cases = vec![
        ("x + 0", Expr::bin(BinOp::Add, Expr::var("x"), Expr::lit(0.0))),
        ("x * 1", Expr::bin(BinOp::Mul, Expr::var("x"), Expr::lit(1.0))),
        ("2 + 3 * 4", Expr::bin(BinOp::Add, Expr::lit(2.0), Expr::bin(BinOp::Mul, Expr::lit(3.0), Expr::lit(4.0)))),
        ("0 * x", Expr::bin(BinOp::Mul, Expr::lit(0.0), Expr::var("x"))),
        ("!!true", Expr::not(Expr::not(Expr::Bool(true)))),
        ("true ? a : b", Expr::ternary(Expr::Bool(true), Expr::var("a"), Expr::var("b"))),
        ("x && false", Expr::bin(BinOp::And, Expr::var("x"), Expr::Bool(false))),
        ("y / 1", Expr::bin(BinOp::Div, Expr::var("y"), Expr::lit(1.0))),
    ];

    println!("expr-optimize — AST optimizer");
    println!("=============================\n");
    println!("{:<25} | {:<25}", "BEFORE", "AFTER");
    println!("{}", "-".repeat(55));
    for (label, expr) in cases {
        let opt = optimize(expr);
        println!("{:<25} | {:<25}", label, display(&opt));
    }
}
