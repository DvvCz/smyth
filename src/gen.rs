pub mod lua;

#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
	#[cfg(feature = "syn")]
	#[error("Internal Syn Error: {0}")]
	Syn(#[from] ::syn::Error),

	#[cfg(feature = "venial")]
	#[error("Internal Venial Error: {0}")]
	Venial(#[from] ::venial::Error),
}

pub type Result<T> = std::result::Result<T, CodegenError>;

pub trait CodeGenerator: Sized {
	fn generate(&self, ast: &impl crate::backend::AST) -> Result<String>;
}
