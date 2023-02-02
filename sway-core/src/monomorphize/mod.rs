mod constraint;
mod gather_constraints;
// mod namespace;
mod context;
mod priv_prelude;

use std::cell::RefCell;

use crate::{language::ty, CompileResult, Engines};

use priv_prelude::*;

pub(super) fn monomorphize(engines: Engines<'_>, module: &mut ty::TyModule) -> CompileResult<()> {
    CompileResult::with_handler(|h| {
        let root_namespace = Namespace::init_root(&module.namespace);
        let constraints = RefCell::new(vec![]);
        let ctx = Context::from_root(&root_namespace, engines, &constraints);
        gather_constraints(ctx, h, module)?;
        println!("{:#?}", constraints.borrow());
        todo!()
    })
}
