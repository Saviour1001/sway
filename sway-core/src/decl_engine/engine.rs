use hashbrown::hash_map::RawEntryMut;
use hashbrown::HashMap;
use std::{
    collections::{HashSet, VecDeque},
    fmt,
    sync::RwLock,
};

use sway_error::error::CompileError;
use sway_types::Span;

use crate::{
    concurrent_slab::{ConcurrentSlab, ListDisplay},
    decl_engine::*,
    engine_threading::*,
    language::ty,
    TypeEngine,
};

/// Used inside of type inference to store declarations.
#[derive(Debug, Default)]
pub struct DeclEngine {
    slab: ConcurrentSlab<DeclWrapper>,
    id_map: RwLock<HashMap<DeclWrapper, DeclId>>,
    parents: RwLock<HashMap<usize, Vec<DeclId>>>,
}

impl fmt::Display for DeclEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.slab.with_slice(|elems| {
            let list = ListDisplay { list: elems.iter() };
            write!(f, "DeclarationEngine {{\n{list}\n}}")
        })
    }
}

impl DeclEngine {
    pub(crate) fn get(&self, index: DeclId) -> DeclWrapper {
        self.slab.get(*index)
    }

    pub(super) fn replace(&self, index: DeclId, wrapper: DeclWrapper) {
        self.slab.replace(index, wrapper);
    }

    pub(crate) fn insert<T>(&self, type_engine: &TypeEngine, decl: T) -> DeclId
    where
        T: Into<(DeclWrapper, Span)>,
    {
        let (decl_wrapper, span) = decl.into();
        self.insert_wrapper(type_engine, decl_wrapper, span)
    }

    pub(crate) fn insert_wrapper(
        &self,
        type_engine: &TypeEngine,
        decl_wrapper: DeclWrapper,
        span: Span,
    ) -> DeclId {
        let mut id_map = self.id_map.write().unwrap();

        let hash_builder = id_map.hasher().clone();
        let decl_hash = make_hasher(&hash_builder, type_engine)(&decl_wrapper);

        match id_map
            .raw_entry_mut()
            .from_hash(decl_hash, |x| x.eq(&decl_wrapper, type_engine))
        {
            RawEntryMut::Occupied(o) => o.get().clone(),
            RawEntryMut::Vacant(v) => {
                let decl_id = DeclId::new(self.slab.insert(decl_wrapper.clone()), span);
                v.insert_with_hasher(
                    decl_hash,
                    decl_wrapper,
                    decl_id.clone(),
                    make_hasher(&hash_builder, type_engine),
                );
                decl_id
            }
        }
    }

    /// Given a [DeclId] `index`, finds all the parents of `index` and all the
    /// recursive parents of those parents, and so on. Does not perform
    /// duplicated computation---if the parents of a [DeclId] have already been
    /// found, we do not find them again.
    #[allow(clippy::map_entry)]
    pub(crate) fn find_all_parents(&self, engines: Engines<'_>, index: DeclId) -> Vec<DeclId> {
        let parents = self.parents.read().unwrap();
        let mut acc_parents: HashMap<usize, DeclId> = HashMap::new();
        let mut already_checked: HashSet<usize> = HashSet::new();
        let mut left_to_check: VecDeque<DeclId> = VecDeque::from([index]);
        while let Some(curr) = left_to_check.pop_front() {
            if !already_checked.insert(*curr) {
                continue;
            }
            if let Some(curr_parents) = parents.get(&*curr) {
                for curr_parent in curr_parents.iter() {
                    if !acc_parents.contains_key(&**curr_parent) {
                        acc_parents.insert(**curr_parent, curr_parent.clone());
                    }
                    todo!();
                    // if !left_to_check.iter().any(|x| x.eq(curr_parent, engines)) {
                    //     left_to_check.push_back(curr_parent.clone());
                    // }
                }
            }
        }
        acc_parents.values().cloned().collect()
    }

    pub(crate) fn register_parent(&self, index: &DeclId, parent: DeclId) {
        let mut parents = self.parents.write().unwrap();
        parents
            .entry(**index)
            .and_modify(|e| e.push(parent.clone()))
            .or_insert_with(|| vec![parent]);
    }

    pub fn get_function(
        &self,
        index: DeclId,
        span: &Span,
    ) -> Result<ty::TyFunctionDeclaration, CompileError> {
        self.slab.get(*index).expect_function(span)
    }

    pub fn get_trait(
        &self,
        index: DeclId,
        span: &Span,
    ) -> Result<ty::TyTraitDeclaration, CompileError> {
        self.slab.get(*index).expect_trait(span)
    }

    pub fn get_trait_fn(&self, index: DeclId, span: &Span) -> Result<ty::TyTraitFn, CompileError> {
        self.slab.get(*index).expect_trait_fn(span)
    }

    pub fn get_impl_trait(
        &self,
        index: DeclId,
        span: &Span,
    ) -> Result<ty::TyImplTrait, CompileError> {
        self.slab.get(*index).expect_impl_trait(span)
    }

    pub fn get_struct(
        &self,
        index: DeclId,
        span: &Span,
    ) -> Result<ty::TyStructDeclaration, CompileError> {
        self.slab.get(*index).expect_struct(span)
    }

    pub fn get_storage(
        &self,
        index: DeclId,
        span: &Span,
    ) -> Result<ty::TyStorageDeclaration, CompileError> {
        self.slab.get(*index).expect_storage(span)
    }

    pub fn get_abi(
        &self,
        index: DeclId,
        span: &Span,
    ) -> Result<ty::TyAbiDeclaration, CompileError> {
        self.slab.get(*index).expect_abi(span)
    }

    pub fn get_constant(
        &self,
        index: DeclId,
        span: &Span,
    ) -> Result<ty::TyConstantDeclaration, CompileError> {
        self.slab.get(*index).expect_constant(span)
    }

    pub fn get_enum(
        &self,
        index: DeclId,
        span: &Span,
    ) -> Result<ty::TyEnumDeclaration, CompileError> {
        self.slab.get(*index).expect_enum(span)
    }
}
