use std::hash::{Hash, Hasher};

use sway_types::{Span, Spanned};

use crate::{engine_threading::*, type_system::*};

use super::DeclMapping;

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

impl DeclId {
    pub(crate) fn new(index: usize, span: Span) -> DeclId {
        DeclId(index, span)
    }

    pub(crate) fn replace_id(&mut self, index: usize) {
        self.0 = index;
    }

    pub(crate) fn subst_types_and_insert_new(
        &self,
        type_mapping: &TypeSubstMap,
        engines: Engines<'_>,
    ) -> DeclId {
        todo!()
        // let type_engine = engines.te();
        // let decl_engine = engines.de();
        // let mut decl = decl_engine.get(self.clone());
        // decl.subst(type_mapping, engines);
        // decl_engine
        //     .insert_wrapper(type_engine, decl, self.1.clone())
        //     .with_parent(decl_engine, self.clone())
    }

    pub(crate) fn replace_self_type_and_insert_new(
        &self,
        engines: Engines<'_>,
        self_type: TypeId,
    ) -> DeclId {
        todo!()
        // let type_engine = engines.te();
        // let decl_engine = engines.de();
        // let mut decl = decl_engine.get(self.clone());
        // decl.replace_self_type(engines, self_type);
        // decl_engine
        //     .insert_wrapper(type_engine, decl, self.1.clone())
        //     .with_parent(decl_engine, self.clone())
    }

    pub(crate) fn replace_decls_and_insert_new(
        &self,
        decl_mapping: &DeclMapping,
        engines: Engines<'_>,
    ) -> DeclId {
        todo!()
        // let type_engine = engines.te();
        // let decl_engine = engines.de();
        // let mut decl = decl_engine.get(self.clone());
        // decl.replace_decls(decl_mapping, engines);
        // decl_engine
        //     .insert_wrapper(type_engine, decl, self.1.clone())
        //     .with_parent(decl_engine, self.clone())
    }
}
