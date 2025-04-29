use std::collections::HashMap;

use function::{Block, Function};
use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    passes::{PassBuilderOptions, PassManager},
    targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine},
    types::{BasicType, BasicTypeEnum},
    values::{BasicValue, BasicValueEnum},
};
use instruction::{AddInst, DefineInst, InstId, Instruction, RetInst};
use module::Module;
use types::MIRType;
use value::Value;

pub mod function;
pub mod instruction;
pub mod module;
pub mod types;
pub mod value;

struct Codegen<'ctx> {
    llvm_ctx: &'ctx Context,
    llvm_mod: inkwell::module::Module<'ctx>,
    llvm_builder: Builder<'ctx>,
    namend: HashMap<InstId, BasicValueEnum<'ctx>>,
}

fn to_llvm_type<'ctx>(ty: MIRType, codegen: &Codegen<'ctx>) -> BasicTypeEnum<'ctx> {
    match ty {
        MIRType::Int32 => codegen.llvm_ctx.i32_type().into(),
    }
}

fn to_llvm_value<'ctx>(value: Value, codegen: &Codegen<'ctx>) -> BasicValueEnum<'ctx> {
    match value {
        Value::Instruction(inst_id) => {
            let llvm_value = codegen
                .namend
                .get(&inst_id)
                .expect("Instruction not found in namend map");
            return llvm_value.clone();
        }
        Value::ConstantInt(literal) => codegen
            .llvm_ctx
            .i64_type()
            .const_int(literal as u64, false)
            .into(),
        Value::ConstantFloat(literal) => codegen.llvm_ctx.f64_type().const_float(literal).into(),
    }
}

fn compile_block<'f, 'ctx>(block: &'f Block, func: &'f Function, codegen: &'f mut Codegen<'ctx>) {
    let instructions = block.get_instructions(func);
    let range = block.get_range();
    for (i, inst) in instructions.iter().enumerate() {
        let inst_id = InstId(range.start.0 + i);
        match inst {
            Instruction::Define(define_inst) => {
                let llvm_type = to_llvm_type(define_inst.get_type(), codegen);
                let llvm_value = codegen
                    .llvm_builder
                    .build_alloca(llvm_type, "temp")
                    .expect("Failed to create alloca");
                let value = to_llvm_value(define_inst.get_value().clone(), codegen);
                codegen
                    .llvm_builder
                    .build_store(llvm_value, value)
                    .expect("Failed to store value");

                codegen
                    .namend
                    .insert(inst_id, llvm_value.as_basic_value_enum());
            }
            Instruction::Assign(assign_inst) => {
                let dest = to_llvm_value(assign_inst.get_dest().clone(), codegen);
                let src = to_llvm_value(assign_inst.get_src().clone(), codegen);
                codegen
                    .llvm_builder
                    .build_store(dest.into_pointer_value(), src)
                    .expect("Failed to store value");
            }
            Instruction::Add(add_inst) => {
                // 1) Get the pointer for the destination
                let dest_ptr = match add_inst.get_dest() {
                    Value::Instruction(dest_id) => {
                        codegen.namend.get(dest_id).unwrap().into_pointer_value()
                    }
                    _ => unreachable!(),
                };

                // 2) LOAD LHS
                let lhs_val = match add_inst.get_lhs().clone() {
                    Value::Instruction(id) => {
                        let ptr = codegen.namend.get(&id).unwrap().into_pointer_value();
                        codegen
                            .llvm_builder
                            .build_load(to_llvm_type(add_inst.get_type(), codegen), ptr, "lhs_load")
                            .unwrap()
                            .into_int_value()
                    }
                    Value::ConstantInt(lit) => {
                        // make sure you use i32_type() if your MIRType::Int32
                        codegen.llvm_ctx.i32_type().const_int(lit as u64, false)
                    }
                    _ => unreachable!(),
                };

                // 3) LOAD or CONST RHS (similarly)
                let rhs_val = match add_inst.get_rhs().clone() {
                    Value::Instruction(id) => {
                        let ptr = codegen.namend.get(&id).unwrap().into_pointer_value();
                        codegen
                            .llvm_builder
                            .build_load(to_llvm_type(add_inst.get_type(), codegen), ptr, "rhs_load")
                            .unwrap()
                            .into_int_value()
                    }
                    Value::ConstantInt(lit) => {
                        codegen.llvm_ctx.i32_type().const_int(lit as u64, false)
                    }
                    _ => unreachable!(),
                };

                // 4) BUILD THE ACTUAL ADD INSTRUCTION
                let sum = codegen
                    .llvm_builder
                    .build_int_add(lhs_val, rhs_val, "add")
                    .unwrap();

                // 5) STORE THE RESULT BACK
                codegen
                    .llvm_builder
                    .build_store(dest_ptr, sum)
                    .expect("store sum");

                // 6) And—very important—remember to put *this* result into your map
                codegen.namend.insert(inst_id, sum.as_basic_value_enum());
            }

            Instruction::Ret(ret_inst) => {
                let ret_value = to_llvm_value(ret_inst.get_value().clone(), codegen);
                codegen
                    .llvm_builder
                    .build_return(Some(&ret_value))
                    .expect("Failed to build return");
            }
        }
    }
}

