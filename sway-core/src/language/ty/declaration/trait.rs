use std::hash::{Hash, Hasher};

use sway_types::{Ident, Span};

use crate::{
    engine_threading::*,
    language::{parsed, ty::*, Visibility},
    transform,
    type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyTraitDeclaration {
    pub name: Ident,
    pub type_parameters: Vec<TypeParameter>,
    pub interface_surface: Vec<TyMethodValue>,
    pub methods: Vec<TyMethodValue>,
    pub supertraits: Vec<parsed::Supertrait>,
    pub visibility: Visibility,
    pub attributes: transform::AttributesMap,
    pub span: Span,
}

impl EqWithEngines for TyTraitDeclaration {}
impl PartialEqWithEngines for TyTraitDeclaration {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        self.name == other.name
            && self.type_parameters.eq(&other.type_parameters, type_engine)
            && self
                .interface_surface
                .eq(&other.interface_surface, type_engine)
            && self.methods.eq(&other.interface_surface, type_engine)
            && self.supertraits == other.supertraits
            && self.visibility == other.visibility
            && self.attributes == other.attributes
    }
}

impl HashWithEngines for TyTraitDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        self.name.hash(state);
        self.type_parameters.hash(state, type_engine);
        self.interface_surface.hash(state, type_engine);
        self.methods.hash(state, type_engine);
        self.supertraits.hash(state);
        self.visibility.hash(state);
    }
}

impl SubstTypes for TyTraitDeclaration {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        todo!();
        // self.type_parameters
        //     .iter_mut()
        //     .for_each(|x| x.subst(type_mapping, engines));
        // self.interface_surface
        //     .iter_mut()
        //     .for_each(|function_decl_id| {
        //         let new_decl_id = function_decl_id
        //             .clone()
        //             .subst_types_and_insert_new(type_mapping, engines);
        //         function_decl_id.replace_id(*new_decl_id);
        //     });
        // // we don't have to type check the methods because it hasn't been type checked yet
    }
}

impl ReplaceSelfType for TyTraitDeclaration {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        todo!();
        // self.type_parameters
        //     .iter_mut()
        //     .for_each(|x| x.replace_self_type(engines, self_type));
        // self.interface_surface
        //     .iter_mut()
        //     .for_each(|function_decl_id| {
        //         let new_decl_id = function_decl_id
        //             .clone()
        //             .replace_self_type_and_insert_new(engines, self_type);
        //         function_decl_id.replace_id(*new_decl_id);
        //     });
        // // we don't have to type check the methods because it hasn't been type checked yet
    }
}

impl MonomorphizeHelper for TyTraitDeclaration {
    fn name(&self) -> &Ident {
        &self.name
    }

    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }
}
