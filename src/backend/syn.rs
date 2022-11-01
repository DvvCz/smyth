#[derive(Debug)]
pub struct SynBackend {
	pub items: Vec<super::Item>,
}

impl super::AST for SynBackend {
	fn generate(code: impl AsRef<[u8]>) -> super::Result<Self> {
		let code = code.as_ref();
		let code = std::str::from_utf8(code).unwrap();

		let syn_ast = syn::parse_file(code)?;

		fn syn_item_to_item(item: syn::Item) -> super::Item {
			match item {
				syn::Item::Fn(data) => {
					use syn::FnArg;

					let param_names: Vec<String> = data
						.sig
						.inputs
						.into_iter()
						.map(|arg| match arg {
							FnArg::Receiver(_) => String::from("self"),
							FnArg::Typed(data) => match data.pat.as_ref() {
								syn::Pat::Ident(name) => name.ident.to_string(),
								_ => todo!(),
							},
						})
						.collect();

					super::Item::FunctionDefinition {
						name: data.sig.ident.to_string(),
						params: param_names,
						stmts: data.block.stmts.into_iter().map(stmt_to_item).collect(),
					}
				}

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

				syn::Expr::Call(syn::ExprCall { func, args, .. }) => {
					super::Item::ExprCall { func: Box::new(expr_to_item(*func)), args: args.into_iter().map(expr_to_item).collect() }
				},

				syn::Expr::Path(syn::ExprPath { path, .. }) => {
					let path_ident = path.segments.iter().map(|x| x.ident.to_string()).collect::<Vec<String>>().join("__");
					super::Item::ExprIdent(path_ident)
				},

				unk => {
					dbg!(unk);
					todo!()
				},
			}
		}

		let nodes: Vec<super::Item> = syn_ast
			.items
			.into_iter()
			.map(|x| syn_item_to_item(x))
			.collect();

		Ok(SynBackend { items: nodes })
	}

	fn items(&self) -> &Vec<super::Item> {
		&self.items
	}
}
