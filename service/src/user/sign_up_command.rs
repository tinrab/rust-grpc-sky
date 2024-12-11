use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use bomboni_common::id::Id;
use bomboni_request::error::CommonError;

use grpc_sky_api::{error::UserError, proto::user_error::UserErrorReason};

use crate::{
    error::AppResult,
    user::repository::{UserInsertRecord, UserRepositoryArc},
};

pub struct SignUpCommand {
    user_repository: UserRepositoryArc,
}

pub struct SignUpCommandInput<'a> {
    pub name: &'a str,
    pub password: &'a str,
}

pub struct SignUpCommandOutput {
    pub user_id: Id,
}

impl SignUpCommand {
    pub fn new(user_repository: UserRepositoryArc) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, input: SignUpCommandInput<'_>) -> AppResult<SignUpCommandOutput> {
        // TODO: transaction
        if self
            .user_repository
            .select_by_name(input.name)
            .await?
            .is_some()
        {
            return Err(CommonError::AlreadyExists.into());
        }

        if input.password.len() < 6 {
            return Err(UserError::new(UserErrorReason::InvalidPassword).into());
        }

        let user_id = Id::generate();

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(input.password.as_bytes(), &salt)
            .map_err(|_| UserError::new(UserErrorReason::InvalidPassword))?
            .to_string();

        self.user_repository
            .insert(UserInsertRecord {
                id: user_id,
                name: input.name.into(),
                password_hash,
            })
            .await?;

        Ok(SignUpCommandOutput { user_id })
    }
}
