mod constraint;
mod context;
mod gather;
mod priv_prelude;
mod solve;

use std::sync::RwLock;

use hashbrown::HashMap;

use crate::{engine_threading::*, language::ty, CompileResult};

use priv_prelude::*;

pub(super) fn monomorphize(engines: Engines<'_>, module: &mut ty::TyModule) -> CompileResult<()> {
    CompileResult::with_handler(|h| {
        let root_namespace = Namespace::init_root(&module.namespace);
        let constraints = RwLock::new(HashMap::new());
        let ctx = Context::from_root(&root_namespace, engines, &constraints);
        gather_constraints(ctx, h, module)?;
        println!("{:#?}", constraints.read().unwrap());
        todo!()
    })
}
