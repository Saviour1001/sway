mod constraint;
mod gather_constraints;
mod namespace;
mod priv_prelude;

use crate::{language::ty, CompileResult, Engines};

use priv_prelude::*;

pub(super) fn monomorphize(engines: Engines<'_>, module: &mut ty::TyModule) -> CompileResult<()> {
    CompileResult::with_handler(|h| {
        let mut root_namespace = Namespace::new_from_root((&module.namespace).into());
        let ctx = Context::from_root(&mut root_namespace, engines);
        gather_constraints(ctx, h, module)?;
        todo!()
    })
}
