use sway_types::Ident;

use crate::{
    decl_engine::{DeclEngine, DeclId},
    language::ty::*,
    language::DepName,
    semantic_analysis::namespace,
};

#[derive(Clone, Debug)]
pub struct TyModule {
    pub submodules: Vec<(DepName, TySubmodule)>,
    pub namespace: namespace::Module,
    pub all_nodes: Vec<TyAstNode>,
}

#[derive(Clone, Debug)]
pub struct TySubmodule {
    pub library_name: Ident,
    pub module: TyModule,
}

/// Iterator type for iterating over submodules.
///
/// Used rather than `impl Iterator` to enable recursive submodule iteration.
pub struct SubmodulesRecursive<'module> {
    submods: std::slice::Iter<'module, (DepName, TySubmodule)>,
    current: Option<(
        &'module (DepName, TySubmodule),
        Box<SubmodulesRecursive<'module>>,
    )>,
}

// /// Iterator type for iterating over modules mutably depth-first.
// ///
// /// Used rather than `impl Iterator` to enable recursive submodule iteration.
// //
// // NOTE: https://aloso.github.io/2021/03/09/creating-an-iterator
// pub struct SubmodulesRecursiveMut<'module> {
//     children: &'module mut [TyModule],
//     parent: Option<Box<SubmodulesRecursiveMut<'module>>>,
// }

impl TyModule {
    /// An iterator yielding all submodules recursively, depth-first.
    pub fn submodules_recursive(&self) -> SubmodulesRecursive {
        SubmodulesRecursive {
            submods: self.submodules.iter(),
            current: None,
        }
    }

    // pub fn submodules_recursive_mut(&mut self) -> SubmodulesRecursiveMut {
    //     SubmodulesRecursiveMut {
    //         children: std::slice::from_mut(self),
    //         parent: None,
    //     }
    // }

    /// All test functions within this module.
    pub fn test_fns<'a: 'b, 'b>(
        &'b self,
        decl_engine: &'a DeclEngine,
    ) -> impl '_ + Iterator<Item = (TyFunctionDeclaration, DeclId)> {
        self.all_nodes.iter().filter_map(|node| {
            if let TyAstNodeContent::Declaration(TyDeclaration::FunctionDeclaration(
                ref decl_id,
                _,
            )) = node.content
            {
                let fn_decl = decl_engine
                    .get_function(decl_id.clone(), &node.span)
                    .expect("no function declaration for ID");
                if fn_decl.is_test() {
                    return Some((fn_decl, decl_id.clone()));
                }
            }
            None
        })
    }
}

impl<'module> Iterator for SubmodulesRecursive<'module> {
    type Item = &'module (DepName, TySubmodule);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.current = match self.current.take() {
                None => match self.submods.next() {
                    None => return None,
                    Some(submod) => {
                        Some((submod, Box::new(submod.1.module.submodules_recursive())))
                    }
                },
                Some((submod, mut submods)) => match submods.next() {
                    Some(next) => {
                        self.current = Some((submod, submods));
                        return Some(next);
                    }
                    None => return Some(submod),
                },
            }
        }
    }
}

// impl<'module> Iterator for SubmodulesRecursiveMut<'module> {
//     type Item = &'module mut TyModule;

//     fn next(&mut self) -> Option<Self::Item> {
//         match self.children.get(0) {
//             None => match self.parent.take() {
//                 Some(parent) => {
//                     *self = *parent;
//                     self.next()
//                 }
//                 None => None,
//             },
//             Some(module) if module.submodules.is_empty() => {
//                 if self.children.is_empty() {
//                     None
//                 } else {
//                     let (first, rest) = self.children.split_first_mut().unwrap();
//                     self.children = rest;
//                     Some(first)
//                 }
//             }
//             Some(module) => todo!(),
//         }
//     }
// }
