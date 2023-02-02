pub(crate) mod context;
pub(crate) mod declaration;
pub(crate) mod expression;
pub(crate) mod module;
pub(crate) mod node;

use std::cell::RefCell;

use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*, Engines};

use self::module::gather_from_root;

pub(super) fn gather_constraints(
    engines: Engines<'_>,
    handler: &Handler,
    module: &ty::TyModule,
) -> Result<Vec<Constraint>, ErrorEmitted> {
    let root_namespace = Namespace::init_root(&module.namespace);
    let constraints = RefCell::new(vec![]);
    let ctx = Context::from_root(&root_namespace, engines, &constraints);
    gather_from_root(ctx, handler, module)?;
    Ok(constraints.into_inner())
}
