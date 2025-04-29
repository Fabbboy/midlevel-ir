use std::collections::HashMap;

use function::{Block, Function};
use inkwell::{
    builder::Builder,
    context::Context,
    types::{BasicType, BasicTypeEnum},
    values::{BasicValue, BasicValueEnum},
};
use instruction::{AssignInst, DefineInst, InstId, Instruction};
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

    let assign_inst = Instruction::Assign(AssignInst::new(
        Value::Instruction(define_inst_id),
        Value::ConstantInt(420),
    ));

    let assign_inst_id = function.add_instruction(assign_inst);
    let block = function.get_block_mut(entry_handle).unwrap();
    block.adjust_range(assign_inst_id);

    println!("{:#?}", module);

    compile(&module, &mut codeg); //here
    codeg.llvm_mod.print_to_stderr();
}
