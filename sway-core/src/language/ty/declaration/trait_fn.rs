use std::hash::{Hash, Hasher};

use sway_types::{Ident, Span};

use crate::{
    engine_threading::*,
    language::{ty::*, Purity},
    transform,
    type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyTraitFn {
    pub name: Ident,
    pub(crate) purity: Purity,
    pub parameters: Vec<TyFunctionParameter>,
    pub return_type: TypeId,
    pub return_type_span: Span,
    pub attributes: transform::AttributesMap,
}

impl EqWithEngines for TyTraitFn {}
impl PartialEqWithEngines for TyTraitFn {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        self.name == other.name
            && self.purity == other.purity
            && self.parameters.eq(&other.parameters, type_engine)
            && self.return_type == other.return_type
            && self.attributes == other.attributes
    }
}

impl HashWithEngines for TyTraitFn {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        self.name.hash(state);
        self.parameters.hash(state, type_engine);
        type_engine.get(self.return_type).hash(state, type_engine);
        self.purity.hash(state);
    }
}

impl SubstTypes for TyTraitFn {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.parameters
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
        self.return_type.subst(type_mapping, engines);
    }
}

impl MonomorphizeHelper for TyTraitFn {
    fn name(&self) -> &Ident {
        &self.name
    }

    fn type_parameters(&self) -> &[TypeParameter] {
        &[]
    }
}
