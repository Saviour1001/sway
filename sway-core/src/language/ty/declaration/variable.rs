use std::hash::{Hash, Hasher};

use sway_types::{Ident, Span};

use crate::{engine_threading::*, language::ty::*, type_system::*};

#[derive(Clone, Debug)]
pub struct TyVariableDeclaration {
    pub name: Ident,
    pub body: TyExpression,
    pub mutability: VariableMutability,
    pub return_type: TypeId,
    pub type_ascription: TypeId,
    pub type_ascription_span: Option<Span>,
}

impl EqWithEngines for TyVariableDeclaration {}
impl PartialEqWithEngines for TyVariableDeclaration {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        self.name == other.name
            && self.body.eq(&other.body, type_engine)
            && self.mutability == other.mutability
            && type_engine
                .get(self.return_type)
                .eq(&type_engine.get(other.return_type), type_engine)
            && type_engine
                .get(self.type_ascription)
                .eq(&type_engine.get(other.type_ascription), type_engine)
    }
}

impl HashWithEngines for TyVariableDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        self.name.hash(state);
        self.body.hash(state, type_engine);
        type_engine.get(self.return_type).hash(state, type_engine);
        type_engine
            .get(self.type_ascription)
            .hash(state, type_engine);
        self.mutability.hash(state);
    }
}

impl SubstTypes for TyVariableDeclaration {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.return_type.subst(type_mapping, engines);
        self.type_ascription.subst(type_mapping, engines);
        self.body.subst(type_mapping, engines)
    }
}

impl ReplaceSelfType for TyVariableDeclaration {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.return_type.replace_self_type(engines, self_type);
        self.type_ascription.replace_self_type(engines, self_type);
        self.body.replace_self_type(engines, self_type)
    }
}
