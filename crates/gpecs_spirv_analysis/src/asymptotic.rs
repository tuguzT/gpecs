use std::{
    fmt::{self, Display},
    ops::{Add, Div, Mul, Rem, Sub},
};

use rspirv::spirv::Word;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    Log,
    Factorial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Const(f64),
    Id(Word),
    Unary {
        op: UnaryOperator,
        arg: Box<Expr>,
    },
    Binary {
        op: BinaryOperator,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

impl Expr {
    pub fn log(self) -> Self {
        Self::Unary {
            op: UnaryOperator::Log,
            arg: self.into(),
        }
    }

    pub fn factorial(self) -> Self {
        Self::Unary {
            op: UnaryOperator::Factorial,
            arg: self.into(),
        }
    }

    pub fn pow(self, rhs: impl Into<Box<Self>>) -> Self {
        Self::Binary {
            op: BinaryOperator::Pow,
            lhs: self.into(),
            rhs: rhs.into(),
        }
    }

    pub fn simplify(self) -> Self {
        todo!()
    }
}

impl<T> Add<T> for Expr
where
    T: Into<Box<Self>>,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Self::Binary {
            op: BinaryOperator::Add,
            lhs: self.into(),
            rhs: rhs.into(),
        }
    }
}

impl<T> Sub<T> for Expr
where
    T: Into<Box<Self>>,
{
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        Self::Binary {
            op: BinaryOperator::Sub,
            lhs: self.into(),
            rhs: rhs.into(),
        }
    }
}

impl<T> Mul<T> for Expr
where
    T: Into<Box<Self>>,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::Binary {
            op: BinaryOperator::Mul,
            lhs: self.into(),
            rhs: rhs.into(),
        }
    }
}

impl<T> Div<T> for Expr
where
    T: Into<Box<Self>>,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self::Binary {
            op: BinaryOperator::Div,
            lhs: self.into(),
            rhs: rhs.into(),
        }
    }
}

impl<T> Rem<T> for Expr
where
    T: Into<Box<Self>>,
{
    type Output = Self;

    fn rem(self, rhs: T) -> Self::Output {
        Self::Binary {
            op: BinaryOperator::Rem,
            lhs: self.into(),
            rhs: rhs.into(),
        }
    }
}

impl Default for Expr {
    fn default() -> Self {
        Self::Const(1.0)
    }
}

impl From<f64> for Expr {
    fn from(value: f64) -> Self {
        Self::Const(value)
    }
}

impl<T> From<T> for Box<Expr>
where
    f64: From<T>,
{
    fn from(value: T) -> Self {
        let value = f64::from(value);
        Box::new(value.into())
    }
}

// impl PartialOrd for Expr {
//     fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
//         let this = self.clone().simplify();
//         let other = other.clone().simplify();
//         match (this, other) {
//             _ => todo!(),
//         }
//     }
// }

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Const(value) => value.fmt(f),
            Expr::Id(id) => write!(f, "%{id}"),
            Expr::Unary { op, arg } => match op {
                UnaryOperator::Log => write!(f, "log({arg})"),
                UnaryOperator::Factorial => write!(f, "({arg})!"),
            },
            Expr::Binary { op, lhs, rhs } => match op {
                BinaryOperator::Add => write!(f, "({lhs}) + ({rhs})"),
                BinaryOperator::Sub => write!(f, "({lhs}) - ({rhs})"),
                BinaryOperator::Mul => write!(f, "({lhs}) * ({rhs})"),
                BinaryOperator::Div => write!(f, "({lhs}) / ({rhs})"),
                BinaryOperator::Rem => write!(f, "({lhs}) % ({rhs})"),
                BinaryOperator::Pow => write!(f, "({lhs}) ^ ({rhs})"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        let expr = Expr::Id(10);
        assert_eq!("%10", expr.to_string());

        let expr = expr + 1.0;
        assert_eq!("(%10) + (1)", expr.to_string());

        let expr = Expr::Id(1) * 2;
        assert_eq!("(%1) * (2)", expr.to_string());

        let expr = expr / 3;
        assert_eq!("((%1) * (2)) / (3)", expr.to_string());

        let expr = expr % 4;
        assert_eq!("(((%1) * (2)) / (3)) % (4)", expr.to_string());

        let expr = Expr::from(5.0).pow(3.0);
        assert_eq!("(5) ^ (3)", expr.to_string());

        let expr = expr.factorial();
        assert_eq!("((5) ^ (3))!", expr.to_string());

        let expr = Expr::from(2.0).log();
        assert_eq!("log(2)", expr.to_string());

        let expr = Expr::Id(1) - (Expr::from(1.0) - 3.0);
        assert_eq!("(%1) - ((1) - (3))", expr.to_string());
    }
}
