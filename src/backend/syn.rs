use syn::parse::Parser;

#[derive(Debug)]
pub struct SynBackend {
	pub items: Vec<super::Item>,
}

impl super::Ast for SynBackend {
	fn generate(code: impl AsRef<[u8]>) -> super::Result<Self> {
		let code = code.as_ref();
		let code = std::str::from_utf8(code).unwrap();

		let syn_ast = syn::parse_file(code)?;

		fn syn_item_to_item(item: syn::Item) -> super::Item {
			match item {
				syn::Item::Fn(syn::ItemFn { sig, block, .. }) => {
					if let Some(syn::Abi { name: _, .. }) = sig.abi {
						return super::Item::Externs {
							functions: vec![sig.ident.to_string()],
						};
					}

					let param_names: Vec<String> = sig
						.inputs
						.into_iter()
						.map(|arg| match arg {
							syn::FnArg::Receiver(_) => String::from("self"),
							syn::FnArg::Typed(data) => match data.pat.as_ref() {
								syn::Pat::Ident(name) => name.ident.to_string(),
								_ => todo!(),
							},
						})
						.collect();

					super::Item::FunctionDefinition {
						name: sig.ident.to_string(),
						params: param_names,
						stmts: block.stmts.into_iter().map(stmt_to_item).collect(),
					}
				}

				syn::Item::ForeignMod(syn::ItemForeignMod { abi: _, items, .. }) => {
					// TODO: Different behavior with "C" abi versus no abi.
					// No abi / "lua" / "lang" abi just defines an extern _G function?

					println!("foreignmod");

					let mut funcs = vec![];
					for item in items {
						match item {
							syn::ForeignItem::Fn(syn::ForeignItemFn { sig, attrs: _, .. }) => {
								let name = sig.ident.to_string();
								funcs.push(name);
							},

							syn::ForeignItem::Verbatim(data) => {
								match syn::parse2::<syn::Item>(data) {
									Ok(syn::Item::Fn(syn::ItemFn { sig, .. })) => {
										funcs.push(sig.ident.to_string());
									},
									other => todo!("{:#?}", other)
								}
							},

							other => {
								dbg!(other);
								todo!()
							}
						}
					}

					super::Item::Externs { functions: funcs }
				}

				syn::Item::Mod(syn::ItemMod {
					ident,
					content: Some((_, v)),
					..
				}) => super::Item::Mod {
					name: ident.to_string(),
					items: v.into_iter().map(syn_item_to_item).collect(),
				},

				_ => todo!(),
			}
		}

		fn stmt_to_item(stmt: syn::Stmt) -> super::Item {
			match stmt {
				syn::Stmt::Item(item) => syn_item_to_item(item),
				syn::Stmt::Local(local) => match local.pat {
					syn::Pat::Ident(name) => super::Item::VarDecl {
						name: name.ident.to_string(),
						expr: Box::new(expr_to_item(*local.init.unwrap().1)),
					},
					_ => todo!(),
				},
				syn::Stmt::Expr(expr) => expr_to_item(expr),
				syn::Stmt::Semi(expr, _) => expr_to_item(expr),
			}
		}

		fn expr_to_item(expr: syn::Expr) -> super::Item {
			match expr {
				syn::Expr::Lit(syn::ExprLit { lit, .. }) => match lit {
					syn::Lit::Bool(b) => super::Item::ExprBool(b.value()),
					syn::Lit::Str(s) => super::Item::ExprString(s.value()),
					syn::Lit::Int(i) => super::Item::ExprInteger(i.base10_parse::<i64>().unwrap()),
					syn::Lit::Float(f) => {
						super::Item::ExprDecimal(f.base10_parse::<f64>().unwrap())
					}
	
					_ => todo!(),
				},

				syn::Expr::Call(syn::ExprCall { func, args, .. }) => super::Item::ExprCall {
					func: Box::new(expr_to_item(*func)),
					args: args.into_iter().map(expr_to_item).collect(),
				},

				syn::Expr::Path(syn::ExprPath { path, .. }) => {
					let path_ident = path
						.segments
						.iter()
						.map(|x| x.ident.to_string())
						.collect::<Vec<String>>()
						.join("__");
					super::Item::ExprIdent(path_ident)
				}

				syn::Expr::AssignOp(syn::ExprAssignOp {
					left, right, op, ..
				}) => match op {
					syn::BinOp::AddEq(_) => {
						let left = expr_to_item(*left);
						if let super::Item::ExprIdent(ref name) = left {
							super::Item::VarSet {
								name: name.to_owned(),
								expr: Box::new(super::Item::ExprBinary {
									lhs: Box::new(left),
									rhs: Box::new(expr_to_item(*right)),
									op: super::BinaryOp::Add,
								}),
							}
						} else {
							todo!()
						}
					}

					syn::BinOp::SubEq(_) => {
						let left = expr_to_item(*left);
						if let super::Item::ExprIdent(ref name) = left {
							super::Item::VarSet {
								name: name.to_owned(),
								expr: Box::new(super::Item::ExprBinary {
									lhs: Box::new(left),
									rhs: Box::new(expr_to_item(*right)),
									op: super::BinaryOp::Sub,
								}),
							}
						} else {
							todo!()
						}
					}

					syn::BinOp::MulEq(_) => {
						let left = expr_to_item(*left);
						if let super::Item::ExprIdent(ref name) = left {
							super::Item::VarSet {
								name: name.to_owned(),
								expr: Box::new(super::Item::ExprBinary {
									lhs: Box::new(left),
									rhs: Box::new(expr_to_item(*right)),
									op: super::BinaryOp::Mul,
								}),
							}
						} else {
							todo!()
						}
					}

					syn::BinOp::DivEq(_) => {
						let left = expr_to_item(*left);
						if let super::Item::ExprIdent(ref name) = left {
							super::Item::VarSet {
								name: name.to_owned(),
								expr: Box::new(super::Item::ExprBinary {
									lhs: Box::new(left),
									rhs: Box::new(expr_to_item(*right)),
									op: super::BinaryOp::Div,
								}),
							}
						} else {
							todo!()
						}
					}

					_ => todo!(),
				},

				syn::Expr::Closure(syn::ExprClosure { inputs, body, .. }) => {
					use syn::Pat;

					let params: Vec<String> = inputs
						.into_iter()
						.map(|arg| match arg {
							Pat::Ident(id) => id.ident.to_string(),
							_ => todo!(),
						})
						.collect();

					if let syn::Expr::Block(syn::ExprBlock { block, .. }) = *body {
						super::Item::ExprClosure {
							params,
							stmts: block.stmts.into_iter().map(stmt_to_item).collect(),
						}
					} else {
						todo!()
					}
				}

				syn::Expr::Binary(syn::ExprBinary {
					left, right, op, ..
				}) => super::Item::ExprBinary {
					lhs: Box::new(expr_to_item(*left)),
					rhs: Box::new(expr_to_item(*right)),
					op: op.into(),
				},

				syn::Expr::Array(syn::ExprArray { elems, .. }) => super::Item::ExprArray {
					elements: elems.into_iter().map(expr_to_item).collect(),
				},

				syn::Expr::Tuple(syn::ExprTuple { elems, .. }) => super::Item::ExprArray {
					elements: elems.into_iter().map(expr_to_item).collect(),
				},

				syn::Expr::Loop(syn::ExprLoop { label: _, body, .. }) => super::Item::While {
					condition: Box::new(super::Item::ExprBool(true)),
					stmts: body.stmts.into_iter().map(stmt_to_item).collect(),
				},

				syn::Expr::While(syn::ExprWhile { cond, body, .. }) => super::Item::While {
					condition: Box::new(expr_to_item(*cond)),
					stmts: body.stmts.into_iter().map(stmt_to_item).collect(),
				},

				syn::Expr::ForLoop(syn::ExprForLoop { pat, body, expr, .. }) => {
					let ident = match pat {
						syn::Pat::Ident(i) => i.ident.to_string(),
						other => todo!("{other:?}")
					};

					match *expr {
						syn::Expr::Range(syn::ExprRange { from, to, .. }) => {
							super::Item::ForRange {
								max: Box::new(to.map(|to| expr_to_item(*to)).unwrap_or(super::Item::ExprInteger(9999))),
								min: Box::new(from.map(|from| expr_to_item(*from)).unwrap_or(super::Item::ExprInteger(9999))),
								jump: None,
								var: ident,
								stmts: body.stmts.into_iter().map(stmt_to_item).collect()
							}
						},
						_ => super::Item::ForIn {
							var: ident,
							expr: Box::new(expr_to_item(*expr)),
							stmts: body.stmts.into_iter().map(stmt_to_item).collect()
						}
					}
				},

				syn::Expr::Range(syn::ExprRange { from: _, to: _, .. }) => unimplemented!(),

				syn::Expr::Break(_) => super::Item::Break,
				syn::Expr::Continue(_) => super::Item::Continue,

				syn::Expr::If(syn::ExprIf {
					cond,
					then_branch,
					else_branch,
					..
				}) => {
					if let Some((.., b)) = else_branch {
						if let syn::Expr::Block(syn::ExprBlock { block, .. }) = *b {
							// Block expression works enough as a statement.
							super::Item::IfElif(super::IfElif {
								condition: Box::new(expr_to_item(*cond)),
								stmts: then_branch.stmts.into_iter().map(stmt_to_item).collect(),
								elif: vec![],
								else_stmts: Some(
									block.stmts.into_iter().map(stmt_to_item).collect(),
								),
							})
						} else if let syn::Expr::If(syn::ExprIf {
							cond, then_branch, ..
						}) = *b
						{
							println!("elif {cond:#?} {then_branch:#?}");
							todo!()
						} else {
							// Expression directly ???
							dbg!(b);
							unreachable!()
						}
					// super::Item::IfElif(super::IfElif { condition: Box::new(expr_to_item(*cond)), stmts: then_branch.stmts.into_iter().map(stmt_to_item).collect(), elif: vec![] })
					} else {
						super::Item::IfElif(super::IfElif {
							condition: Box::new(expr_to_item(*cond)),
							stmts: then_branch.stmts.into_iter().map(stmt_to_item).collect(),
							elif: vec![],
							else_stmts: None,
						})
					}
				},

				syn::Expr::Macro(syn::ExprMacro { attrs: _, mac }) => {
					let path = mac.path
						.segments
						.iter()
						.map(|x| x.ident.to_string())
						.collect::<Vec<String>>()
						.join("__");

					if path == "println" {
						// Arguments
						// use syn::parse_quote::ParseQuote;
						let args: Vec<super::Item> = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
							.parse2(mac.tokens)
							.expect("Only accepts items")
							.into_iter()
							.map(expr_to_item)
							.collect();

						super::Item::ExprCall { func: Box::new(super::Item::ExprIdent("print".into())), args }
					} else if path == "format" {
						let mut args = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
							.parse2(mac.tokens)
							.expect("Only accepts items")
							.into_iter()
							.map(expr_to_item);

						let mut strings = vec![];
						let mut replacements = vec![];
						let mut values = vec![];

						match args.next() {
							Some(super::Item::ExprString(str)) => {
								let split = str.split("{}");
								for (k, s) in split.enumerate() {
									strings.push(s.to_owned());
									if k != 0 {
										replacements.push(k as u16);
									}
								}
							},
							other => panic!("First arg should be a string: {other:?}")
						}

						for value in args {
							values.push(value);
						}

						super::Item::ExprFString {
							strings,
							replacements,
							values
						}
					} else {
						todo!("Unsupported macro: {path}")
					}
				},

				unk => todo!("unknown expr: {unk:#?}")
			}
		}

		let nodes: Vec<super::Item> = syn_ast
			.items
			.into_iter()
			.map(syn_item_to_item)
			.collect();

		Ok(SynBackend { items: nodes })
	}

	fn items(&self) -> &Vec<super::Item> {
		&self.items
	}
}
