use bomboni_common::{btree_map_into, date_time::UtcDateTime, id::Id};
use bomboni_request::{
    derive::Parse,
    error::RequestResult,
    parse::helpers::id_convert,
    query::list::ListQuery,
    schema::{FieldMemberSchema, Schema, ValueType},
};
use prost::Name;

use crate::error::UserError;
use crate::proto::{
    user_error::UserErrorReason, ListPostsRequest, Post, PostRequest, SignUpRequest, User,
};

#[derive(Debug, Clone, Parse)]
#[parse(source = SignUpRequest, request, write)]
pub struct SignUpRequestDto {
    #[parse(regex = r#"^[a-zA-Z][a-zA-Z0-9_]{2,15}$"#)]
    pub name: String,
    #[parse(convert = parse_password)]
    pub password: String,
}

#[derive(Debug, Clone, Parse)]
#[parse(source = PostRequest, request, write)]
pub struct PostRequestDto {
    pub content: String,
}

#[derive(Debug, Clone, Parse)]
#[parse(source = ListPostsRequest, request, write)]
pub struct ListPostsRequestDto {
    #[parse(list_query)]
    pub query: ListQuery,
}

#[derive(Debug, Clone, Parse)]
#[parse(source = User, write)]
pub struct UserDto {
    #[parse(convert = id_convert)]
    pub id: Id,
    pub name: String,
}

#[derive(Debug, Clone, Parse)]
#[parse(source = Post, write)]
pub struct PostDto {
    #[parse(convert = id_convert)]
    pub id: Id,
    #[parse(timestamp)]
    pub create_time: Option<UtcDateTime>,
    pub content: String,
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

impl PostDto {
    pub fn get_schema() -> Schema {
        Schema {
            members: btree_map_into!(
                "id" => FieldMemberSchema::new_ordered(ValueType::String),
                "userId" => FieldMemberSchema::new(ValueType::String),
                "createTime" => FieldMemberSchema::new_ordered(ValueType::Timestamp),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use bomboni_request::{
        error::{CommonError, RequestError},
        parse::RequestParse,
    };

    use super::*;

    #[test]
    fn parse_sign_up() {
        let input = SignUpRequest {
            name: "tester".into(),
            password: "123456".into(),
        };
        let parsed = SignUpRequestDto::parse(input.clone()).unwrap();
        assert_eq!(&parsed.name, &input.name);
        assert_eq!(&parsed.password, &input.password);

        assert!(matches!(
            SignUpRequestDto::parse(SignUpRequest {
                name: "".into(),
                password: "123456".into(),
            }).unwrap_err(),
            RequestError::BadRequest { name, violations }
                if name == SignUpRequest::NAME
                    && matches!(
                        violations[0].error.as_any().downcast_ref::<CommonError>().unwrap(),
                        CommonError::RequiredFieldMissing,
                    ) && violations[0].path_to_string() == "name",
        ));

        assert!(matches!(
            SignUpRequestDto::parse(SignUpRequest {
                name: "_".into(),
                password: "123456".into(),
            }).unwrap_err(),
            RequestError::BadRequest { name, violations }
            if name == SignUpRequest::NAME
                && matches!(
                    violations[0].error.as_any().downcast_ref::<CommonError>().unwrap(),
                    CommonError::InvalidStringFormat { .. },
                ) && violations[0].path_to_string() == "name",
        ));

        assert!(matches!(
            SignUpRequestDto::parse(SignUpRequest {
                name: "tester".into(),
                password: "abc".into(),
            }).unwrap_err(),
            RequestError::BadRequest { name, violations }
            if name == SignUpRequest::NAME
                && matches!(
                    violations[0].error.as_any().downcast_ref::<UserError>().unwrap().reason,
                    UserErrorReason::InvalidPassword,
                ) && violations[0].path_to_string() == "password",
        ));
    }
}
