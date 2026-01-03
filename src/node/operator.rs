use core::str::FromStr;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) enum BinaryOp {
    Assign, // =
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    Rem,    // %
    Shl,    // <<
    Shr,    // >>
    BitAnd, // &
    BitXor, // ^
    BitOr,  // |
    Eq,     // ==
    Ne,     // !=
    Lt,     // <
    Le,     // <=
}

impl FromStr for BinaryOp {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // assign operators
            "=" => Ok(BinaryOp::Assign),
            "*=" => Ok(BinaryOp::Mul),
            "/=" => Ok(BinaryOp::Div),
            "%=" => Ok(BinaryOp::Rem),
            "+=" => Ok(BinaryOp::Add),
            "-=" => Ok(BinaryOp::Sub),
            "<<=" => Ok(BinaryOp::Shl),
            ">>=" => Ok(BinaryOp::Shr),
            "&=" => Ok(BinaryOp::BitAnd),
            "^=" => Ok(BinaryOp::BitXor),
            "|=" => Ok(BinaryOp::BitOr),
            _ => Err(()),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) enum UnaryOp {
    BitNot,     // ~
    LogicalNot, // !
    Addr,       // &
    Deref,      // *
    PreInc,     // ++pre
    PreDec,     // --pre
    PostInc,    // post++
    PostDec,    // post--
}
