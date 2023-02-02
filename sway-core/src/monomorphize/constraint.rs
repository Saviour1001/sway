use sway_types::Ident;

use crate::type_system::*;

#[derive(Debug, Clone)]
pub(crate) enum Constraint {
    VarDecl(Ident, TypeInfo),
}
