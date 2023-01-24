use std::hash::{Hash, Hasher};

use sway_types::{Span, Spanned};

use crate::{
    engine_threading::*,
    language::ty,
    type_system::{SubstTypes, TypeSubstMap},
    ReplaceSelfType, TypeId,
};

use super::{DeclEngine, DeclMapping, ReplaceDecls, ReplaceFunctionImplementingType};

/// An ID used to refer to an item in the [DeclarationEngine](super::decl_engine::DeclarationEngine)
#[derive(Debug)]
pub struct DeclId(usize, Span);

impl Clone for DeclId {
    fn clone(&self) -> Self {
        Self(self.0, self.1.clone())
    }
}

impl PartialEq for DeclId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Hash for DeclId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl std::ops::Deref for DeclId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(clippy::from_over_into)]
impl Into<usize> for DeclId {
    fn into(self) -> usize {
        self.0
    }
}

impl Spanned for DeclId {
    fn span(&self) -> Span {
        self.1.clone()
    }
}

impl SubstTypes for DeclId {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self.clone());
        decl.subst(type_mapping, engines);
        decl_engine.replace(self.clone(), decl);
    }
}

impl ReplaceSelfType for DeclId {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self.clone());
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(self.clone(), decl);
    }
}

impl ReplaceDecls for DeclId {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>) {
        let decl_engine = engines.de();
        if let Some(new_decl_id) = decl_mapping.find_match(self) {
            self.0 = *new_decl_id;
            return;
        }
        let all_parents = decl_engine.find_all_parents(engines, self.clone());
        for parent in all_parents.into_iter() {
            if let Some(new_decl_id) = decl_mapping.find_match(&parent) {
                self.0 = *new_decl_id;
                return;
            }
        }
    }
}

impl ReplaceFunctionImplementingType for DeclId {
    fn replace_implementing_type(
        &mut self,
        engines: Engines<'_>,
        implementing_type: ty::TyDeclaration,
    ) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self.clone());
        decl.replace_implementing_type(engines, implementing_type);
        decl_engine.replace(self.clone(), decl);
    }
}

impl DeclId {
    pub(crate) fn new(index: usize, span: Span) -> DeclId {
        DeclId(index, span)
    }

    pub(crate) fn with_parent(self, decl_engine: &DeclEngine, parent: DeclId) -> DeclId {
        decl_engine.register_parent(&self, parent);
        self
    }

    pub(crate) fn replace_id(&mut self, index: usize) {
        self.0 = index;
    }

    pub(crate) fn subst_types_and_insert_new(
        &self,
        type_mapping: &TypeSubstMap,
        engines: Engines<'_>,
    ) -> DeclId {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self.clone());
        decl.subst(type_mapping, engines);
        decl_engine
            .insert_wrapper(type_engine, decl, self.1.clone())
            .with_parent(decl_engine, self.clone())
    }

    pub(crate) fn replace_self_type_and_insert_new(
        &self,
        engines: Engines<'_>,
        self_type: TypeId,
    ) -> DeclId {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self.clone());
        decl.replace_self_type(engines, self_type);
        decl_engine
            .insert_wrapper(type_engine, decl, self.1.clone())
            .with_parent(decl_engine, self.clone())
    }

    pub(crate) fn replace_decls_and_insert_new(
        &self,
        decl_mapping: &DeclMapping,
        engines: Engines<'_>,
    ) -> DeclId {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self.clone());
        decl.replace_decls(decl_mapping, engines);
        decl_engine
            .insert_wrapper(type_engine, decl, self.1.clone())
            .with_parent(decl_engine, self.clone())
    }
}
