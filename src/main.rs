use cranelift::prelude::*;
use std::io::Write;

fn main() -> anyhow::Result<()> {
    // 対象のISAを作る
    // 今回は動いてるマシンと同じISA
    let flag_builder = codegen::settings::builder();
    let isa_builder = cranelift_native::builder().unwrap();
    let isa = isa_builder.finish(cranelift_codegen::settings::Flags::new(flag_builder));

    let obj_builder = cranelift_object::ObjectBuilder::new(
        isa,
        "test", // よくわからないが何か全体の名前、`objdump -t` すると出てくる
        cranelift_module::default_libcall_names(),
    );

    let mut module: cranelift_module::Module<cranelift_object::ObjectBackend> =
        cranelift_module::Module::new(obj_builder);

    // このISAにおけるint
    let int = module.target_config().pointer_type();

    let mut ctx = module.make_context();
    let mut builder_context = FunctionBuilderContext::new();

    {
        ctx.func.signature.returns.push(AbiParam::new(int));
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_context);
        let entry_block = builder.create_block();
        // builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);
        let ret = builder.ins().iconst(int, 42);
        // return 42;
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
        module.clear_context(&mut ctx);
    }

    let obj = module.finish();
    let bytes = obj.emit()?;

    std::fs::File::create("out.o")?.write_all(&bytes)?;
    // cc -o out out.o
    // ./out
    // echo $? # => 42 (yay)

    Ok(())
}
