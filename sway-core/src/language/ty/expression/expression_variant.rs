use std::{
    collections::HashMap,
    fmt::{self, Write},
    hash::{Hash, Hasher},
};

use sway_types::{state::StateIndex, Ident, Span};

use crate::{
    decl_engine::*,
    engine_threading::*,
    language::{ty::*, *},
    type_system::*,
};

#[derive(Clone, Debug)]
pub enum TyExpressionVariant {
    Literal(Literal),
    FunctionApplication {
        call_path: CallPath,
        contract_call_params: HashMap<String, TyExpression>,
        arguments: Vec<(Ident, TyExpression)>,
        function_decl_id: DeclId,
        function_type_subst_list: TypeSubstList,
        /// If this is `Some(val)` then `val` is the metadata. If this is `None`, then
        /// there is no selector.
        self_state_idx: Option<StateIndex>,
        selector: Option<ContractCallParams>,
        /// optional binding information for the LSP
        type_binding: Option<TypeBinding<()>>,
    },
    LazyOperator {
        op: LazyOp,
        lhs: Box<TyExpression>,
        rhs: Box<TyExpression>,
    },
    VariableExpression {
        name: Ident,
        span: Span,
        mutability: VariableMutability,
    },
    Tuple {
        fields: Vec<TyExpression>,
    },
    Array {
        contents: Vec<TyExpression>,
    },
    ArrayIndex {
        prefix: Box<TyExpression>,
        index: Box<TyExpression>,
    },
    StructExpression {
        struct_name: Ident,
        fields: Vec<TyStructExpressionField>,
        span: Span,
        type_binding: TypeBinding<()>,
    },
    CodeBlock(TyCodeBlock),
    // a flag that this value will later be provided as a parameter, but is currently unknown
    FunctionParameter,
    IfExp {
        condition: Box<TyExpression>,
        then: Box<TyExpression>,
        r#else: Option<Box<TyExpression>>,
    },
    AsmExpression {
        registers: Vec<TyAsmRegisterDeclaration>,
        body: Vec<AsmOp>,
        returns: Option<(AsmRegister, Span)>,
        whole_block_span: Span,
    },
    // like a variable expression but it has multiple parts,
    // like looking up a field in a struct
    StructFieldAccess {
        prefix: Box<TyExpression>,
        field_to_access: TyStructField,
        field_instantiation_span: Span,
        resolved_type_of_parent: TypeId,
    },
    TupleElemAccess {
        prefix: Box<TyExpression>,
        elem_to_access_num: usize,
        resolved_type_of_parent: TypeId,
        elem_to_access_span: Span,
    },
    EnumInstantiation {
        // Type of this enum, including its definition and fields. We expect
        // this to be the [TypeInfo][Enum] variant.
        type_id: TypeId,
        /// for printing
        variant_name: Ident,
        tag: usize,
        contents: Option<Box<TyExpression>>,
        /// If there is an error regarding this instantiation of the enum,
        /// use these spans as it points to the call site and not the declaration.
        /// They are also used in the language server.
        enum_instantiation_span: Span,
        variant_instantiation_span: Span,
        type_binding: TypeBinding<()>,
    },
    AbiCast {
        abi_name: CallPath,
        address: Box<TyExpression>,
        #[allow(dead_code)]
        // this span may be used for errors in the future, although it is not right now.
        span: Span,
    },
    StorageAccess(TyStorageAccess),
    IntrinsicFunction(TyIntrinsicFunctionKind),
    /// a zero-sized type-system-only compile-time thing that is used for constructing ABI casts.
    AbiName(AbiName),
    /// grabs the enum tag from the particular enum and variant of the `exp`
    EnumTag {
        exp: Box<TyExpression>,
    },
    /// performs an unsafe cast from the `exp` to the type of the given enum `variant`
    UnsafeDowncast {
        exp: Box<TyExpression>,
        variant: TyEnumVariant,
    },
    WhileLoop {
        condition: Box<TyExpression>,
        body: TyCodeBlock,
    },
    Break,
    Continue,
    Reassignment(Box<TyReassignment>),
    StorageReassignment(Box<TyStorageReassignment>),
    Return(Box<TyExpression>),
}

