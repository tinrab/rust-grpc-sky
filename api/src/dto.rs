use bomboni_request::derive::Parse;
use bomboni_request::error::RequestResult;
// TODO: remove
use bomboni_request::parse::RequestParse;

use crate::error::UserError;
use crate::proto::{user_error::UserErrorReason, SignUpRequest};

#[derive(Debug, Clone, Parse)]
#[parse(source = SignUpRequest, write)]
pub struct SignUpRequestDto {
    #[parse(regex = r#"^[a-zA-Z][a-zA-Z0-9_]{2,15}$"#)]
    name: String,
    #[parse(convert = parse_password)]
    password: String,
}

mod parse_password {

    use super::*;

    pub fn parse(password: String) -> RequestResult<String> {
        if password.len() < 6 {
            return Err(UserError::new(UserErrorReason::InvalidPassword).into());
        }
        Ok(password)
    }

    pub fn write(password: String) -> String {
        password
    }
}

#[cfg(test)]
mod tests {
    use bomboni_request::{
        error::{CommonError, RequestError},
        parse::RequestParse,
    };

    use super::*;

    macro_rules! assert_common_error {
        ($expr:expr, $path:expr, $error:pat $(,)?) => {
            assert!(matches!(
                { $expr }.unwrap_err(),
                RequestError::Path(error)
                    if matches!(
                        error.error.as_any().downcast_ref::<CommonError>().unwrap(),
                        $error,
                    ) && error.path_to_string() == $path
            ));
        };
    }

    #[test]
    fn parse_sign_up() {
        let input = SignUpRequest {
            name: "tester".into(),
            password: "123456".into(),
        };
        let parsed = SignUpRequestDto::parse(input.clone()).unwrap();
        assert_eq!(&parsed.name, &input.name);
        assert_eq!(&parsed.password, &input.password);

        assert_common_error!(
            SignUpRequestDto::parse(SignUpRequest {
                name: "".into(),
                password: "123456".into(),
            }),
            "name",
            CommonError::RequiredFieldMissing,
        );
        assert_common_error!(
            SignUpRequestDto::parse(SignUpRequest {
                name: "_".into(),
                password: "123456".into(),
            }),
            "name",
            CommonError::InvalidStringFormat { .. },
        );

        assert!(matches!(
            SignUpRequestDto::parse(SignUpRequest {
                name: "tester".into(),
                password: "abc".into(),
            }).unwrap_err(),
            RequestError::Path(error)
                if matches!(
                    error.error.as_any().downcast_ref::<UserError>().unwrap().reason,
                    UserErrorReason::InvalidPassword,
                ) && error.path_to_string() == "password"
        ));
    }
}
