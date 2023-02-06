use crate::{monomorphize::priv_prelude::*, namespace};

/// The set of items that represent the namespace context passed throughout
/// gathering the monomorphization constraints.
pub(crate) struct Namespace<'a> {
    /// An absolute path from the `root` that represents the current module
    /// being gathered.
    pub(crate) mod_path: PathBuf,

    /// The `root` of the project namespace.
    pub(crate) root: &'a mut namespace::Module,
}

impl<'a> Namespace<'a> {
    /// Initialize the namespace at its root from the given initial namespace.
    pub(crate) fn new_from_root(root: &'a mut namespace::Module) -> Self {
        let mod_path = vec![];
        Self { root, mod_path }
    }

    /// A reference to the path of the module where constraints are currently
    /// being gathered.
    pub(crate) fn mod_path(&self) -> &Path {
        &self.mod_path
    }

    /// Access to the current [namespace::Module], i.e. the module at the inner `mod_path`.
    pub(crate) fn module(&self) -> &namespace::Module {
        &self.root[&self.mod_path]
    }

    /// Mutable access to the current [namespace::Module], i.e. the module at the inner
    /// `mod_path`.
    pub(crate) fn module_mut(&mut self) -> &mut namespace::Module {
        &mut self.root[&self.mod_path]
    }
}

impl<'a> std::ops::Deref for Namespace<'a> {
    type Target = namespace::Module;
    fn deref(&self) -> &Self::Target {
        self.module()
    }
}

impl<'a> std::ops::DerefMut for Namespace<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.module_mut()
    }
}