impl EqWithEngines for TyExpressionVariant {}
impl PartialEqWithEngines for TyExpressionVariant {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        match (self, other) {
            (Self::Literal(l0), Self::Literal(r0)) => l0 == r0,
            (
                Self::FunctionApplication {
                    call_path: l_name,
                    arguments: l_arguments,
                    function_decl_id: l_function_decl_id,
                    ..
                },
                Self::FunctionApplication {
                    call_path: r_name,
                    arguments: r_arguments,
                    function_decl_id: r_function_decl_id,
                    ..
                },
            ) => {
                l_name == r_name
                    && l_arguments.len() == r_arguments.len()
                    && l_arguments
                        .iter()
                        .zip(r_arguments.iter())
                        .all(|((xa, xb), (ya, yb))| xa == ya && xb.eq(yb, type_engine))
                    && l_function_decl_id == r_function_decl_id
            }
            (
                Self::LazyOperator {
                    op: l_op,
                    lhs: l_lhs,
                    rhs: l_rhs,
                },
                Self::LazyOperator {
                    op: r_op,
                    lhs: r_lhs,
                    rhs: r_rhs,
                },
            ) => {
                l_op == r_op
                    && (**l_lhs).eq(&(**r_lhs), type_engine)
                    && (**l_rhs).eq(&(**r_rhs), type_engine)
            }
            (
                Self::VariableExpression {
                    name: l_name,
                    span: l_span,
                    mutability: l_mutability,
                },
                Self::VariableExpression {
                    name: r_name,
                    span: r_span,
                    mutability: r_mutability,
                },
            ) => l_name == r_name && l_span == r_span && l_mutability == r_mutability,
            (Self::Tuple { fields: l_fields }, Self::Tuple { fields: r_fields }) => {
                l_fields.eq(r_fields, type_engine)
            }
            (
                Self::Array {
                    contents: l_contents,
                },
                Self::Array {
                    contents: r_contents,
                },
            ) => l_contents.eq(r_contents, type_engine),
            (
                Self::ArrayIndex {
                    prefix: l_prefix,
                    index: l_index,
                },
                Self::ArrayIndex {
                    prefix: r_prefix,
                    index: r_index,
                },
            ) => {
                (**l_prefix).eq(&**r_prefix, type_engine) && (**l_index).eq(&**r_index, type_engine)
            }
            (
                Self::StructExpression {
                    struct_name: l_struct_name,
                    fields: l_fields,
                    span: l_span,
                    type_binding: l_type_binding,
                },
                Self::StructExpression {
                    struct_name: r_struct_name,
                    fields: r_fields,
                    span: r_span,
                    type_binding: r_type_binding,
                },
            ) => {
                l_struct_name == r_struct_name
                    && l_fields.eq(r_fields, type_engine)
                    && l_span == r_span
                    && l_type_binding.eq(r_type_binding, type_engine)
            }
            (Self::CodeBlock(l0), Self::CodeBlock(r0)) => l0.eq(r0, type_engine),
            (
                Self::IfExp {
                    condition: l_condition,
                    then: l_then,
                    r#else: l_r,
                },
                Self::IfExp {
                    condition: r_condition,
                    then: r_then,
                    r#else: r_r,
                },
            ) => {
                (**l_condition).eq(&**r_condition, type_engine)
                    && (**l_then).eq(&**r_then, type_engine)
                    && if let (Some(l), Some(r)) = (l_r, r_r) {
                        (**l).eq(&**r, type_engine)
                    } else {
                        true
                    }
            }
            (
                Self::AsmExpression {
                    registers: l_registers,
                    body: l_body,
                    returns: l_returns,
                    ..
                },
                Self::AsmExpression {
                    registers: r_registers,
                    body: r_body,
                    returns: r_returns,
                    ..
                },
            ) => {
                l_registers.eq(r_registers, type_engine)
                    && l_body.clone() == r_body.clone()
                    && l_returns == r_returns
            }
            (
                Self::StructFieldAccess {
                    prefix: l_prefix,
                    field_to_access: l_field_to_access,
                    resolved_type_of_parent: l_resolved_type_of_parent,
                    ..
                },
                Self::StructFieldAccess {
                    prefix: r_prefix,
                    field_to_access: r_field_to_access,
                    resolved_type_of_parent: r_resolved_type_of_parent,
                    ..
                },
            ) => {
                (**l_prefix).eq(&**r_prefix, type_engine)
                    && l_field_to_access.eq(r_field_to_access, type_engine)
                    && type_engine
                        .get(*l_resolved_type_of_parent)
                        .eq(&type_engine.get(*r_resolved_type_of_parent), type_engine)
            }
            (
                Self::TupleElemAccess {
                    prefix: l_prefix,
                    elem_to_access_num: l_elem_to_access_num,
                    resolved_type_of_parent: l_resolved_type_of_parent,
                    ..
                },
                Self::TupleElemAccess {
                    prefix: r_prefix,
                    elem_to_access_num: r_elem_to_access_num,
                    resolved_type_of_parent: r_resolved_type_of_parent,
                    ..
                },
            ) => {
                (**l_prefix).eq(&**r_prefix, type_engine)
                    && l_elem_to_access_num == r_elem_to_access_num
                    && type_engine
                        .get(*l_resolved_type_of_parent)
                        .eq(&type_engine.get(*r_resolved_type_of_parent), type_engine)
            }
            (Self::EnumInstantiation { .. }, Self::EnumInstantiation { .. }) => {
                todo!()
                // l_enum_decl.eq(r_enum_decl, engines)
                //     && l_variant_name == r_variant_name
                //     && l_tag == r_tag
                //     && if let (Some(l_contents), Some(r_contents)) = (l_contents, r_contents) {
                //         (**l_contents).eq(&**r_contents, engines)
                //     } else {
                //         true
                //     }
            }
            (
                Self::AbiCast {
                    abi_name: l_abi_name,
                    address: l_address,
                    ..
                },
                Self::AbiCast {
                    abi_name: r_abi_name,
                    address: r_address,
                    ..
                },
            ) => l_abi_name == r_abi_name && (**l_address).eq(&**r_address, type_engine),
            (Self::IntrinsicFunction(l_kind), Self::IntrinsicFunction(r_kind)) => {
                l_kind.eq(r_kind, type_engine)
            }
            (
                Self::UnsafeDowncast {
                    exp: l_exp,
                    variant: l_variant,
                },
                Self::UnsafeDowncast {
                    exp: r_exp,
                    variant: r_variant,
                },
            ) => l_exp.eq(r_exp, type_engine) && l_variant.eq(r_variant, type_engine),
            (Self::EnumTag { exp: l_exp }, Self::EnumTag { exp: r_exp }) => {
                l_exp.eq(&**r_exp, type_engine)
            }
            (Self::StorageAccess(l_exp), Self::StorageAccess(r_exp)) => {
                l_exp.eq(r_exp, type_engine)
            }
            (
                Self::WhileLoop {
                    body: l_body,
                    condition: l_condition,
                },
                Self::WhileLoop {
                    body: r_body,
                    condition: r_condition,
                },
            ) => l_body.eq(r_body, type_engine) && l_condition.eq(r_condition, type_engine),
            _ => false,
        }
    }
}

