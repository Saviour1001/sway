use std::hash::{Hash, Hasher};

use sway_types::Span;

use crate::{
    decl_engine::DeclId,
    engine_threading::*,
    language::{ty::*, CallPath},
    type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyImplTrait {
    pub impl_type_parameters: Vec<TypeParameter>,
    pub trait_name: CallPath,
    pub trait_type_arguments: Vec<TypeArgument>,
    pub methods: Vec<TyMethodValue>,
    pub implementing_for_type_id: TypeId,
    pub trait_decl_id: Option<DeclId>,
    pub type_implementing_for_span: Span,
    pub span: Span,
}

impl EqWithEngines for TyImplTrait {}
impl PartialEqWithEngines for TyImplTrait {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        self.impl_type_parameters
            .eq(&other.impl_type_parameters, type_engine)
            && self.trait_name == other.trait_name
            && self
                .trait_type_arguments
                .eq(&other.trait_type_arguments, type_engine)
            && self.methods.eq(&other.methods, type_engine)
            && type_engine.get(self.implementing_for_type_id).eq(
                &type_engine.get(other.implementing_for_type_id),
                type_engine,
            )
    }
}

impl HashWithEngines for TyImplTrait {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        self.trait_name.hash(state);
        self.impl_type_parameters.hash(state, type_engine);
        self.trait_type_arguments.hash(state, type_engine);
        self.methods.hash(state, type_engine);
        type_engine
            .get(self.implementing_for_type_id)
            .hash(state, type_engine);
    }
}

impl SubstTypes for TyImplTrait {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.impl_type_parameters
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
        self.implementing_for_type_id.subst(type_mapping, engines);
        self.methods
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
    }
}

impl ReplaceSelfType for TyImplTrait {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.impl_type_parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
        self.implementing_for_type_id
            .replace_self_type(engines, self_type);
        self.methods
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
    }
}
