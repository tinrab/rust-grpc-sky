use std::str::FromStr;

use bomboni_common::id::Id;
use bomboni_request::error::CommonError;
use tonic::Request;

use crate::error::AppResult;

pub struct Context {
    user_id: Option<Id>,
}

const AUTHORIZATION_METADATA_KEY: &str = "authorization";

impl Context {
    pub fn from_request<T>(request: &Request<T>) -> Self {
        // TODO: Make actual auth
        let user_id = request
            .metadata()
            .get(AUTHORIZATION_METADATA_KEY)
            .and_then(|auth| {
                let bearer = auth.to_str().ok()?;
                let token = bearer.strip_prefix("Bearer ")?;
                Id::from_str(token).ok()
            });
        Self { user_id }
    }

    pub fn authenticate(&self) -> AppResult<Id> {
        self.user_id.ok_or(CommonError::Unauthorized.into())
    }
}