impl HashWithEngines for TyExpressionVariant {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        match self {
            Self::Literal(lit) => {
                state.write_u8(1);
                lit.hash(state);
            }
            Self::FunctionApplication {
                call_path,
                arguments,
                function_decl_id,
                ..
            } => {
                state.write_u8(2);
                call_path.hash(state);
                function_decl_id.hash(state);
                arguments.iter().for_each(|(name, arg)| {
                    name.hash(state);
                    arg.hash(state, type_engine);
                });
            }
            Self::LazyOperator { op, lhs, rhs } => {
                state.write_u8(3);
                op.hash(state);
                lhs.hash(state, type_engine);
                rhs.hash(state, type_engine);
            }
            Self::VariableExpression {
                name, mutability, ..
            } => {
                state.write_u8(4);
                name.hash(state);
                mutability.hash(state);
            }
            Self::Tuple { fields } => {
                state.write_u8(5);
                fields.hash(state, type_engine);
            }
            Self::Array { contents } => {
                state.write_u8(6);
                contents.hash(state, type_engine);
            }
            Self::ArrayIndex { prefix, index } => {
                state.write_u8(7);
                prefix.hash(state, type_engine);
                index.hash(state, type_engine);
            }
            Self::StructExpression {
                struct_name,
                fields,
                ..
            } => {
                state.write_u8(8);
                struct_name.hash(state);
                fields.hash(state, type_engine);
            }
            Self::CodeBlock(contents) => {
                state.write_u8(9);
                contents.hash(state, type_engine);
            }
            Self::FunctionParameter => {
                state.write_u8(10);
            }
            Self::IfExp {
                condition,
                then,
                r#else,
            } => {
                state.write_u8(11);
                condition.hash(state, type_engine);
                then.hash(state, type_engine);
                r#else.as_ref().map(|x| x.hash(state, type_engine));
            }
            Self::AsmExpression {
                registers,
                body,
                returns,
                ..
            } => {
                state.write_u8(12);
                registers.hash(state, type_engine);
                body.hash(state);
                returns.hash(state);
            }
            Self::StructFieldAccess {
                prefix,
                field_to_access,
                resolved_type_of_parent,
                ..
            } => {
                state.write_u8(13);
                prefix.hash(state, type_engine);
                field_to_access.hash(state, type_engine);
                type_engine
                    .get(*resolved_type_of_parent)
                    .hash(state, type_engine);
            }
            Self::TupleElemAccess {
                prefix,
                elem_to_access_num,
                resolved_type_of_parent,
                ..
            } => {
                state.write_u8(14);
                prefix.hash(state, type_engine);
                elem_to_access_num.hash(state);
                type_engine
                    .get(*resolved_type_of_parent)
                    .hash(state, type_engine);
            }
            Self::EnumInstantiation { .. } => {
                todo!();
                // state.write_u8(15);
                // enum_decl.hash(state, type_engine);
                // variant_name.hash(state);
                // tag.hash(state);
                // contents.map(|x| x.hash(state, type_engine));
            }
            Self::AbiCast {
                abi_name, address, ..
            } => {
                state.write_u8(16);
                abi_name.hash(state);
                address.hash(state, type_engine);
            }
            Self::StorageAccess(exp) => {
                state.write_u8(17);
                exp.hash(state, type_engine);
            }
            Self::IntrinsicFunction(exp) => {
                state.write_u8(18);
                exp.hash(state, type_engine);
            }
            Self::AbiName(name) => {
                state.write_u8(19);
                name.hash(state);
            }
            Self::EnumTag { exp } => {
                state.write_u8(20);
                exp.hash(state, type_engine);
            }
            Self::UnsafeDowncast { exp, variant } => {
                state.write_u8(21);
                exp.hash(state, type_engine);
                variant.hash(state, type_engine);
            }
            Self::WhileLoop { condition, body } => {
                state.write_u8(22);
                condition.hash(state, type_engine);
                body.hash(state, type_engine);
            }
            Self::Break => {
                state.write_u8(23);
            }
            Self::Continue => {
                state.write_u8(24);
            }
            Self::Reassignment(exp) => {
                state.write_u8(25);
                exp.hash(state, type_engine);
            }
            Self::StorageReassignment(exp) => {
                state.write_u8(26);
                exp.hash(state, type_engine);
            }
            Self::Return(exp) => {
                state.write_u8(27);
                exp.hash(state, type_engine);
            }
        }
    }
}

