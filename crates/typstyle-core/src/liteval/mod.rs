//! Evaluate simple constant Typst expressions in code mode without scopes or VMs.
//!
//! Currently, this is only used for determine table columns.

use typst_syntax::ast::*;

/// A value of a simple constant Typst expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// The `none` value.
    None,
    /// The `auto` value.
    Auto,
    /// An integer.
    Int(i64),
    /// Only represented by length.
    ///
    /// An array evaluates to the number of its items.
    Array(usize),
}

/// An error encountered while evaluating a constant expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    /// The expression kind or operator is not supported by the evaluator.
    NotSupported,
    /// The operands do not support the operation (e.g., division by zero).
    InvalidOperation,
}

/// The result of evaluating a constant expression.
pub type EvalResult = Result<Value, EvalError>;

/// A lightweight, read-only evaluator of Typst AST nodes into [`Value`]s.
pub trait Liteval {
    /// Evaluates the node to a [`Value`], or reports an [`EvalError`].
    fn liteval(&self) -> EvalResult;
}

impl Liteval for Expr<'_> {
    /// Evaluates the expression by kind; unsupported kinds yield
    /// [`EvalError::NotSupported`].
    fn liteval(&self) -> EvalResult {
        match self {
            Expr::None(v) => v.liteval(),
            Expr::Auto(v) => v.liteval(),
            Expr::Int(v) => v.liteval(),
            Expr::Parenthesized(v) => v.liteval(),
            Expr::Array(v) => v.liteval(),
            Expr::Unary(v) => v.liteval(),
            Expr::Binary(v) => v.liteval(),
            _ => Err(EvalError::NotSupported),
        }
    }
}

impl Liteval for None<'_> {
    /// Evaluates the `none` literal to [`Value::None`].
    fn liteval(&self) -> EvalResult {
        Ok(Value::None)
    }
}

impl Liteval for Auto<'_> {
    /// Evaluates the `auto` literal to [`Value::Auto`].
    fn liteval(&self) -> EvalResult {
        Ok(Value::Auto)
    }
}

impl Liteval for Int<'_> {
    /// Evaluates the integer literal to [`Value::Int`].
    fn liteval(&self) -> EvalResult {
        Ok(Value::Int(self.get()))
    }
}

impl Liteval for Parenthesized<'_> {
    /// Evaluates the parenthesized inner expression.
    fn liteval(&self) -> EvalResult {
        self.expr().liteval()
    }
}

impl Liteval for Array<'_> {
    /// Evaluates the array to [`Value::Array`] with the number of its items.
    fn liteval(&self) -> EvalResult {
        Ok(Value::Array(self.items().count()))
    }
}

impl Liteval for Unary<'_> {
    /// Evaluates the unary expression.
    ///
    /// Only `+` and `-` on integers are supported; `not` is not supported.
    fn liteval(&self) -> EvalResult {
        let expr = self.expr().liteval()?;
        match self.op() {
            UnOp::Pos => match expr {
                Value::Int(i) => Ok(Value::Int(i)),
                _ => Err(EvalError::InvalidOperation),
            },
            UnOp::Neg => match expr {
                Value::Int(i) => Ok(Value::Int(-i)),
                _ => Err(EvalError::InvalidOperation),
            },
            UnOp::Not => Err(EvalError::NotSupported),
        }
    }
}
impl Liteval for Binary<'_> {
    /// Evaluates the binary expression.
    ///
    /// Supports integer arithmetic, array concatenation (`+`), and array scaling
    /// (`*` by a non-negative integer); division by zero is invalid.
    fn liteval(&self) -> EvalResult {
        let lhs = self.lhs().liteval()?;
        let rhs = self.rhs().liteval()?;
        match self.op() {
            BinOp::Add => match (lhs, rhs) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l + r)),
                (Value::Array(l), Value::Array(r)) => Ok(Value::Array(l + r)),
                _ => Err(EvalError::InvalidOperation),
            },
            BinOp::Sub => match (lhs, rhs) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l - r)),
                _ => Err(EvalError::InvalidOperation),
            },
            BinOp::Mul => match (lhs, rhs) {
                (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l * r)),
                (Value::Array(l), Value::Int(r)) if r >= 0 => Ok(Value::Array(l * r as usize)),
                (Value::Int(l), Value::Array(r)) if l >= 0 => Ok(Value::Array(l as usize * r)),
                _ => Err(EvalError::InvalidOperation),
            },
            BinOp::Div => match (lhs, rhs) {
                (Value::Int(l), Value::Int(r)) if r != 0 => Ok(Value::Int(l / r)),
                _ => Err(EvalError::InvalidOperation),
            },
            _ => Err(EvalError::NotSupported),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_liteval(code: &str, expected: Value) {
        let root = typst_syntax::parse_code(code);
        let expr = root.cast::<Code>().unwrap().exprs().next().unwrap();
        assert_eq!(expr.liteval(), Ok(expected), "expr: {expr:#?}");
    }

    #[test]
    fn test_simple_expr() {
        use Value::*;

        test_liteval("none", None);
        test_liteval("auto", Auto);
        test_liteval("0", Int(0));
        test_liteval("1 + 2", Int(3));
        test_liteval("1 * 2", Int(2));
        test_liteval("1 - 2", Int(-1));
        test_liteval("(1 + 2) * 3", Int(9));
        test_liteval("(1fr,)", Array(1));
        test_liteval("(1pt, 2em) * 3", Array(6));
        test_liteval("(1, 2) + (3, 4, 5)", Array(5));
        test_liteval("(1,) * 2 + 2 * (3, 4)", Array(6));
        test_liteval("((1,) * 2 + 2 * (3,)) * 4", Array(16));
    }
}