fn compile_function<'f, 'ctx>(function: &'f Function, codegen: &'f mut Codegen<'ctx>) {
    let ret_type = to_llvm_type(function.get_ret_type(), codegen);
    let fn_type = ret_type.fn_type(&[], false);
    let llvm_func = codegen
        .llvm_mod
        .add_function(function.get_name(), fn_type, None);

    for block in function.get_blocks() {
        let bb = codegen
            .llvm_ctx
            .append_basic_block(llvm_func, block.get_name());
        codegen.llvm_builder.position_at_end(bb);
        compile_block(block, function, codegen);
    }
}

fn compile<'ctx>(module: &Module, codegen: &mut Codegen<'ctx>) {
    for func in module.get_functions() {
        compile_function(func, codegen);
    }
}

fn main() {
    let llvm_ctx = Context::create();
    let mut codeg = Codegen {
        llvm_ctx: &llvm_ctx,
        llvm_mod: llvm_ctx.create_module("main"),
        llvm_builder: llvm_ctx.create_builder(),
        namend: HashMap::new(),
    };

    let mut module = Module::new("main");
    let main_handle = module.add_function(Function::new("main", MIRType::Int32));

    let entry_block = Block::new("entry", InstId(0));

    let function = module.get_function_mut(main_handle).unwrap();
    let entry_handle = function.add_block(entry_block);

    let define_inst = Instruction::Define(DefineInst::new(MIRType::Int32, Value::ConstantInt(69)));
    let define_inst_id = function.add_instruction(define_inst);
    let block = function.get_block_mut(entry_handle).unwrap();
    block.adjust_range(define_inst_id);

    let new_define_inst = Instruction::Define(DefineInst::new(
        MIRType::Int32,
        Value::Instruction(define_inst_id),
    ));
    let new_define_inst_id = function.add_instruction(new_define_inst);
    let block = function.get_block_mut(entry_handle).unwrap();
    block.adjust_range(new_define_inst_id);

    let add_inst = Instruction::Add(AddInst::new(
        Value::Instruction(define_inst_id),
        Value::Instruction(new_define_inst_id),
        Value::ConstantInt(2),
        MIRType::Int32,
    ));

    let add_inst_id = function.add_instruction(add_inst);
    let block = function.get_block_mut(entry_handle).unwrap();
    block.adjust_range(add_inst_id);

    let ret_inst = Instruction::Ret(RetInst::new(Value::Instruction(add_inst_id)));
    let ret_inst_id = function.add_instruction(ret_inst);
    let block = function.get_block_mut(entry_handle).unwrap();
    block.adjust_range(ret_inst_id);

    println!("{:#?}", module);

    compile(&module, &mut codeg); //here
    codeg.llvm_mod.verify().unwrap();
    codeg
        .llvm_mod
        .print_to_stderr();
}
