use sway_core::language::ty::{TyDeclaration, TyImplTrait, TyStructDeclaration, TyStructField};
use sway_types::Spanned;
use tower_lsp::lsp_types::{CodeActionDisabled, Position, Range, Url};

use crate::{
    capabilities::code_actions::{CodeActionContext, CodeActionTrait, CODE_ACTION_NEW_TITLE},
    core::token::TypedAstToken,
};

pub(crate) struct StructNewCodeAction<'a> {
    decl: &'a TyStructDeclaration,
    uri: &'a Url,
    existing_impl_decl: Option<TyImplTrait>,
}

impl<'a> CodeActionTrait<'a, TyStructDeclaration> for StructNewCodeAction<'a> {
    fn new(ctx: CodeActionContext<'a>, decl: &'a TyStructDeclaration) -> Self {
        // Before the other functions are called, we need to determine if the new function
        // should be generated in a new impl block, an existing impl block, or not at all.
        // First, find the first impl block for this struct if it exists.
        let existing_impl_decl = ctx
            .tokens
            .all_references_of_token(ctx.token, ctx.engines.te())
            .find_map(|(_, token)| {
                if let Some(TypedAstToken::TypedDeclaration(TyDeclaration::ImplTrait(decl_id))) =
                    token.typed
                {
                    ctx.engines
                        .de()
                        .get_impl_trait(decl_id.clone(), &decl_id.span())
                        .ok()
                } else {
                    None
                }
            });

        Self {
            decl,
            uri: ctx.uri,
            existing_impl_decl,
        }
    }

    fn new_text(&self) -> String {
        let params = self.params_string(&self.decl.fields);
        let new_fn = self.fn_signature_string(
            "new".to_string(),
            params,
            &self.decl.attributes,
            self.return_type_string(),
            Some(self.fn_body()),
        );

        // If there is already an impl block for this struct, add only the function to it.
        if self.existing_impl_decl.is_some() {
            format!("{new_fn}\n")
        } else {
            // Otherwise, generate the impl block with the `new` function inside.
            self.impl_string(
                self.type_param_string(&self.decl.type_parameters),
                format!("\n{new_fn}\n"),
                None,
            )
        }
    }

    fn range(&self) -> Range {
        // If there is already an impl block for this struct, insert the new function at the top of it.
        if self.existing_impl_decl.is_some() {
            let (first_line, _) = self
                .existing_impl_decl
                .clone()
                .unwrap()
                .span
                .start_pos()
                .line_col();
            let insertion_position = Position {
                line: first_line as u32,
                character: 0,
            };
            Range {
                start: insertion_position,
                end: insertion_position,
            }
        } else {
            // If we're inserting a whole new impl block, default to the line after the struct declaration.
            let (last_line, _) = self.decl().span().end_pos().line_col();
            let insertion_position = Position {
                line: last_line as u32,
                character: 0,
            };
            Range {
                start: insertion_position,
                end: insertion_position,
            }
        }
    }

    fn title(&self) -> String {
        CODE_ACTION_NEW_TITLE.to_string()
    }

    fn decl_name(&self) -> String {
        self.decl.call_path.suffix.to_string()
    }

    fn decl(&self) -> &TyStructDeclaration {
        self.decl
    }

    fn uri(&self) -> &Url {
        self.uri
    }

    fn disabled(&self) -> Option<CodeActionDisabled> {
        // If there is already a `new` function in the impl block, don't generate a new one.
        if self
            .existing_impl_decl
            .clone()?
            .methods
            .iter()
            .any(|method| method.span().as_str().contains("fn new"))
        {
            Some(CodeActionDisabled {
                reason: format!("Struct {} already has a `new` function", self.decl_name()),
            })
        } else {
            None
        }
    }
}

impl StructNewCodeAction<'_> {
    fn return_type_string(&self) -> String {
        " -> Self".to_string()
    }

    fn params_string(&self, params: &[TyStructField]) -> String {
        params
            .iter()
            .map(|field| format!("{}: {}", field.name, field.type_span.as_str()))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn fn_body(&self) -> String {
        format!(
            "Self {{ {} }}",
            self.decl
                .fields
                .iter()
                .map(|field| format!("{}", field.name))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
