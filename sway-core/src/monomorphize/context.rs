use std::{cell::RefCell, sync::Arc};

use sway_types::Ident;

use crate::{decl_engine::*, monomorphize::priv_prelude::*, namespace, Engines, TypeEngine};

type PathBuf = Vec<Ident>;

/// Contextual state tracked and accumulated throughout gathering the trait
/// constraints.
pub(crate) struct Context<'a> {
    /// The namespace context accumulated throughout type-checking.
    pub(crate) namespace: &'a Namespace<'a>,

    /// The type engine storing types.
    pub(crate) type_engine: &'a TypeEngine,

    /// The declaration engine holds declarations.
    pub(crate) decl_engine: &'a DeclEngine,

    /// The list of constraints.
    constraints: &'a RefCell<Vec<Constraint>>,
}

impl<'a> Context<'a> {
    /// Initialize a context at the top-level of a module with its namespace.
    pub(crate) fn from_root(
        root_namespace: &'a Namespace<'a>,
        engines: Engines<'a>,
        constraints: &'a RefCell<Vec<Constraint>>,
    ) -> Context<'a> {
        Self::from_module_namespace(root_namespace, engines, constraints)
    }

    fn from_module_namespace(
        namespace: &'a Namespace<'a>,
        engines: Engines<'a>,
        constraints: &'a RefCell<Vec<Constraint>>,
    ) -> Self {
        let (type_engine, decl_engine) = engines.unwrap();
        Self {
            namespace,
            type_engine,
            decl_engine,
            constraints,
        }
    }

    /// Create a new context that mutably borrows the inner [Namespace] with a
    /// lifetime bound by `self`.
    pub(crate) fn by_ref(&mut self) -> Context<'_> {
        Context {
            namespace: self.namespace,
            type_engine: self.type_engine,
            decl_engine: self.decl_engine,
            constraints: self.constraints,
        }
    }

    /// Scope the [Context] with the given [Namespace].
    pub(crate) fn scoped(self, namespace: &'a Namespace<'a>) -> Context<'a> {
        Context {
            namespace,
            type_engine: self.type_engine,
            decl_engine: self.decl_engine,
            constraints: self.constraints,
        }
    }
}

/// The set of items that represent the namespace context passed throughout
/// gathering the trait constraints.
#[derive(Clone, Debug)]
pub(crate) struct Namespace<'a> {
    /// An immutable namespace that consists of the names that should always be
    /// present, no matter what module or scope we are currently checking.
    init: &'a namespace::Module,

    /// The `root` of the project namespace.
    pub(crate) root: &'a namespace::Module,

    /// An absolute path from the `root` that represents the current module
    /// being gathered.
    pub(crate) mod_path: PathBuf,
}

impl<'a> Namespace<'a> {
    /// Initialize the namespace at its root from the given initial namespace.
    pub(crate) fn init_root(init: &'a namespace::Module) -> Namespace<'a> {
        let mod_path = vec![];
        Self {
            init,
            root: init,
            mod_path,
        }
    }

    pub(crate) fn new_with_module(&self, module: &'a namespace::Module) -> Namespace<'a> {
        let mut mod_path = self.mod_path.clone();
        if let Some(name) = &module.name {
            mod_path.push(name.clone());
        }
        Namespace {
            init: module,
            root: self.root,
            mod_path,
        }
    }

    /// Access to the current [Module], i.e. the module at the inner `mod_path`.
    pub(crate) fn module(&self) -> &namespace::Module {
        &self.root[&self.mod_path]
    }
}

impl<'a> std::ops::Deref for Namespace<'a> {
    type Target = namespace::Module;
    fn deref(&self) -> &Self::Target {
        self.module()
    }
}
