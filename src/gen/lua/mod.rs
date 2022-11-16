use crate::backend::Item;

#[derive(Debug)]
pub struct LuaCodegen {}

impl LuaCodegen {
	pub fn new() -> Self {
		Self {}
	}
}

impl super::CodeGenerator for LuaCodegen {
	fn generate(&self, ast: &impl crate::backend::Ast) -> super::Result<String> {
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
				}

				Item::VarSet { name, expr } => {
					buf.push_str(&format!("{name} = "));
					push_item(buf, indent, expr);
					buf.push(';');
				}

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
				}

				Item::ExprIdent(ident) => buf.push_str(ident),

				Item::ExprInteger(val) => buf.push_str(&val.to_string()),
				Item::ExprBool(val) => buf.push_str(&val.to_string()),
				Item::ExprString(val) => buf.push_str(&format!("\"{}\"", val.escape_default())),
				Item::ExprFString { strings, replacements, values } => {
					println!("{strings:?}, {replacements:?}, {values:?}");
					let mut replacements = replacements.iter();
					buf.push_str("string.format(\"");
					for s in strings {
						buf.push_str(s);

						if replacements.next().is_some() {
							buf.push_str("%s");
						}
					}
					buf.push('"');
					for v in values {
						buf.push(',');
						push_item(buf, indent, v);
					}
					buf.push(')');
				},

				Item::ExprClosure { params, stmts } => {
					buf.push_str("function(");
					buf.push_str(&params.join(","));
					buf.push(')');

					push_stmts(buf, indent, stmts);
					buf.push_str("end;")
				}

				Item::ExprBinary { lhs, rhs, op } => {
					use crate::backend::BinaryOp;

					push_item(buf, indent, lhs);
					match op {
						BinaryOp::Add => buf.push('+'),
						BinaryOp::Sub => buf.push('-'),
						BinaryOp::Mul => buf.push('*'),
						BinaryOp::Div => buf.push('/'),
						BinaryOp::Mod => buf.push('%'),

						_ => buf.push_str("test"),
					}

					push_item(buf, indent, rhs);
				}

				Item::ExprArray { elements } => {
					buf.push('{');
					for (i, arg) in elements.iter().enumerate() {
						push_item(buf, indent, arg);

						if i != elements.len() - 1 {
							buf.push(',');
						}
					}
					buf.push('}');
				}

				Item::IfElif(crate::backend::IfElif {
					condition,
					stmts,
					elif,
					else_stmts,
				}) => {
					buf.push_str("if ");
					push_item(buf, indent, condition);
					buf.push_str(" then ");
					push_stmts(buf, indent, stmts);

					if !elif.is_empty() {
						for (cond, stmts) in elif {
							buf.push_str(";elseif ");
							push_item(buf, indent, cond);
							buf.push_str(" then ");
							push_stmts(buf, indent, stmts);
						}
					}

					if let Some(els) = else_stmts {
						buf.push_str(" else ");
						push_stmts(buf, indent, els);
					}

					buf.push_str(" end;")
				}

				Item::While { condition, stmts } => {
					buf.push_str("while ");
					push_item(buf, indent, condition);
					buf.push_str(" do ");
					push_stmts(buf, indent, stmts);
					buf.push_str(" ::__continue__:: end;")
				}

				Item::ForRange { var, min, max, jump: _, stmts } => {
					buf.push_str("for ");
					buf.push_str(var);
					buf.push_str(" = ");
					push_item(buf, indent, min);
					buf.push(',');
					push_item(buf, indent, max);
					buf.push_str(" do ");
					//buf.push_str(",");
					//push_item(buf, indent, jump);

					push_stmts(buf, indent, stmts);
					buf.push_str(" ::__continue__:: end;")
				}

				Item::ForIn { var, expr, stmts } => {
					buf.push_str("for ");
					buf.push_str(var);
					buf.push_str(" in ");
					push_item(buf, indent, expr);
					buf.push_str(" do ");
					push_stmts(buf, indent, stmts);
					buf.push_str(" ::__continue__:: end;")
				}

				Item::Break => buf.push_str("break;"),

				Item::Continue => buf.push_str("goto __continue__;"),

				Item::Externs { functions } => {
					for name in functions {
						buf.push_str(&format!(";local {name} = _G.{name};"));
					}
				}

				Item::Mod { name, items } => {
					buf.push_str(&format!("local {name} = {{}};"));
					for item in items {
						push_item(buf, indent, item);
					}
				}

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
