use cranelift::prelude::*;
use std::io::Write;

fn main() -> anyhow::Result<()> {
    let flag_builder = codegen::settings::builder();
    let isa_builder = cranelift_native::builder().unwrap();
    let isa = isa_builder.finish(cranelift_codegen::settings::Flags::new(flag_builder));

    let obj_builder = cranelift_object::ObjectBuilder::new(
        isa,
        "test",
        cranelift_module::default_libcall_names(),
    );

    let mut module: cranelift_module::Module<cranelift_object::ObjectBackend> =
        cranelift_module::Module::new(obj_builder);

    let int = module.target_config().pointer_type();

    let mut ctx = codegen::Context::new();
    let mut builder_context = FunctionBuilderContext::new();
    ctx.func.signature.returns.push(AbiParam::new(int));
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_context);
    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block);
    let ret = builder.ins().iconst(int, 42);
    builder.ins().return_(&[ret]);
    builder.finalize();

    let id = module.declare_function(
        "main",
        cranelift_module::Linkage::Export,
        &ctx.func.signature,
    )?;
    module.define_function(
        id,
        &mut ctx,
        &mut cranelift_codegen::binemit::NullTrapSink {},
    )?;

    let obj = module.finish();
    let bytes = obj.emit()?;

    std::fs::File::create("out.o")?.write_all(&bytes)?;

    Ok(())
}
