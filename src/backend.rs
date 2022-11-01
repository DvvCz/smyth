#[cfg(feature = "syn")]
mod syn;

#[cfg(feature = "syn")]
pub use self::syn::SynBackend as Backend;

#[cfg(feature = "venial")]
mod venial;

#[cfg(feature = "venial")]
pub use self::venial::VenialBackend as Backend;

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
	#[cfg(feature = "syn")]
	#[error("Internal Syn Error: {0}")]
	Syn(#[from] ::syn::Error),

	#[cfg(feature = "venial")]
	#[error("Internal Venial Error: {0}")]
	Venial(#[from] ::venial::Error),
}

pub type Result<T> = std::result::Result<T, BackendError>;

pub trait AST: Sized {
	fn generate(code: impl AsRef<[u8]>) -> Result<Self>;
	fn items(&self) -> &Vec<Item>;
}

#[derive(Debug)]
struct IfElif {
	condition: Box<Item>,
	stmts: Vec<Item>,

	elif: Vec<Self>,
	else_stmts: Option<Vec<Item>>,
}

#[derive(Debug)]
pub enum Item {
	FunctionDefinition {
		name: String,
		params: Vec<String>,
		stmts: Vec<Self>,
	},

	While {
		condition: Box<Self>,
		stmts: Vec<Self>,
	},
	IfElif(IfElif),

	VarSet {
		name: String,
		expr: Box<Self>,
	},
	VarDecl {
		name: String,
		expr: Box<Self>,
	},

	// Expressions
	ExprCall { func: Box<Self>, args: Vec<Self> },
	ExprIdent(String),

	ExprDecimal(f64),
	ExprInteger(i64),

	ExprString(String),
	ExprBool(bool),
}
