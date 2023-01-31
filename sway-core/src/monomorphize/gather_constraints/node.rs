use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*, Engines};

pub(crate) fn gather_from_node(
    handler: &Handler,
    engines: Engines<'_>,
    node: &ty::TyAstNodeContent,
) -> Result<Vec<Constraint>, ErrorEmitted> {
    match node {
        ty::TyAstNodeContent::Declaration(decl) => gather_from_decl(handler, engines, decl),
        ty::TyAstNodeContent::Expression(exp) => gather_from_exp(handler, engines, exp),
        ty::TyAstNodeContent::ImplicitReturnExpression(exp) => {
            gather_from_exp(handler, engines, exp)
        }
        ty::TyAstNodeContent::SideEffect => Ok(vec![]),
    }
}
