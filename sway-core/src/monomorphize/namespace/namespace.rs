use std::fmt;

use sway_types::Ident;

use crate::monomorphize::priv_prelude::*;

/// The set of items that represent the namespace context passed throughout
/// gathering the monomorphization constraints.
#[derive(Clone)]
pub(crate) struct Namespace<'a> {
    /// An absolute path from the `root` that represents the current module
    /// being gathered.
    pub(crate) mod_path: PathBuf,

    /// The `root` of the project namespace.
    pub(crate) root: Root<'a>,
}

impl<'a> Namespace<'a> {
    /// Initialize the namespace at its root from the given initial namespace.
    pub(crate) fn new_from_root(root: Module<'a>) -> Self {
        let mod_path = vec![];
        Self {
            root: Root::from(root),
            mod_path,
        }
    }

    /// A reference to the path of the module where constraints are currently
    /// being gathered.
    pub(crate) fn mod_path(&self) -> &Path {
        &self.mod_path
    }

    /// Access to the current [Module], i.e. the module at the inner `mod_path`.
    pub(crate) fn module(&self) -> &Module<'a> {
        &self.root.module[&self.mod_path]
    }

    /// Mutable access to the current [Module], i.e. the module at the inner
    /// `mod_path`.
    pub(crate) fn module_mut(&mut self) -> &mut Module<'a> {
        &mut self.root.module[&self.mod_path]
    }

    /// "Enter" the submodule at the given path by returning a new
    /// [SubmoduleNamespace].
    ///
    /// Here we temporarily change `mod_path` to the given `dep_mod_path` and
    /// wrap `self` in a [SubmoduleNamespace] type. When dropped, the
    /// [SubmoduleNamespace] resets the `mod_path` back to the original path so
    /// that we can continue type-checking the current module after finishing
    /// with the dependency.
    pub(crate) fn enter_submodule<'b>(&'b mut self, dep_name: Ident) -> SubmoduleNamespace<'b, 'a>
    where
        'a: 'b,
    {
        let submod_path: Vec<_> = self
            .mod_path
            .iter()
            .cloned()
            .chain(Some(dep_name))
            .collect();
        let parent_mod_path = std::mem::replace(&mut self.mod_path, submod_path);
        SubmoduleNamespace {
            namespace: self,
            parent_mod_path,
        }
    }
}

impl<'a> std::ops::Deref for Namespace<'a> {
    type Target = Module<'a>;
    fn deref(&self) -> &Self::Target {
        self.module()
    }
}

impl<'a> std::ops::DerefMut for Namespace<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.module_mut()
    }
}

impl fmt::Debug for Namespace<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.root)
    }
}
