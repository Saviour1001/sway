//! Code to validate the IR in a [`Context`].
//!
//! During creation, deserialization and optimization the IR should be verified to be in a
//! consistent valid state, using the functions in this module.

use crate::{
    block::BlockContent,
    context::Context,
    error::IrError,
    function::{Function, FunctionContent},
    instruction::{FuelVmInstruction, Instruction, Predicate},
    irtype::Type,
    metadata::{MetadataIndex, Metadatum},
    module::ModuleContent,
    pointer::Pointer,
    value::{Value, ValueDatum},
    BinaryOpKind, BlockArgument, BranchToWithArgs, TypeContent,
};

impl Context {
    /// Verify the contents of this [`Context`] is valid.
    pub fn verify(self) -> Result<Self, IrError> {
        for (_, module) in &self.modules {
            self.verify_module(module)?;
        }
        Ok(self)
    }

    fn verify_module(&self, module: &ModuleContent) -> Result<(), IrError> {
        for function in &module.functions {
            self.verify_function(module, &self.functions[function.0])?;
        }
        Ok(())
    }

    fn verify_function(
        &self,
        cur_module: &ModuleContent,
        function: &FunctionContent,
    ) -> Result<(), IrError> {
        for block in &function.blocks {
            self.verify_block(cur_module, function, &self.blocks[block.0])?;
        }
        self.verify_metadata(function.metadata)?;
        Ok(())
    }

    fn verify_block(
        &self,
        cur_module: &ModuleContent,
        cur_function: &FunctionContent,
        block: &BlockContent,
    ) -> Result<(), IrError> {
        if block.instructions.len() <= 1 && block.preds.is_empty() {
            // Empty unreferenced blocks are a harmless artefact.
            return Ok(());
        }

        for (arg_idx, arg_val) in block.args.iter().enumerate() {
            match self.values[arg_val.0].value {
                ValueDatum::Argument(BlockArgument { idx, .. }) if idx == arg_idx => (),
                _ => return Err(IrError::VerifyBlockArgMalformed),
            }
        }

        InstructionVerifier {
            context: self,
            cur_module,
            cur_function,
            cur_block: block,
        }
        .verify_instructions()?;

        let (last_is_term, num_terms) =
            block.instructions.iter().fold((false, 0), |(_, n), ins| {
                if ins.is_terminator(self) {
                    (true, n + 1)
                } else {
                    (false, n)
                }
            });
        if !last_is_term {
            Err(IrError::MissingTerminator(block.label.clone()))
        } else if num_terms != 1 {
            Err(IrError::MisplacedTerminator(block.label.clone()))
        } else {
            Ok(())
        }
    }

    fn verify_metadata(&self, md_idx: Option<MetadataIndex>) -> Result<(), IrError> {
        // For now we check only that struct tags are valid identiers.
        if let Some(md_idx) = md_idx {
            match &self.metadata[md_idx.0] {
                Metadatum::List(md_idcs) => {
                    for md_idx in md_idcs {
                        self.verify_metadata(Some(*md_idx))?;
                    }
                }
                Metadatum::Struct(tag, ..) => {
                    // We could import Regex to match it, but it's a simple identifier style pattern:
                    // alpha start char, alphanumeric for the rest, or underscore anywhere.
                    if tag.is_empty() {
                        return Err(IrError::InvalidMetadatum(
                            "Struct has empty tag.".to_owned(),
                        ));
                    }
                    let mut chs = tag.chars();
                    let ch0 = chs.next().unwrap();
                    if !(ch0.is_ascii_alphabetic() || ch0 == '_')
                        || chs.any(|ch| !(ch.is_ascii_alphanumeric() || ch == '_'))
                    {
                        return Err(IrError::InvalidMetadatum(format!(
                            "Invalid struct tag: '{tag}'."
                        )));
                    }
                }
                _otherwise => (),
            }
        }
        Ok(())
    }
}

struct InstructionVerifier<'a> {
    context: &'a Context,
    cur_module: &'a ModuleContent,
    cur_function: &'a FunctionContent,
    cur_block: &'a BlockContent,
}