impl SubstTypes for TyExpressionVariant {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        use TyExpressionVariant::*;
        match self {
            Literal(..) => (),
            FunctionApplication {
                arguments,
                ref mut function_decl_id,
                ..
            } => {
                arguments
                    .iter_mut()
                    .for_each(|(_ident, expr)| expr.subst(type_mapping, engines));
                let new_decl_id = function_decl_id
                    .clone()
                    .subst_types_and_insert_new(type_mapping, engines);
                function_decl_id.replace_id(*new_decl_id);
            }
            LazyOperator { lhs, rhs, .. } => {
                (*lhs).subst(type_mapping, engines);
                (*rhs).subst(type_mapping, engines);
            }
            VariableExpression { .. } => (),
            Tuple { fields } => fields
                .iter_mut()
                .for_each(|x| x.subst(type_mapping, engines)),
            Array { contents } => contents
                .iter_mut()
                .for_each(|x| x.subst(type_mapping, engines)),
            ArrayIndex { prefix, index } => {
                (*prefix).subst(type_mapping, engines);
                (*index).subst(type_mapping, engines);
            }
            StructExpression { fields, .. } => fields
                .iter_mut()
                .for_each(|x| x.subst(type_mapping, engines)),
            CodeBlock(block) => {
                block.subst(type_mapping, engines);
            }
            FunctionParameter => (),
            IfExp {
                condition,
                then,
                r#else,
            } => {
                condition.subst(type_mapping, engines);
                then.subst(type_mapping, engines);
                if let Some(ref mut r#else) = r#else {
                    r#else.subst(type_mapping, engines);
                }
            }
            AsmExpression {
                registers, //: Vec<TyAsmRegisterDeclaration>,
                ..
            } => {
                registers
                    .iter_mut()
                    .for_each(|x| x.subst(type_mapping, engines));
            }
            // like a variable expression but it has multiple parts,
            // like looking up a field in a struct
            StructFieldAccess {
                prefix,
                field_to_access,
                ref mut resolved_type_of_parent,
                ..
            } => {
                resolved_type_of_parent.subst(type_mapping, engines);
                field_to_access.subst(type_mapping, engines);
                prefix.subst(type_mapping, engines);
            }
            TupleElemAccess {
                prefix,
                ref mut resolved_type_of_parent,
                ..
            } => {
                resolved_type_of_parent.subst(type_mapping, engines);
                prefix.subst(type_mapping, engines);
            }
            EnumInstantiation { .. } => {
                todo!();
                // enum_decl.subst(type_mapping, engines);
                // if let Some(ref mut contents) = contents {
                //     contents.subst(type_mapping, engines)
                // };
            }
            AbiCast { address, .. } => address.subst(type_mapping, engines),
            // storage is never generic and cannot be monomorphized
            StorageAccess { .. } => (),
            IntrinsicFunction(kind) => {
                kind.subst(type_mapping, engines);
            }
            EnumTag { exp } => {
                exp.subst(type_mapping, engines);
            }
            UnsafeDowncast { exp, variant } => {
                exp.subst(type_mapping, engines);
                variant.subst(type_mapping, engines);
            }
            AbiName(_) => (),
            WhileLoop {
                ref mut condition,
                ref mut body,
            } => {
                condition.subst(type_mapping, engines);
                body.subst(type_mapping, engines);
            }
            Break => (),
            Continue => (),
            Reassignment(reassignment) => reassignment.subst(type_mapping, engines),
            StorageReassignment(..) => (),
            Return(stmt) => stmt.subst(type_mapping, engines),
        }
    }
}

