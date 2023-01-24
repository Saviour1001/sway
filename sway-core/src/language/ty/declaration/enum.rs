use std::hash::{Hash, Hasher};

use sway_error::error::CompileError;
use sway_types::{Ident, Span, Spanned};

use crate::{engine_threading::*, error::*, language::Visibility, transform, type_system::*};

#[derive(Clone, Debug)]
pub struct TyEnumDeclaration {
    pub name: Ident,
    pub type_parameters: Vec<TypeParameter>,
    pub attributes: transform::AttributesMap,
    pub variants: Vec<TyEnumVariant>,
    pub span: Span,
    pub visibility: Visibility,
}

impl EqWithEngines for TyEnumDeclaration {}
impl PartialEqWithEngines for TyEnumDeclaration {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        self.name == other.name
            && self.type_parameters.eq(&other.type_parameters, type_engine)
            && self.variants.eq(&other.variants, type_engine)
            && self.visibility == other.visibility
    }
}

impl HashWithEngines for TyEnumDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        self.name.hash(state);
        self.variants.hash(state, type_engine);
        self.type_parameters.hash(state, type_engine);
        self.visibility.hash(state);
    }
}

impl SubstTypes for TyEnumDeclaration {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.variants
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
    }
}

impl FinalizeTypes for TyEnumDeclaration {
    fn finalize_inner(&mut self, engines: Engines<'_>, subst_list: &TypeSubstList) {
        self.variants
            .iter_mut()
            .for_each(|x| x.finalize(engines, subst_list));
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.finalize(engines, subst_list));
    }
}

impl ReplaceSelfType for TyEnumDeclaration {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.variants
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
    }
}

impl Spanned for TyEnumDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl MonomorphizeHelper for TyEnumDeclaration {
    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.name
    }
}

impl TyEnumDeclaration {
    pub(crate) fn expect_variant_from_name(
        &self,
        variant_name: &Ident,
    ) -> CompileResult<&TyEnumVariant> {
        let warnings = vec![];
        let mut errors = vec![];
        match self
            .variants
            .iter()
            .find(|x| x.name.as_str() == variant_name.as_str())
        {
            Some(variant) => ok(variant, warnings, errors),
            None => {
                errors.push(CompileError::UnknownEnumVariant {
                    enum_name: self.name.clone(),
                    variant_name: variant_name.clone(),
                    span: variant_name.span(),
                });
                err(warnings, errors)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TyEnumVariant {
    pub name: Ident,
    pub type_id: TypeId,
    pub initial_type_id: TypeId,
    pub type_span: Span,
    pub(crate) tag: usize,
    pub span: Span,
    pub attributes: transform::AttributesMap,
}

impl HashWithEngines for TyEnumVariant {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        self.name.hash(state);
        type_engine.get(self.type_id).hash(state, type_engine);
        self.tag.hash(state);
    }
}

impl EqWithEngines for TyEnumVariant {}
impl PartialEqWithEngines for TyEnumVariant {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        self.name == other.name
            && type_engine
                .get(self.type_id)
                .eq(&type_engine.get(other.type_id), type_engine)
            && self.tag == other.tag
    }
}

impl SubstTypes for TyEnumVariant {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.type_id.subst(type_mapping, engines);
    }
}

impl FinalizeTypes for TyEnumVariant {
    fn finalize_inner(&mut self, engines: Engines<'_>, subst_list: &TypeSubstList) {
        self.type_id.finalize(engines, subst_list);
    }
}

impl ReplaceSelfType for TyEnumVariant {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.type_id.replace_self_type(engines, self_type);
    }
}