impl<'a> InstructionVerifier<'a> {
    fn verify_instructions(&self) -> Result<(), IrError> {
        for ins in &self.cur_block.instructions {
            let value_content = &self.context.values[ins.0];
            if let ValueDatum::Instruction(instruction) = &value_content.value {
                match instruction {
                    Instruction::AddrOf(arg, _ty) => self.verify_addr_of(arg)?,
                    Instruction::AsmBlock(..) => (),
                    Instruction::BitCast(value, ty) => self.verify_bitcast(value, ty)?,
                    Instruction::BinaryOp { op, arg1, arg2 } => {
                        self.verify_binary_op(op, arg1, arg2)?
                    }
                    Instruction::Branch(block) => self.verify_br(block)?,
                    Instruction::Call(func, args) => self.verify_call(func, args)?,
                    Instruction::Cmp {
                        pred,
                        lhs_value,
                        rhs_value,
                        ret_ty: _,
                    } => self.verify_cmp(pred, lhs_value, rhs_value)?,
                    Instruction::ConditionalBranch {
                        cond_value,
                        true_block,
                        false_block,
                    } => self.verify_cbr(cond_value, true_block, false_block)?,
                    Instruction::ContractCall {
                        params,
                        coins,
                        asset_id,
                        gas,
                        ..
                    } => self.verify_contract_call(params, coins, asset_id, gas)?,
                    Instruction::ExtractElement {
                        array,
                        array_ty,
                        index_val,
                    } => self.verify_extract_element(array, array_ty, index_val)?,
                    Instruction::ExtractValue {
                        aggregate,
                        struct_ty,
                        indices,
                    } => self.verify_extract_value(aggregate, struct_ty, indices)?,
                    Instruction::GetPointer {
                        base_ptr,
                        ptr_ty,
                        offset,
                    } => self.verify_get_ptr(base_ptr, ptr_ty, offset)?,
                    Instruction::InsertElement {
                        array,
                        array_ty,
                        value,
                        index_val,
                    } => self.verify_insert_element(array, array_ty, value, index_val)?,
                    Instruction::InsertValue {
                        aggregate,
                        struct_ty,
                        value,
                        indices,
                    } => self.verify_insert_value(aggregate, struct_ty, value, indices)?,
                    Instruction::IntToPtr(value, ty) => self.verify_int_to_ptr(value, ty)?,
                    Instruction::Load(ptr) => self.verify_load(ptr)?,
                    Instruction::MemCopy {
                        dst_val,
                        src_val,
                        byte_len,
                        ret_ty: _,
                    } => self.verify_mem_copy(dst_val, src_val, byte_len)?,
                    Instruction::Nop => (),
                    Instruction::Ret(val, ty) => self.verify_ret(val, ty)?,
                    Instruction::Store {
                        dst_val,
                        stored_val,
                        ret_ty: _,
                    } => self.verify_store(dst_val, stored_val)?,

                    Instruction::FuelVm(fuel_instr) => match fuel_instr {
                        FuelVmInstruction::GetStorageKey(_ty) => (),
                        FuelVmInstruction::Gtf {
                            index,
                            tx_field_id,
                            ret_ty: _,
                        } => self.verify_gtf(index, tx_field_id)?,
                        FuelVmInstruction::Log {
                            log_val,
                            log_ty,
                            log_id,
                            ret_ty: _,
                        } => self.verify_log(log_val, log_ty, log_id)?,
                        FuelVmInstruction::ReadRegister(..) => (),
                        FuelVmInstruction::Revert(val) => self.verify_revert(val)?,
                        FuelVmInstruction::Smo {
                            recipient_and_message,
                            message_size,
                            output_index,
                            coins,
                            ret_ty: _,
                        } => self.verify_smo(
                            recipient_and_message,
                            message_size,
                            output_index,
                            coins,
                        )?,
                        FuelVmInstruction::StateLoadWord(key, _ty) => {
                            self.verify_state_load_word(key)?
                        }
                        FuelVmInstruction::StateLoadQuadWord {
                            load_val: dst_val,
                            key,
                            ret_ty: _,
                        }
                        | FuelVmInstruction::StateStoreQuadWord {
                            stored_val: dst_val,
                            key,
                            ret_ty: _,
                        } => self.verify_state_load_store(dst_val, key)?,
                        FuelVmInstruction::StateStoreWord {
                            stored_val: dst_val,
                            key,
                            ret_ty: _,
                        } => self.verify_state_store_word(dst_val, key)?,
                    },
                };

                // Verify the instruction metadata too.
                self.context.verify_metadata(value_content.metadata)?;
            } else {
                unreachable!("Verify instruction is not an instruction.");
            }
        }
        Ok(())
    }

