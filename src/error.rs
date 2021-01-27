use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
	#[error("Invalid selector:'{context}'<{reason}>")]
	InvalidSelector { context: String, reason: String },
	#[error("Call method '{method}' with {error}")]
	MethodOnInvalidSelector { method: String, error: String },
}
