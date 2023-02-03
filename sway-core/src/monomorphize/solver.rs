use sway_error::handler::{ErrorEmitted, Handler};

use crate::{decl_engine::DeclEngine, monomorphize::priv_prelude::*, Engines, TypeEngine};

pub(crate) struct Solver<'a> {
    /// The type engine storing types.
    pub(crate) type_engine: &'a TypeEngine,

    /// The declaration engine holds declarations.
    pub(crate) decl_engine: &'a DeclEngine,

    pub(crate) constraints: Vec<Constraint>,
}

impl<'a> Solver<'a> {
    pub(crate) fn new(engines: Engines<'a>, constraints: Vec<Constraint>) -> Solver<'a> {
        let (type_engine, decl_engine) = engines.unwrap();
        Solver {
            type_engine,
            decl_engine,
            constraints,
        }
    }

    pub(crate) fn solve(&mut self, handler: &Handler) -> Result<Instructions, ErrorEmitted> {
        todo!()
    }
}
