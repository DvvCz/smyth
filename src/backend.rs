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
pub struct IfElif {
	pub condition: Box<Item>,
	pub stmts: Vec<Item>,

	pub elif: Vec<(Item, Vec<Item>)>,
	pub else_stmts: Option<Vec<Item>>,
}

#[derive(Debug)]
pub enum BinaryOp {
	Add,
	Sub,
	Mul,
	Div,

	Mod,
	And,
	Or,

	Bxor,
	Band,
	Bor,
	Bshl,
	Bshr,

	Eq,
	Lt,
	Le,
	Ne,
	Ge,
	Gt,

	AddEq,
	SubEq,
	MulEq,
	DivEq,

	ModEq,

	BxorEq,
	BandEq,
	BorEq,
	BshlEq,
	BshrEq,
}

#[cfg(feature = "syn")]
impl From<::syn::BinOp> for BinaryOp {
	fn from(x: ::syn::BinOp) -> Self {
		match x {
			::syn::BinOp::Add(_) => Self::Add,
			::syn::BinOp::Sub(_) => Self::Sub,
			::syn::BinOp::Mul(_) => Self::Mul,
			::syn::BinOp::Div(_) => Self::Div,
			::syn::BinOp::Rem(_) => Self::Mod,
			::syn::BinOp::And(_) => Self::And,
			::syn::BinOp::Or(_) => Self::Or,
			::syn::BinOp::BitXor(_) => Self::Bxor,
			::syn::BinOp::BitAnd(_) => Self::Band,
			::syn::BinOp::BitOr(_) => Self::Bor,
			::syn::BinOp::Shl(_) => Self::Bshl,
			::syn::BinOp::Shr(_) => Self::Bshr,
			::syn::BinOp::Eq(_) => Self::Eq,
			::syn::BinOp::Lt(_) => Self::Lt,
			::syn::BinOp::Le(_) => Self::Le,
			::syn::BinOp::Ne(_) => Self::Ne,
			::syn::BinOp::Ge(_) => Self::Ge,
			::syn::BinOp::Gt(_) => Self::Gt,
			::syn::BinOp::AddEq(_) => Self::AddEq,
			::syn::BinOp::SubEq(_) => Self::SubEq,
			::syn::BinOp::MulEq(_) => Self::MulEq,
			::syn::BinOp::DivEq(_) => Self::DivEq,
			::syn::BinOp::RemEq(_) => Self::ModEq,
			::syn::BinOp::BitXorEq(_) => Self::BxorEq,
			::syn::BinOp::BitAndEq(_) => Self::BandEq,
			::syn::BinOp::BitOrEq(_) => Self::BorEq,
			::syn::BinOp::ShlEq(_) => Self::BshlEq,
			::syn::BinOp::ShrEq(_) => Self::BshrEq,
		}
	}
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

	ForIn {
		// for {var} in {expr}
		var: String,
		expr: Box<Self>,
		stmts: Vec<Self>
	},

	ForRange {
		// for i = 1, 2, 3
		var: String,
		min: Box<Self>,
		max: Box<Self>,
		jump: Option<Box<Self>>,
		stmts: Vec<Self>
	},

	// C style for loops are equivalent to while.

	IfElif(IfElif),

	VarSet {
		name: String,
		expr: Box<Self>,
	},
	VarDecl {
		name: String,
		expr: Box<Self>,
	},

	Break,
	Continue,

	Externs {
		functions: Vec<String>,
	},

	Mod {
		name: String,
		items: Vec<Self>,
	},

	// Expressions
	ExprCall {
		func: Box<Self>,
		args: Vec<Self>,
	},
	ExprIdent(String),

	ExprDecimal(f64),
	ExprInteger(i64),

	ExprString(String),
	ExprBool(bool),

	ExprClosure {
		params: Vec<String>,
		stmts: Vec<Self>,
	},

	ExprArray {
		elements: Vec<Self>,
	},

	ExprBinary {
		lhs: Box<Self>,
		rhs: Box<Self>,
		op: BinaryOp,
	},

	ExprFString {
		strings: Vec<String>,

		// Vector of numbers, which correspond to which value to insert in the string gap.
		replacements: Vec<u16>,

		// Expressions to insert inside of the string
		values: Vec<Self>,
	},
}