    fn verify_addr_of(&self, value: &Value) -> Result<(), IrError> {
        let val_ty = value
            .get_stripped_ptr_type(self.context)
            .ok_or(IrError::VerifyAddrOfUnknownSourceType)?;
        if val_ty.is_copy_type(self.context) {
            return Err(IrError::VerifyAddrOfCopyType);
        }
        Ok(())
    }

    fn verify_bitcast(&self, value: &Value, to_ty: &Type) -> Result<(), IrError> {
        // The to and from types must be copy-types, excluding short strings, and the same size.
        let val_ty = value
            .get_type(self.context)
            .ok_or(IrError::VerifyBitcastUnknownSourceType)?;
        if !val_ty.is_copy_type(self.context) {
            return Err(IrError::VerifyBitcastFromNonCopyType(
                val_ty.as_string(self.context),
            ));
        }
        if !to_ty.is_copy_type(self.context) {
            return Err(IrError::VerifyBitcastToNonCopyType(
                val_ty.as_string(self.context),
            ));
        }
        let is_valid = match val_ty.get_content(self.context) {
            TypeContent::Unit | TypeContent::Bool => true, // Unit or bool to any copy type works.
            TypeContent::Uint(from_nbits) => match to_ty.get_content(self.context) {
                TypeContent::Unit | TypeContent::Bool => true, // We can construct a unit or bool from any sized integer.
                TypeContent::Uint(to_nbits) => from_nbits == to_nbits,
                _otherwise => false,
            },
            TypeContent::B256
            | TypeContent::String(_)
            | TypeContent::Array(..)
            | TypeContent::Union(_)
            | TypeContent::Struct(_)
            | TypeContent::Pointer { .. }
            | TypeContent::Slice => false,
        };
        if !is_valid {
            Err(IrError::VerifyBitcastBetweenInvalidTypes(
                val_ty.as_string(self.context),
                to_ty.as_string(self.context),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_binary_op(
        &self,
        _op: &BinaryOpKind,
        arg1: &Value,
        arg2: &Value,
    ) -> Result<(), IrError> {
        let arg1_ty = arg1
            .get_type(self.context)
            .ok_or(IrError::VerifyBinaryOpIncorrectArgType)?;
        let arg2_ty = arg2
            .get_type(self.context)
            .ok_or(IrError::VerifyBinaryOpIncorrectArgType)?;
        if !arg1_ty.eq(self.context, &arg2_ty) || !arg1_ty.is_uint(self.context) {
            return Err(IrError::VerifyBinaryOpIncorrectArgType);
        }

        Ok(())
    }

    fn verify_br(&self, dest_block: &BranchToWithArgs) -> Result<(), IrError> {
        if !self.cur_function.blocks.contains(&dest_block.block) {
            Err(IrError::VerifyBranchToMissingBlock(
                self.context.blocks[dest_block.block.0].label.clone(),
            ))
        } else {
            self.verify_dest_args(dest_block)
        }
    }

    fn verify_call(&self, callee: &Function, args: &[Value]) -> Result<(), IrError> {
        let callee_content = &self.context.functions[callee.0];
        if !self.cur_module.functions.contains(callee) {
            Err(IrError::VerifyCallToMissingFunction(
                callee_content.name.clone(),
            ))
        } else {
            let callee_arg_types = callee_content
                .arguments
                .iter()
                .map(|(_, arg_val)| {
                    if let ValueDatum::Argument(BlockArgument { ty, .. }) =
                        &self.context.values[arg_val.0].value
                    {
                        Ok(*ty)
                    } else {
                        Err(IrError::VerifyArgumentValueIsNotArgument(
                            callee_content.name.clone(),
                        ))
                    }
                })
                .collect::<Result<Vec<Type>, IrError>>()?;

            for (opt_caller_arg_type, callee_arg_type) in args
                .iter()
                .map(|val| val.get_type(self.context))
                .zip(callee_arg_types.iter())
            {
                if opt_caller_arg_type.is_none() {
                    return Err(IrError::VerifyUntypedValuePassedToFunction);
                }

                let caller_arg_type = opt_caller_arg_type.as_ref().unwrap();
                let is_ref_call = !callee_arg_type.is_copy_type(self.context)
                    && caller_arg_type.is_ptr_type(self.context);
                if !caller_arg_type.eq(self.context, callee_arg_type) && !is_ref_call {
                    return Err(IrError::VerifyCallArgTypeMismatch(
                        callee_content.name.clone(),
                    ));
                }
            }

            Ok(())
        }
    }

    fn verify_dest_args(&self, dest: &BranchToWithArgs) -> Result<(), IrError> {
        if dest.block.num_args(self.context) != dest.args.len() {
            return Err(IrError::VerifyBranchParamsMismatch);
        }
        for (arg_idx, dest_param) in dest.block.arg_iter(self.context).enumerate() {
            match dest.args.get(arg_idx) {
                Some(actual)
                    if dest_param
                        .get_type(self.context)
                        .unwrap()
                        .eq(self.context, &actual.get_type(self.context).unwrap()) => {}
                _ =>
                // TODO: https://github.com/FuelLabs/sway/pull/2880
                {
                    // return Err(IrError::VerifyBranchParamsMismatch)
                }
            }
        }
        Ok(())
    }

    fn verify_cbr(
        &self,
        cond_val: &Value,
        true_block: &BranchToWithArgs,
        false_block: &BranchToWithArgs,
    ) -> Result<(), IrError> {
        if !(cond_val
            .get_type(self.context)
            .map_or(false, |t| t.is_bool(self.context)))
        {
            Err(IrError::VerifyConditionExprNotABool)
        } else if !self.cur_function.blocks.contains(&true_block.block) {
            Err(IrError::VerifyBranchToMissingBlock(
                self.context.blocks[true_block.block.0].label.clone(),
            ))
        } else if !self.cur_function.blocks.contains(&false_block.block) {
            Err(IrError::VerifyBranchToMissingBlock(
                self.context.blocks[false_block.block.0].label.clone(),
            ))
        } else {
            self.verify_dest_args(true_block)
                .and_then(|()| self.verify_dest_args(false_block))
        }
    }

    fn verify_cmp(
        &self,
        _pred: &Predicate,
        lhs_value: &Value,
        rhs_value: &Value,
    ) -> Result<(), IrError> {
        // Comparisons must be between integers at this stage.
        match (
            lhs_value.get_type(self.context),
            rhs_value.get_type(self.context),
        ) {
            (Some(lhs_ty), Some(rhs_ty)) => match (
                lhs_ty.get_content(self.context),
                rhs_ty.get_content(self.context),
            ) {
                (TypeContent::Uint(lhs_nbits), TypeContent::Uint(rhs_nbits)) => {
                    if lhs_nbits != rhs_nbits {
                        Err(IrError::VerifyCmpTypeMismatch(
                            lhs_ty.as_string(self.context),
                            rhs_ty.as_string(self.context),
                        ))
                    } else {
                        Ok(())
                    }
                }

                (TypeContent::Bool, TypeContent::Bool) => Ok(()),

                _otherwise => Err(IrError::VerifyCmpBadTypes(
                    lhs_ty.as_string(self.context),
                    rhs_ty.as_string(self.context),
                )),
            },
            _otherwise => Err(IrError::VerifyCmpUnknownTypes),
        }
    }

    fn verify_contract_call(
        &self,
        params: &Value,
        coins: &Value,
        asset_id: &Value,
        gas: &Value,
    ) -> Result<(), IrError> {
        // - The params must be a struct with the B256 address, u64 selector and u64 address to
        //   user args.
        // - The coins and gas must be u64s.
        // - The asset_id must be a B256
        if let Some(params_ty) = params
            .get_stripped_ptr_type(self.context)
            .map(|t| t.get_content(self.context))
        {
            match params_ty {
                TypeContent::Struct(fields)
                    if fields.len() == 3
                        && fields[0].is_b256(self.context)
                        && fields[1].is_uint_of(self.context, 64)
                        && fields[2].is_uint_of(self.context, 64) =>
                {
                    Ok(())
                }
                _ => Err(IrError::VerifyContractCallBadTypes("params".to_owned())),
            }
        } else {
            Err(IrError::VerifyContractCallBadTypes("params".to_owned()))
        }
        .and_then(|_| {
            if coins
                .get_type(self.context)
                .map_or(false, |t| t.is_uint_of(self.context, 64))
            {
                Ok(())
            } else {
                Err(IrError::VerifyContractCallBadTypes("coins".to_owned()))
            }
        })
        .and_then(|_| {
            if asset_id
                .get_type(self.context)
                .map_or(false, |t| t.is_b256(self.context))
            {
                Ok(())
            } else {
                Err(IrError::VerifyContractCallBadTypes("asset_id".to_owned()))
            }
        })
        .and_then(|_| {
            if gas
                .get_type(self.context)
                .map_or(false, |t| t.is_uint_of(self.context, 64))
            {
                Ok(())
            } else {
                Err(IrError::VerifyContractCallBadTypes("gas".to_owned()))
            }
        })
    }

    fn verify_extract_element(
        &self,
        array: &Value,
        ty: &Type,
        index_val: &Value,
    ) -> Result<(), IrError> {
        array.get_stripped_ptr_type(self.context).map_or(
            Err(IrError::VerifyAccessElementOnNonArray),
            |array_ty| {
                if !array_ty.is_array(self.context) {
                    Err(IrError::VerifyAccessElementOnNonArray)
                } else if !array_ty
                    .get_array_elem_type(self.context)
                    .map_or(false, |elem_ty| elem_ty.eq(self.context, ty))
                {
                    Err(IrError::VerifyAccessElementInconsistentTypes)
                } else if !index_val
                    .get_type(self.context)
                    .map_or(false, |t| t.is_uint(self.context))
                {
                    Err(IrError::VerifyAccessElementNonIntIndex)
                } else {
                    Ok(())
                }
            },
        )
    }

    fn verify_extract_value(
        &self,
        aggregate: &Value,
        ty: &Type,
        indices: &[u64],
    ) -> Result<(), IrError> {
        let agg_ty = aggregate
            .get_stripped_ptr_type(self.context)
            .ok_or(IrError::VerifyAccessValueOnNonStruct)?;

        if !agg_ty.eq(self.context, ty) {
            Err(IrError::VerifyAccessValueInconsistentTypes)
        } else if !agg_ty.is_struct(self.context) && !agg_ty.is_union(self.context) {
            Err(IrError::VerifyAccessValueOnNonStruct)
        } else if ty.get_indexed_type(self.context, indices).is_none() {
            Err(IrError::VerifyAccessValueInvalidIndices)
        } else {
            Ok(())
        }
    }

    fn verify_get_ptr(
        &self,
        base_ptr: &Pointer,
        _ptr_ty: &Option<Type>,
        _offset: &u64,
    ) -> Result<(), IrError> {
        // We should perhaps verify that the offset and the casted type fit within the base type.
        if !self
            .cur_function
            .local_storage
            .values()
            .any(|x| x == base_ptr)
        {
            Err(IrError::VerifyGetNonExistentPointer)
        } else {
            Ok(())
        }
    }

    fn verify_gtf(&self, index: &Value, _tx_field_id: &u64) -> Result<(), IrError> {
        // We should perhaps verify that _tx_field_id fits in a twelve bit immediate
        if !index
            .get_type(self.context)
            .map_or(false, |ty| ty.is_uint(self.context))
        {
            Err(IrError::VerifyInvalidGtfIndexType)
        } else {
            Ok(())
        }
    }

    fn verify_insert_element(
        &self,
        array: &Value,
        ty: &Type,
        value: &Value,
        index_val: &Value,
    ) -> Result<(), IrError> {
        if let Some(array_ty) = array.get_stripped_ptr_type(self.context) {
            if !array_ty.is_array(self.context) {
                Err(IrError::VerifyAccessElementOnNonArray)
            } else if !array_ty.eq(self.context, ty) {
                Err(IrError::VerifyAccessElementInconsistentTypes)
            } else if self.opt_ty_not_eq(
                &array_ty.get_array_elem_type(self.context),
                &value.get_type(self.context),
            ) {
                Err(IrError::VerifyInsertElementOfIncorrectType)
            } else if !index_val
                .get_type(self.context)
                .map_or(false, |t| t.is_uint(self.context))
            {
                Err(IrError::VerifyAccessElementNonIntIndex)
            } else {
                Ok(())
            }
        } else {
            Err(IrError::VerifyAccessElementOnNonArray)
        }
    }

    fn verify_insert_value(
        &self,
        aggregate: &Value,
        ty: &Type,
        value: &Value,
        idcs: &[u64],
    ) -> Result<(), IrError> {
        let agg_ty = aggregate
            .get_stripped_ptr_type(self.context)
            .ok_or(IrError::VerifyAccessValueOnNonStruct)?;

        if !agg_ty.eq(self.context, ty) {
            Err(IrError::VerifyAccessValueInconsistentTypes)
        } else if !agg_ty.is_struct(self.context) && !agg_ty.is_union(self.context) {
            Err(IrError::VerifyAccessValueOnNonStruct)
        } else if self.opt_ty_not_eq(
            &ty.get_indexed_type(self.context, idcs),
            &value.get_stripped_ptr_type(self.context),
        ) {
            Err(IrError::VerifyAccessValueInvalidIndices)
        } else {
            Ok(())
        }
    }

    fn verify_int_to_ptr(&self, value: &Value, ty: &Type) -> Result<(), IrError> {
        // We want the source value to be an integer and the destination type to be a reference
        // type.
        let val_ty = value
            .get_type(self.context)
            .ok_or(IrError::VerifyIntToPtrUnknownSourceType)?;
        if !val_ty.is_uint_of(self.context, 64) {
            return Err(IrError::VerifyIntToPtrFromNonIntegerType(
                val_ty.as_string(self.context),
            ));
        }
        if !ty.is_ptr_type(self.context) {
            return Err(IrError::VerifyIntToPtrToCopyType(
                val_ty.as_string(self.context),
            ));
        }

        Ok(())
    }

    fn verify_load(&self, src_val: &Value) -> Result<(), IrError> {
        if !src_val
            .get_type(self.context)
            .map_or(false, |t| t.is_ptr_type(self.context))
        {
            Err(IrError::VerifyLoadFromNonPointer)
        } else {
            Ok(())
        }
    }

    fn verify_log(&self, log_val: &Value, log_ty: &Type, log_id: &Value) -> Result<(), IrError> {
        match log_id.get_type(self.context) {
            Some(ty) if ty.is_uint_of(self.context, 64) => {}
            _ => {
                return Err(IrError::VerifyLogId);
            }
        }

        if self.opt_ty_not_eq(&log_val.get_stripped_ptr_type(self.context), &Some(*log_ty)) {
            return Err(IrError::VerifyMismatchedLoggedTypes);
        }

        Ok(())
    }

    fn verify_mem_copy(
        &self,
        _dst_val: &Value,
        _src_val: &Value,
        _byte_len: &u64,
    ) -> Result<(), IrError> {
        // We should perhaps verify that the pointer types are the same size in bytes, or at least
        // the dst is equal to or larger than the src.
        //
        //| XXX Pointers are broken, pending https://github.com/FuelLabs/sway/issues/2819
        //| So here we may still get non-pointers, but still ref-types, passed as the source for
        //| mem_copy, especially when dealing with constant b256s or similar.
        //|if dst_val.get_pointer(self.context).is_none()
        //|    || !(src_val.get_pointer(self.context).is_some()
        //|        || matches!(
        //|            src_val.get_instruction(self.context),
        //|            Some(Instruction::GetStorageKey) | Some(Instruction::IntToPtr(..))
        //|        ))
        //|{
        //|    Err(IrError::VerifyMemcopyNonExistentPointer)
        //|} else {
        Ok(())
        //|}
    }

    fn verify_ret(&self, val: &Value, ty: &Type) -> Result<(), IrError> {
        //| XXX Also waiting for better pointers in https://github.com/FuelLabs/sway/issues/2819
        //| We should disallow returning ref types, as we're using 'out' parameters for anything
        //| that doesn't fit in a reg. So we should instead return pointers to those ref type
        //| values.  But we need better support from a data section for constant ref-type values,
        //| which is currently handled in ASMgen, but should be handled here in IR.
        //|
        //|if !self.cur_func_is_entry() && !ty.is_copy_type(self.context) {
        //|    Err(IrError::VerifyReturnRefTypeValue(
        //|        self.cur_function.name.clone(),
        //|        ty.as_string(self.context),
        //|    ))
        //|} else
        if !self.cur_function.return_type.eq(self.context, ty)
            || (self.opt_ty_not_eq(&val.get_type(self.context), &Some(*ty))
                && self.opt_ty_not_eq(&val.get_stripped_ptr_type(self.context), &Some(*ty)))
        {
            Err(IrError::VerifyMismatchedReturnTypes(
                self.cur_function.name.clone(),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_revert(&self, val: &Value) -> Result<(), IrError> {
        if !val
            .get_type(self.context)
            .map_or(false, |t| t.is_uint_of(self.context, 64))
        {
            Err(IrError::VerifyRevertCodeBadType)
        } else {
            Ok(())
        }
    }

    fn verify_smo(
        &self,
        recipient_and_message: &Value,
        message_size: &Value,
        output_index: &Value,
        coins: &Value,
    ) -> Result<(), IrError> {
        // Check a value is a `u64`, returns a `Result<(), IrError>`.
        let is_u64_or = |val: &Value, err| {
            val.get_type(self.context)
                .map_or(false, |t| t.is_uint_of(self.context, 64))
                .then_some(())
                .ok_or(err)
        };

        // Check that the first operand is a struct with the first field being a `b256`
        // representing the recipient address
        recipient_and_message
            .get_stripped_ptr_type(self.context)
            .map_or(Err(IrError::VerifySmoRecipientBadType), |msg_ty| {
                if !msg_ty.is_struct(self.context)
                    || msg_ty
                        .field_iter(self.context)
                        .next()
                        .map_or(false, |fld_ty| !fld_ty.is_b256(self.context))
                {
                    Err(IrError::VerifySmoRecipientBadType)
                } else {
                    Ok(())
                }
            })
            .and_then(|_| is_u64_or(message_size, IrError::VerifySmoMessageSize))
            .and_then(|_| is_u64_or(output_index, IrError::VerifySmoOutputIndex))
            .and_then(|_| is_u64_or(coins, IrError::VerifySmoCoins))
    }

    fn verify_state_load_store(&self, dst_val: &Value, key: &Value) -> Result<(), IrError> {
        if !matches!(dst_val.get_stripped_ptr_type(self.context), Some(ty) if ty.is_b256(self.context))
        {
            Err(IrError::VerifyStateDestBadType("b256".to_owned()))
        } else if !key
            .get_stripped_ptr_type(self.context)
            .map_or(false, |t| t.is_b256(self.context))
        {
            Err(IrError::VerifyStateKeyBadType)
        } else {
            Ok(())
        }
    }

    fn verify_state_load_word(&self, key: &Value) -> Result<(), IrError> {
        if !key
            .get_stripped_ptr_type(self.context)
            .map_or(false, |t| t.is_b256(self.context))
        {
            Err(IrError::VerifyStateKeyBadType)
        } else {
            Ok(())
        }
    }

    fn verify_state_store_word(&self, dst_val: &Value, key: &Value) -> Result<(), IrError> {
        if !key
            .get_stripped_ptr_type(self.context)
            .map_or(false, |t| t.is_b256(self.context))
        {
            Err(IrError::VerifyStateKeyBadType)
        } else if !dst_val
            .get_type(self.context)
            .map_or(false, |t| t.is_uint_of(self.context, 64))
        {
            Err(IrError::VerifyStateDestBadType("u64".to_owned()))
        } else {
            Ok(())
        }
    }

    fn verify_store(&self, dst_val: &Value, stored_val: &Value) -> Result<(), IrError> {
        let dst_ty = dst_val.get_stripped_ptr_type(self.context);
        let stored_ty = stored_val.get_stripped_ptr_type(self.context);
        if dst_ty.is_none() {
            Err(IrError::VerifyStoreToNonPointer)
        } else if self.opt_ty_not_eq(&dst_ty, &stored_ty) {
            Err(IrError::VerifyStoreMismatchedTypes)
        } else {
            Ok(())
        }
    }

    // This is a really common operation above... calling `Value::get_type()` and then failing when
    // two don't match.
    fn opt_ty_not_eq(&self, l_ty: &Option<Type>, r_ty: &Option<Type>) -> bool {
        l_ty.is_none() || r_ty.is_none() || !l_ty.unwrap().eq(self.context, r_ty.as_ref().unwrap())
    }
}
