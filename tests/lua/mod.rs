use include_dir::include_dir;

#[test]
fn main() -> Result<(), Box<dyn std::error::Error>> {
	let dir = std::fs::read_dir("tests/lua/examples").unwrap();
	for entry in dir {
		let entry = entry?;
		let path = entry.path();

		match path.extension() {
			Some(extension) if extension == "rs" => {
				let source = std::fs::read_to_string(&path)?;

				use smyth::backend::Ast;
				use smyth::gen::CodeGenerator;
			
				let ast = smyth::backend::Backend::generate(source)?;
				let codegen = smyth::gen::lua::LuaCodegen::new();
				let code = codegen.generate(&ast)?;
			
				let expected = std::fs::read_to_string(path.with_extension("lua"))?;
				assert_eq!(code, expected);
			}
			_ => ()
		}

	}

	Ok(())
}