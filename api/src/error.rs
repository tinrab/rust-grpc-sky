use bomboni_proto::google::rpc::Status;
use bomboni_proto::google::{protobuf::Any, rpc::ErrorInfo};
use bomboni_request::error::{CommonError, GenericError, RequestError};
use paste::paste;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{self, Debug, Display, Formatter};
use thiserror::Error;

use crate::proto::common_error::CommonErrorReason;
use crate::proto::user_error::UserErrorReason;

#[derive(Error, Debug, PartialEq)]
#[error(transparent)]
pub enum SkyError {
    User(#[from] UserError),
}

pub type SkyResult<T> = Result<T, SkyError>;

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkyErrorMetadata {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "metadata_field_serde"
    )]
    pub user_name: Option<String>,
}

pub const COMMON_ERROR_DOMAIN: &str = "common.example.com";
pub const SKY_ERROR_DOMAIN: &str = "sky.example.com";

impl GenericError for SkyError {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Custom field serde to handle optional fields.
mod metadata_field_serde {
    use std::str::FromStr;

    use super::*;

    pub fn serialize<F, S>(value: &Option<F>, serializer: S) -> Result<S::Ok, S::Error>
    where
        F: ToString,
        S: serde::Serializer,
    {
        if let Some(value) = value {
            serializer.serialize_str(&value.to_string())
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, F, D>(
        deserializer: D,
    ) -> Result<Option<F>, <D as serde::Deserializer<'de>>::Error>
    where
        F: FromStr,
        D: serde::Deserializer<'de>,
    {
        let value = Option::<String>::deserialize(deserializer)?;
        if let Some(value) = value {
            F::from_str(&value)
                .map(Some)
                .map_err(|_| serde::de::Error::custom("failed to parse metadata field"))
        } else {
            Ok(None)
        }
    }
}

impl SkyErrorMetadata {
    pub fn to_map(&self) -> BTreeMap<String, String> {
        let value = serde_json::to_value(self).unwrap();
        serde_json::from_value(value).unwrap()
    }

    pub fn from_map(map: BTreeMap<String, String>) -> Option<Self> {
        let value = serde_json::to_value(map).ok()?;
        serde_json::from_value(value).ok()
    }
}

impl Display for SkyErrorMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.to_map().fmt(f)
    }
}

/// Debug that does not print empty fields.
impl Debug for SkyErrorMetadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut d = f.debug_struct("SkyErrorMetadata");

        macro_rules! debug_fields {
            ($($field:ident),* $(,)?) => {
                $(
                    if let Some(value) = &self.$field {
                        d.field(stringify!($field), value);
                    }
                )*
            };
        }

        debug_fields![user_name];

        Ok(())
    }
}

pub enum SkyErrorReason {
    Common(CommonErrorReason),
    User(UserErrorReason),
}

macro_rules! impl_sky_error_reason_variants {
    ($( ($variant:ident, $type:ty) $(,)? )* ) => {
        $(
            impl From<$type> for SkyErrorReason {
                fn from(value: $type) -> Self {
                    SkyErrorReason::$variant(value)
                }
            }
        )*
    };
}

impl_sky_error_reason_variants![(Common, CommonErrorReason), (User, UserErrorReason)];

macro_rules! convert_sky_error_reason {
    ($reason:ident, $type:ty, $kind:ident) => {{
        let parsed_reason: SkyErrorReason = $reason.into();
        let mut common_reason = CommonErrorReason::Unspecified;
        let mut domain_reason = <$type>::Unspecified;
        match parsed_reason {
            SkyErrorReason::Common(parsed_reason) => {
                common_reason = parsed_reason;
            }
            SkyErrorReason::$kind(parsed_reason) => {
                domain_reason = parsed_reason;
            }
        }
        (domain_reason, common_reason)
    }};
}