impl ReplaceSelfType for TyExpressionVariant {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        use TyExpressionVariant::*;
        match self {
            Literal(..) => (),
            FunctionApplication {
                arguments,
                ref mut function_decl_id,
                ..
            } => {
                arguments
                    .iter_mut()
                    .for_each(|(_ident, expr)| expr.replace_self_type(engines, self_type));
                let new_decl_id = function_decl_id
                    .clone()
                    .replace_self_type_and_insert_new(engines, self_type);
                function_decl_id.replace_id(*new_decl_id);
            }
            LazyOperator { lhs, rhs, .. } => {
                (*lhs).replace_self_type(engines, self_type);
                (*rhs).replace_self_type(engines, self_type);
            }
            VariableExpression { .. } => (),
            Tuple { fields } => fields
                .iter_mut()
                .for_each(|x| x.replace_self_type(engines, self_type)),
            Array { contents } => contents
                .iter_mut()
                .for_each(|x| x.replace_self_type(engines, self_type)),
            ArrayIndex { prefix, index } => {
                (*prefix).replace_self_type(engines, self_type);
                (*index).replace_self_type(engines, self_type);
            }
            StructExpression { fields, .. } => fields
                .iter_mut()
                .for_each(|x| x.replace_self_type(engines, self_type)),
            CodeBlock(block) => {
                block.replace_self_type(engines, self_type);
            }
            FunctionParameter => (),
            IfExp {
                condition,
                then,
                r#else,
            } => {
                condition.replace_self_type(engines, self_type);
                then.replace_self_type(engines, self_type);
                if let Some(ref mut r#else) = r#else {
                    r#else.replace_self_type(engines, self_type);
                }
            }
            AsmExpression { registers, .. } => {
                registers
                    .iter_mut()
                    .for_each(|x| x.replace_self_type(engines, self_type));
            }
            StructFieldAccess {
                prefix,
                field_to_access,
                ref mut resolved_type_of_parent,
                ..
            } => {
                resolved_type_of_parent.replace_self_type(engines, self_type);
                field_to_access.replace_self_type(engines, self_type);
                prefix.replace_self_type(engines, self_type);
            }
            TupleElemAccess {
                prefix,
                ref mut resolved_type_of_parent,
                ..
            } => {
                resolved_type_of_parent.replace_self_type(engines, self_type);
                prefix.replace_self_type(engines, self_type);
            }
            EnumInstantiation { .. } => {
                todo!();
                // enum_decl.replace_self_type(engines, self_type);
                // if let Some(ref mut contents) = contents {
                //     contents.replace_self_type(engines, self_type)
                // };
            }
            AbiCast { address, .. } => address.replace_self_type(engines, self_type),
            StorageAccess { .. } => (),
            IntrinsicFunction(kind) => {
                kind.replace_self_type(engines, self_type);
            }
            EnumTag { exp } => {
                exp.replace_self_type(engines, self_type);
            }
            UnsafeDowncast { exp, variant } => {
                exp.replace_self_type(engines, self_type);
                variant.replace_self_type(engines, self_type);
            }
            AbiName(_) => (),
            WhileLoop {
                ref mut condition,
                ref mut body,
            } => {
                condition.replace_self_type(engines, self_type);
                body.replace_self_type(engines, self_type);
            }
            Break => (),
            Continue => (),
            Reassignment(reassignment) => reassignment.replace_self_type(engines, self_type),
            StorageReassignment(..) => (),
            Return(stmt) => stmt.replace_self_type(engines, self_type),
        }
    }
}

