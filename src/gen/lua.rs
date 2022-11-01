use crate::backend::Item;

#[derive(Debug)]
pub struct LuaCodegen {}

impl LuaCodegen {
	pub fn new() -> Self {
		Self {}
	}
}

impl super::CodeGenerator for LuaCodegen {
	fn generate(&self, ast: &impl crate::backend::AST) -> super::Result<String> {
		let items = ast.items();
		let mut buf = String::new();

		let mut indent = 0;

		fn push_stmts(buf: &mut String, indent: &mut u8, stmts: &Vec<Item>) {
			*indent += 1;
			for item in stmts {
				push_item(buf, indent, item);
			}
			*indent -= 1;
		}

		fn push_item(buf: &mut String, indent: &mut u8, item: &Item) {
			match item {
				Item::FunctionDefinition {
					name,
					params,
					stmts,
				} => {
					buf.push_str(&format!("function {name}("));
					buf.push_str(&params.join(","));
					buf.push(')');

					push_stmts(buf, indent, stmts);
					buf.push_str("end;")
				}

				Item::VarDecl { name, expr } => {
					buf.push_str(&format!("local {name} = "));
					push_item(buf, indent, expr);
					buf.push(';');
				},

				Item::ExprCall { func, args } => {
					push_item(buf, indent, func); // todo: push_item_inline
					buf.push('(');
					for (i, arg) in args.iter().enumerate() {
						push_item(buf, indent, arg);

						if i != args.len() - 1 {
							buf.push(',');
						}
					}
					buf.push(')');
				},

				Item::ExprIdent(ident) => buf.push_str(&ident),

				Item::ExprInteger(val) => buf.push_str(&val.to_string()),
				Item::ExprBool(val) => buf.push_str(&val.to_string()),
				Item::ExprString(val) => buf.push_str(&format!("\"{}\"", val.escape_default())),

				x => {
					dbg!(x);
					todo!()
				}
			}
		}

		for item in items {
			push_item(&mut buf, &mut indent, item);
		}

		Ok(buf)
	}
}