macro_rules! impl_domain_error {
    ($name:ident) => {
        paste! {
            #[derive(Error, Debug, Clone, PartialEq)]
            pub struct [<$name Error>] {
                pub reason: [<$name ErrorReason>],
                pub message: Option<String>,
                pub common_reason: CommonErrorReason,
                pub common_error: Option<CommonError>,
                pub metadata: Option<SkyErrorMetadata>,
            }
        }

        paste! {
            pub type [<$name Result>]<T> = Result<T, [<$name Error>]>;
        }

        // Constructors
        paste! {
            #[automatically_derived]
            #[allow(dead_code)]
            impl [<$name Error>] {
                pub fn new<R: Into<SkyErrorReason>>(reason: R) -> Self {
                    let (reason, common_reason) =
                        convert_sky_error_reason!(reason, [<$name ErrorReason>], $name);
                    Self {
                        reason,
                        common_reason,
                        metadata: None,
                        message: None,
                        common_error: None,
                    }
                }

                pub fn new_common(common_error: CommonError) -> Self {
                    [<$name Error>] {
                        reason: [<$name ErrorReason>]::Unspecified,
                        common_reason: get_common_error_reason(&common_error),
                        metadata: None,
                        message: None,
                        common_error: Some(common_error),
                    }
                }

                pub fn new_with_metadata<R: Into<SkyErrorReason>>(
                    reason: R,
                    metadata: SkyErrorMetadata,
                ) -> Self {
                    let (reason, common_reason) =
                        convert_sky_error_reason!(reason, [<$name ErrorReason>], $name);
                    Self {
                        reason,
                        common_reason,
                        metadata: Some(metadata),
                        message: None,
                        common_error: None,
                    }
                }

                pub fn new_with_message<R: Into<SkyErrorReason>, S: ToString>(
                    reason: R,
                    message: S,
                ) -> Self {
                    let (reason, common_reason) =
                        convert_sky_error_reason!(reason, [<$name ErrorReason>], $name);
                    Self {
                        reason,
                        common_reason,
                        metadata: None,
                        message: Some(message.to_string()),
                        common_error: None,
                    }
                }

                pub fn with_reason(mut self, reason: [<$name ErrorReason>]) -> Self {
                    self.reason = reason;
                    self
                }

                pub fn with_message<S: ToString>(mut self, message: S) -> Self {
                    self.message = Some(message.to_string());
                    self
                }

                fn modify_metadata<F: FnMut(&mut SkyErrorMetadata)>(
                    mut self,
                    mut f: F,
                ) -> Self {
                    self.metadata = if let Some(mut metadata) = self.metadata {
                        f(&mut metadata);
                        Some(metadata)
                    } else {
                        let mut metadata = SkyErrorMetadata::default();
                        f(&mut metadata);
                        Some(metadata)
                    };
                    self
                }
            }
        }

        impl Display for paste! { [<$name Error>] } {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                if self.common_reason != CommonErrorReason::Unspecified {
                    write!(f, "{}", self.common_reason.as_str_name())?;
                } else {
                    write!(f, "{}", self.reason.as_str_name())?;
                }

                if let Some(message) = self.message.as_ref() {
                    write!(f, ": {}", message)?;
                } else if let Some(common_error) = self.common_error.as_ref() {
                    write!(f, ": {}", common_error)?;
                } else if let Some(metadata) = self.metadata.as_ref() {
                    write!(f, ": {}", metadata)?;
                }
                Ok(())
            }
        }

        // For conversion between request errors
        impl GenericError for paste! { [<$name Error>] } {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn details(&self) -> Vec<Any> {
                let mut details = Vec::new();
                if self.common_reason != CommonErrorReason::Unspecified {
                    details.push(Any::pack_from(&ErrorInfo {
                        reason: self.common_reason.as_str_name().into(),
                        domain: COMMON_ERROR_DOMAIN.into(),
                        metadata: self.metadata.clone().unwrap_or_default().to_map(),
                    }).unwrap());
                }
                if self.reason != paste! { [<$name ErrorReason>]::Unspecified } {
                    details.push(
                        Any::pack_from(&ErrorInfo {
                            reason: self.reason.as_str_name().into(),
                            domain: format!("{}/{}", SKY_ERROR_DOMAIN, stringify!($name)),
                            metadata: self.metadata.clone().unwrap_or_default().to_map(),
                        })
                        .unwrap(),
                    );
                }
                details
            }
        }

        paste! {
            impl From<CommonError> for [<$name Error>] {
                fn from(common_error: CommonError) -> Self {
                    Self {
                        reason: [<$name ErrorReason>]::Unspecified,
                        message: None,
                        common_reason: get_common_error_reason(&common_error),
                        common_error: Some(common_error),
                        metadata: None,
                    }
                }
            }
        }

        // Serde and Status impls
        paste! {
            impl [<$name Error>] {
                pub fn into_status(self) -> Status {
                    Status::from(RequestError::generic(self))
                }

                pub fn from_status(mut status: Status) -> Option<Self> {
                    if status.details.is_empty() {
                        return None;
                    }
                    let error_info: ErrorInfo = Any::unpack_into(status.details.remove(0)).ok()?;
                    let metadata = if error_info.metadata.is_empty() {
                        None
                    } else {
                        SkyErrorMetadata::from_map(error_info.metadata)
                    };

                    if error_info.domain == COMMON_ERROR_DOMAIN {
                        let reason = CommonErrorReason::from_str_name(&error_info.reason)?;
                        return Some([<$name Error>] {
                            reason: [<$name ErrorReason>]::Unspecified,
                            common_reason: reason,
                            metadata,
                            message: None,
                            common_error: None,
                        });
                    } else if error_info.domain == format!("{}/{}", SKY_ERROR_DOMAIN, stringify!($name)) {
                        let reason = [<$name ErrorReason>]::from_str_name(&error_info.reason)?;
                        return Some([<$name Error>] {
                            reason,
                            common_reason: CommonErrorReason::Unspecified,
                            metadata,
                            message: None,
                            common_error: None,
                        });
                    }

                    None
                }
            }

            #[automatically_derived]
            impl Serialize for [<$name Error>] {
                fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                    Status::from(RequestError::generic(self.clone())).serialize(serializer)
                }
            }

            impl<'de> Deserialize<'de> for [<$name Error>] {
                fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    let status = Status::deserialize(deserializer)?;
                    Self::from_status(status)
                        .ok_or_else(|| serde::de::Error::custom("failed to deserialize GraphError from Status"))
                }
            }
        }
    };
}

