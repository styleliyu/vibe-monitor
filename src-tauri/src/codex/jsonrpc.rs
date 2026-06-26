use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcRequest<T> {
    pub id: u64,
    pub method: String,
    pub params: T,
}

pub fn encode_request<T: Serialize>(request: &JsonRpcRequest<T>) -> Result<String, AppError> {
    serde_json::to_string(request).map_err(|error| AppError::CommandFailed(error.to_string()))
}
