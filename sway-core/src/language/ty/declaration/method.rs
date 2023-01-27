use std::hash::{Hash, Hasher};

use sway_types::Ident;

use crate::{decl_engine::DeclId, engine_threading::*, type_system::*, TypeSubstList};

#[derive(Clone, Debug)]
pub struct TyMethodValue {
    pub name: Ident,
    pub decl_id: DeclId,
    pub type_subst_list: TypeSubstList,
}

impl TyMethodValue {
    pub(crate) fn new(
        name: Ident,
        decl_id: DeclId,
        type_subst_list: TypeSubstList,
    ) -> TyMethodValue {
        TyMethodValue {
            name,
            decl_id,
            type_subst_list,
        }
    }
}

impl EqWithEngines for TyMethodValue {}
impl PartialEqWithEngines for TyMethodValue {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        self.decl_id == other.decl_id
            && self.type_subst_list.eq(&other.type_subst_list, type_engine)
    }
}

impl HashWithEngines for TyMethodValue {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        self.decl_id.hash(state);
        self.type_subst_list.hash(state, type_engine);
    }
}

impl SubstTypes for TyMethodValue {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.type_subst_list.subst(type_mapping, engines);
    }
}

impl ReplaceSelfType for TyMethodValue {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.type_subst_list.replace_self_type(engines, self_type);
    }
}