impl DisplayWithEngines for TyExpressionVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result {
        let s = match self {
            TyExpressionVariant::Literal(lit) => format!("literal {lit}"),
            TyExpressionVariant::FunctionApplication {
                call_path: name, ..
            } => {
                format!("\"{}\" fn entry", name.suffix.as_str())
            }
            TyExpressionVariant::LazyOperator { op, .. } => match op {
                LazyOp::And => "&&".into(),
                LazyOp::Or => "||".into(),
            },
            TyExpressionVariant::Tuple { fields } => {
                let fields = fields
                    .iter()
                    .map(|field| engines.help_out(field).to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("tuple({fields})")
            }
            TyExpressionVariant::Array { .. } => "array".into(),
            TyExpressionVariant::ArrayIndex { .. } => "[..]".into(),
            TyExpressionVariant::StructExpression { struct_name, .. } => {
                format!("\"{}\" struct init", struct_name.as_str())
            }
            TyExpressionVariant::CodeBlock(_) => "code block entry".into(),
            TyExpressionVariant::FunctionParameter => "fn param access".into(),
            TyExpressionVariant::IfExp { .. } => "if exp".into(),
            TyExpressionVariant::AsmExpression { .. } => "inline asm".into(),
            TyExpressionVariant::AbiCast { abi_name, .. } => {
                format!("abi cast {}", abi_name.suffix.as_str())
            }
            TyExpressionVariant::StructFieldAccess {
                resolved_type_of_parent,
                field_to_access,
                ..
            } => {
                format!(
                    "\"{}.{}\" struct field access",
                    engines.help_out(*resolved_type_of_parent),
                    field_to_access.name
                )
            }
            TyExpressionVariant::TupleElemAccess {
                resolved_type_of_parent,
                elem_to_access_num,
                ..
            } => {
                format!(
                    "\"{}.{}\" tuple index",
                    engines.help_out(*resolved_type_of_parent),
                    elem_to_access_num
                )
            }
            TyExpressionVariant::VariableExpression { name, .. } => {
                format!("\"{}\" variable exp", name.as_str())
            }
            TyExpressionVariant::EnumInstantiation { .. } => {
                todo!();
                // format!(
                //     "{}::{} enum instantiation (tag: {})",
                //     enum_decl.name.as_str(),
                //     variant_name.as_str(),
                //     tag
                // )
            }
            TyExpressionVariant::StorageAccess(access) => {
                format!("storage field {} access", access.storage_field_name())
            }
            TyExpressionVariant::IntrinsicFunction(kind) => engines.help_out(kind).to_string(),
            TyExpressionVariant::AbiName(n) => format!("ABI name {n}"),
            TyExpressionVariant::EnumTag { exp } => {
                format!("({} as tag)", engines.help_out(exp.return_type))
            }
            TyExpressionVariant::UnsafeDowncast { exp, variant } => {
                format!(
                    "({} as {})",
                    engines.help_out(exp.return_type),
                    variant.name
                )
            }
            TyExpressionVariant::WhileLoop { condition, .. } => {
                format!("while loop on {}", engines.help_out(&**condition))
            }
            TyExpressionVariant::Break => "break".to_string(),
            TyExpressionVariant::Continue => "continue".to_string(),
            TyExpressionVariant::Reassignment(reassignment) => {
                let mut place = reassignment.lhs_base_name.to_string();
                for index in &reassignment.lhs_indices {
                    place.push('.');
                    match index {
                        ProjectionKind::StructField { name } => place.push_str(name.as_str()),
                        ProjectionKind::TupleField { index, .. } => {
                            write!(&mut place, "{index}").unwrap();
                        }
                        ProjectionKind::ArrayIndex { index, .. } => {
                            write!(&mut place, "{index:#?}").unwrap();
                        }
                    }
                }
                format!("reassignment to {place}")
            }
            TyExpressionVariant::StorageReassignment(storage_reassignment) => {
                let place: String = {
                    storage_reassignment
                        .fields
                        .iter()
                        .map(|field| field.name.as_str())
                        .collect()
                };
                format!("storage reassignment to {place}")
            }
            TyExpressionVariant::Return(exp) => {
                format!("return {}", engines.help_out(&**exp))
            }
        };
        write!(f, "{s}")
    }
}

