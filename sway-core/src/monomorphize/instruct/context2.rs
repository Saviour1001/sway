use sway_types::Ident;

use crate::{
    decl_engine::*, engine_threading::*, monomorphize::priv_prelude::*, namespace, TypeEngine,
};

type PathBuf = Vec<Ident>;

/// Contextual state tracked and accumulated throughout applying the
/// monomorphization instructions.
pub(crate) struct InstructContext<'a, 'b> {
    /// The namespace context accumulated throughout type-checking.
    pub(crate) namespace: &'a mut Namespace<'b>,

    /// The type engine storing types.
    pub(crate) type_engine: &'a TypeEngine,

    /// The declaration engine holds declarations.
    pub(crate) decl_engine: &'a DeclEngine,
}

impl<'a, 'b> InstructContext<'a, 'b> {
    /// Initialize a context at the top-level of a module with its namespace.
    pub(crate) fn from_root(
        root_namespace: &'a mut Namespace<'b>,
        engines: Engines<'a>,
    ) -> InstructContext<'a, 'b> {
        Self::from_module_namespace(root_namespace, engines)
    }

    fn from_module_namespace(namespace: &'a mut Namespace<'b>, engines: Engines<'a>) -> Self {
        let (type_engine, decl_engine) = engines.unwrap();
        Self {
            namespace,
            type_engine,
            decl_engine,
        }
    }

    /// Create a new context that mutably borrows the inner [Namespace] with a
    /// lifetime bound by `self`.
    pub(crate) fn by_ref(&mut self) -> InstructContext<'_, '_> {
        InstructContext {
            namespace: self.namespace,
            type_engine: self.type_engine,
            decl_engine: self.decl_engine,
        }
    }

    /// Scope the [InstructContext] with the given [Namespace].
    pub(crate) fn scoped(self, namespace: &'a mut Namespace<'b>) -> InstructContext<'a, 'b> {
        InstructContext {
            namespace,
            type_engine: self.type_engine,
            decl_engine: self.decl_engine,
        }
    }
}

/// The set of items that represent the namespace context passed throughout
/// gathering the trait constraints.
#[derive(Debug)]
pub(crate) struct Namespace<'a> {
    /// The `root` of the project namespace.
    pub(crate) root: &'a mut namespace::Module,

    /// An absolute path from the `root` that represents the current module
    /// being gathered.
    pub(crate) mod_path: PathBuf,
}

impl<'a> Namespace<'a> {
    /// Initialize the namespace at its root from the given initial namespace.
    pub(crate) fn init_root(root: &'a mut namespace::Module) -> Namespace<'a> {
        let mod_path = vec![];
        Self { root, mod_path }
    }

    pub(crate) fn new_with_module(&mut self, module: &mut namespace::Module) -> Namespace<'_> {
        let mut mod_path = self.mod_path.clone();
        if let Some(name) = &module.name {
            mod_path.push(name.clone());
        }
        Namespace {
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
