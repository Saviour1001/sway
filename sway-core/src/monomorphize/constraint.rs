use sway_types::Ident;

use crate::type_system::*;

pub(crate) enum Constraint {
    VarDecl(Ident, TypeInfo),
}
