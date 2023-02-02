use crate::monomorphize::priv_prelude::*;

/// A namespace session type representing the constraint gathering of a
/// submodule.
///
/// This type allows for re-using the parent's [Namespace] in order to provide
/// access to the `root` and `init` throughout gathering constraints of the submodule,
/// but with an updated `mod_path` to represent the submodule's path. When
/// dropped, the [SubmoduleNamespace] reset's the [Namespace]'s `mod_path` to
/// the parent module path so that gathering constraints of the parent may
/// continue.
pub(crate) struct SubmoduleNamespace<'a, 'b: 'a> {
    pub(crate) namespace: &'a mut Namespace<'b>,
    pub(crate) parent_mod_path: PathBuf,
}

impl<'a, 'b> std::ops::Deref for SubmoduleNamespace<'a, 'b> {
    type Target = Namespace<'b>;
    fn deref(&self) -> &Self::Target {
        self.namespace
    }
}

impl<'a, 'b> std::ops::DerefMut for SubmoduleNamespace<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.namespace
    }
}

impl<'a, 'b> Drop for SubmoduleNamespace<'a, 'b> {
    fn drop(&mut self) {
        self.namespace.mod_path = std::mem::take(&mut self.parent_mod_path);
    }
}