impl_domain_error!(User);

pub fn get_common_error_reason(error: &CommonError) -> CommonErrorReason {
    match error {
        CommonError::ResourceNotFound => CommonErrorReason::ResourceNotFound,
        CommonError::Unauthorized => CommonErrorReason::Unauthorized,
        CommonError::RequiredFieldMissing => CommonErrorReason::RequiredFieldMissing,
        CommonError::InvalidName { .. } => CommonErrorReason::InvalidName,
        CommonError::InvalidNameAlternative { .. } => CommonErrorReason::InvalidName,
        CommonError::InvalidParent { .. } => CommonErrorReason::InvalidParent,
        CommonError::InvalidStringFormat { .. } => CommonErrorReason::InvalidStringFormat,
        CommonError::InvalidId => CommonErrorReason::InvalidId,
        CommonError::DuplicateId => CommonErrorReason::DuplicateId,
        CommonError::InvalidDisplayName => CommonErrorReason::InvalidDisplayName,
        CommonError::InvalidDateTime => CommonErrorReason::InvalidDateTime,
        CommonError::InvalidEnumValue => CommonErrorReason::InvalidEnumValue,
        CommonError::UnknownOneofVariant => CommonErrorReason::UnknownOneofVariant,
        CommonError::InvalidNumericValue => CommonErrorReason::InvalidNumericValue,
        CommonError::FailedConvertValue => CommonErrorReason::FailedConvertValue,
        CommonError::NumericOutOfRange => CommonErrorReason::NumericOutOfRange,
        CommonError::DuplicateValue => CommonErrorReason::DuplicateValue,
        CommonError::AlreadyExists => CommonErrorReason::AlreadyExists,
        CommonError::NotFound => CommonErrorReason::NotFound,
        CommonError::TypeMismatch => CommonErrorReason::TypeMismatch,
    }
}

macro_rules! impl_sky_metadata_field {
    ($ident:ident, $type:ty, $convert:ident) => {
        paste! {
            pub fn $ident<R: Into<SkyErrorReason>>(value: $type, reason: R) -> Self {
                Self::new_with_metadata(
                    reason,
                    SkyErrorMetadata {
                        $ident: Some( value.$convert () ),
                        ..Default::default()
                    },
                )
            }
            #[must_use]
            pub fn [<with_ $ident>] (self, value: $type) -> Self {
                self.modify_metadata(|metadata| {
                    // Don't overwrite existing metadata
                    if metadata.$ident.is_none() {
                        metadata.$ident = Some( value.$convert () );
                    }
                })
            }
        }
    };
}

impl UserError {
    impl_sky_metadata_field!(user_name, &str, into);
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn it_works() {
        let err = UserError::new(UserErrorReason::InvalidName).with_user_name("tester");
        assert_eq!(
            err.to_string(),
            r#"USER_ERROR_REASON_INVALID_NAME: {"userName": "tester"}"#
        );

        assert_eq!(
            serde_json::to_value(&err.clone().into_status()).unwrap(),
            json!({
                "code": "INVALID_ARGUMENT",
                "message": r#"USER_ERROR_REASON_INVALID_NAME: {"userName": "tester"}"#,
                "details": [
                    {
                        "@type": "type.googleapis.com/google.rpc.ErrorInfo",
                        "reason": "USER_ERROR_REASON_INVALID_NAME",
                        "domain": "sky.example.com/User",
                        "metadata": {
                            "userName": "tester"
                        }
                    }
                ],
            }),
        );
    }
}
