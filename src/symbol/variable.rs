use crate::node::Node;

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct Variable {
    pub(crate) symbol_id: usize,
    pub(crate) init: Vec<Node>,
}
