///! Function argument demotion.
///!
///! This pass demotes 'by-value' function arg types to 'by-reference` pointer types, based on target
///! specific parameters.
///!
///! It does NOT change the call sites to modified functions.  This is done in subsequent passes
///! (TODO).
use crate::{
    AnalysisResults, BlockArgument, Context, Function, Instruction, IrError, Pass, PassMutability,
    ScopedPass, Type, TypeContent, Value, ValueDatum,
};

pub fn create_arg_demotion_pass() -> Pass {
    Pass {
        name: "argdemotion",
        descr: "By-value function argument demotion to by-reference.",
        runner: ScopedPass::FunctionPass(PassMutability::Transform(fn_arg_demotion)),
    }
}

pub fn fn_arg_demotion(
    context: &mut Context,
    _: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    // The criteria for now for demotion is whether the arg type is larger than 64-bits or is an
    // aggregate.  This info should be instead determined by a target info analysis pass.

    // Find candidate argument indices.
    let candidate_args = function
        .args_iter(context)
        .enumerate()
        .filter_map(|(idx, (_name, arg_val))| {
            arg_val.get_type(context).and_then(|ty| {
                (match ty.get_content(context) {
                    TypeContent::Unit | TypeContent::Bool | TypeContent::Pointer(_) => false,
                    TypeContent::Uint(bits) => *bits > 64,
                    _ => true,
                })
                .then_some((idx, ty))
            })
        })
        .collect::<Vec<(usize, Type)>>();

    if candidate_args.is_empty() {
        return Ok(false);
    }

    // Find all the call sites for this function.
    let call_sites = context
        .module_iter()
        .flat_map(|module| module.function_iter(context))
        .flat_map(|function| function.block_iter(context))
        .flat_map(|block| block.instruction_iter(context))
        .filter_map(|instr_val| {
            if let Instruction::Call(call_to_func, _) = instr_val
                .get_instruction(context)
                .expect("`instruction_iter()` must return instruction values.")
            {
                (*call_to_func == function).then_some(instr_val)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut result = false;
    for (idx, arg_ty) in candidate_args {
        // Demote each arg _if possible_.
        result = demote_arg(context, &call_sites, &function, idx, &arg_ty) || result;
    }

    Ok(result)
}

fn demote_arg(
    context: &mut Context,
    call_sites: &[Value],
    function: &Function,
    arg_idx: usize,
    arg_ty: &Type,
) -> bool {
    // Match the mutable argument value and set its type.
    macro_rules! set_arg_type {
        ($arg_val: ident, $new_ty: ident) => {
            if let ValueDatum::Argument(BlockArgument { ty, .. }) =
                &mut context.values[$arg_val.0].value
            {
                *ty = $new_ty
            }
        };
    }

    // We need to convert the caller arg value at *every* call site from a by-value to a
    // by-reference. At this stage we assume that the arg value must be a `load` instruction, and
    // by getting the pointer arg to the `load` we can pass that at the call site instead.  If it
    // isn't a `load` then we abort for this arg demotion.
    let Some(callers_and_ptrs) = call_sites.iter().map(|call_val| {
        let Some(Instruction::Call(_, args)) = call_val.get_instruction(context) else {
            unreachable!("call value is definitely a call instruction");
        };
        if let Some(Instruction::Load(ptr_val)) = args[arg_idx].get_instruction(context) {
            // Pair the caller with its ptr value for this argument.
            Some((call_val, *ptr_val))
        } else {
            // Not a `load`, abort.
            None
        }
    }).collect::<Option<Vec<_>>>() else {
        // Abort.
        return false;
    };

    // At this stage we're happy that we can convert the function signature and each caller.
    let ptr_ty = Type::new_ptr(context, *arg_ty);

    // Update the function signature.
    let fn_args = &context.functions[function.0].arguments;
    let (_name, arg_val) = &fn_args[arg_idx];
    set_arg_type!(arg_val, ptr_ty);

    // Update the entry block signature.
    let entry_block = function.get_entry_block(context);
    let arg_val = entry_block
        .get_arg(context, arg_idx)
        .expect("Entry block args should be mirror of function args.");
    set_arg_type!(arg_val, ptr_ty);

    // Update each call site, replacing the old `load` argument with the pointer it was loading.
    for (call_val, ptr_val) in callers_and_ptrs {
        let Some(Instruction::Call(_, args)) = call_val.get_instruction_mut(context) else {
            unreachable!("call value is definitely a call instruction");
        };
        args[arg_idx] = ptr_val;
    }

    true
}
