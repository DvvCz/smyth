mod backend;
mod gen;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	use backend::Ast;
	use gen::CodeGenerator;

	let ast = backend::Backend::generate(include_bytes!("../test.rs"))?;

	let codegen = gen::lua::LuaCodegen::new();
	let code = codegen.generate(&ast)?;

	std::fs::write("out.lua", code)?;

	Ok(())
}
