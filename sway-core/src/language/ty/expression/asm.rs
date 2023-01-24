use std::hash::{Hash, Hasher};

use sway_types::Ident;

use crate::{engine_threading::*, language::ty::*, type_system::*};

#[derive(Clone, Debug)]
pub struct TyAsmRegisterDeclaration {
    pub(crate) initializer: Option<TyExpression>,
    pub(crate) name: Ident,
}

impl PartialEqWithEngines for TyAsmRegisterDeclaration {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        self.name == other.name
            && if let (Some(l), Some(r)) = (&self.initializer, &other.initializer) {
                l.eq(r, type_engine)
            } else {
                true
            }
    }
}

impl HashWithEngines for TyAsmRegisterDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        self.name.hash(state);
        self.initializer
            .as_ref()
            .map(|x| x.hash(state, type_engine));
    }
}

impl SubstTypes for TyAsmRegisterDeclaration {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        if let Some(ref mut initializer) = self.initializer {
            initializer.subst(type_mapping, engines)
        }
    }
}

impl ReplaceSelfType for TyAsmRegisterDeclaration {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        if let Some(ref mut initializer) = self.initializer {
            initializer.replace_self_type(engines, self_type)
        }
    }
}