impl TyExpressionVariant {
    /// Returns `self` as a literal, if possible.
    pub(crate) fn extract_literal_value(&self) -> Option<Literal> {
        match self {
            TyExpressionVariant::Literal(value) => Some(value.clone()),
            _ => None,
        }
    }

    /// recurse into `self` and get any return statements -- used to validate that all returns
    /// do indeed return the correct type
    /// This does _not_ extract implicit return statements as those are not control flow! This is
    /// _only_ for explicit returns.
    pub(crate) fn gather_return_statements(&self) -> Vec<&TyExpression> {
        match self {
            TyExpressionVariant::IfExp {
                condition,
                then,
                r#else,
            } => {
                let mut buf = condition.gather_return_statements();
                buf.append(&mut then.gather_return_statements());
                if let Some(ref r#else) = r#else {
                    buf.append(&mut r#else.gather_return_statements());
                }
                buf
            }
            TyExpressionVariant::CodeBlock(TyCodeBlock { contents, .. }) => {
                let mut buf = vec![];
                for node in contents {
                    buf.append(&mut node.gather_return_statements())
                }
                buf
            }
            TyExpressionVariant::WhileLoop { condition, body } => {
                let mut buf = condition.gather_return_statements();
                for node in &body.contents {
                    buf.append(&mut node.gather_return_statements())
                }
                buf
            }
            TyExpressionVariant::Reassignment(reassignment) => {
                reassignment.rhs.gather_return_statements()
            }
            TyExpressionVariant::StorageReassignment(storage_reassignment) => {
                storage_reassignment.rhs.gather_return_statements()
            }
            TyExpressionVariant::LazyOperator { lhs, rhs, .. } => [lhs, rhs]
                .into_iter()
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TyExpressionVariant::Tuple { fields } => fields
                .iter()
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TyExpressionVariant::Array { contents } => contents
                .iter()
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TyExpressionVariant::ArrayIndex { prefix, index } => [prefix, index]
                .into_iter()
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TyExpressionVariant::StructFieldAccess { prefix, .. } => {
                prefix.gather_return_statements()
            }
            TyExpressionVariant::TupleElemAccess { prefix, .. } => {
                prefix.gather_return_statements()
            }
            TyExpressionVariant::EnumInstantiation { contents, .. } => contents
                .iter()
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TyExpressionVariant::AbiCast { address, .. } => address.gather_return_statements(),
            TyExpressionVariant::IntrinsicFunction(intrinsic_function_kind) => {
                intrinsic_function_kind
                    .arguments
                    .iter()
                    .flat_map(|expr| expr.gather_return_statements())
                    .collect()
            }
            TyExpressionVariant::StructExpression { fields, .. } => fields
                .iter()
                .flat_map(|field| field.value.gather_return_statements())
                .collect(),
            TyExpressionVariant::FunctionApplication {
                contract_call_params,
                arguments,
                selector,
                ..
            } => contract_call_params
                .values()
                .chain(arguments.iter().map(|(_name, expr)| expr))
                .chain(
                    selector
                        .iter()
                        .map(|contract_call_params| &*contract_call_params.contract_address),
                )
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TyExpressionVariant::EnumTag { exp } => exp.gather_return_statements(),
            TyExpressionVariant::UnsafeDowncast { exp, .. } => exp.gather_return_statements(),

            TyExpressionVariant::Return(exp) => {
                vec![exp]
            }
            // if it is impossible for an expression to contain a return _statement_ (not an
            // implicit return!), put it in the pattern below.
            TyExpressionVariant::Literal(_)
            | TyExpressionVariant::FunctionParameter { .. }
            | TyExpressionVariant::AsmExpression { .. }
            | TyExpressionVariant::VariableExpression { .. }
            | TyExpressionVariant::AbiName(_)
            | TyExpressionVariant::StorageAccess { .. }
            | TyExpressionVariant::Break
            | TyExpressionVariant::Continue => vec![],
        }
    }
}
