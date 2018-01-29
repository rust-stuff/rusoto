//! Provides a way to create static/programmatically generated AWS Credentials.
//! For those who can't get them from an environment, or a file.

use chrono::{Duration, Utc};
use futures::future::{FutureResult, ok};

use {AwsCredentials, CredentialsError, ProvideAwsCredentials};

/// Provides AWS credentials from statically/programmatically provided strings.
#[derive(Clone, Debug)]
pub struct StaticProvider {
    /// The AWS Access Key ID to use for authenticating to AWS.
    aws_access_key_id: String,
    /// The AWS Secret Access Key to use for authenticating to AWS.
    aws_secret_access_key: String,
    /// The optional token to use for authenticating to AWS.
    token: Option<String>,
    /// The time in seconds each issued token should be valid for.
    valid_for: Option<i64>,
}

impl StaticProvider {
    /// Creates a new Static Provider. This should be used when you want to statically, or programmatically
    /// provide access to AWS.
    pub fn new(
        access_key: String,
        secret_access_key: String,
        token: Option<String>,
        valid_for: Option<i64>,
    ) -> StaticProvider {
        StaticProvider {
            aws_access_key_id: access_key,
            aws_secret_access_key: secret_access_key,
            token: token,
            valid_for: valid_for,
        }
    }

    /// Creates a new minimal Static Provider. This will set the token as optional none.
    pub fn new_minimal(access_key: String, secret_access_key: String) -> StaticProvider {
        StaticProvider {
            aws_access_key_id: access_key,
            aws_secret_access_key: secret_access_key,
            token: None,
            valid_for: None,
        }
    }

    /// Gets the AWS Access Key ID for this Static Provider.
    pub fn get_aws_access_key_id(&self) -> &str {
        &self.aws_access_key_id
    }

    /// Gets the AWS Secret Access Key for this Static Provider.
    pub fn get_aws_secret_access_key(&self) -> &str {
        &self.aws_secret_access_key
    }

    /// Determines if this Static Provider was given a Token.
    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }

    /// Gets The Token this Static Provider was given.
    pub fn get_token(&self) -> &Option<String> {
        &self.token
    }

    /// Returns the length in seconds this Static Provider will be valid for.
    pub fn is_valid_for(&self) -> &Option<i64> {
        &self.valid_for
    }
}

impl ProvideAwsCredentials for StaticProvider {
    type Future = FutureResult<AwsCredentials, CredentialsError>;

    fn credentials(&self) -> Self::Future {
        ok(AwsCredentials::new(
            self.aws_access_key_id.clone(),
            self.aws_secret_access_key.clone(),
            self.token.clone(),
            self.valid_for.map(|v| Utc::now() + Duration::seconds(v)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use futures::Future;

    use ProvideAwsCredentials;
    use super::*;

    #[test]
    fn test_static_provider_creation() {
        let result = StaticProvider::new(
            "fake-key".to_owned(),
            "fake-secret".to_owned(),
            Some("token".to_owned()),
            Some(300),
        ).credentials().wait();
        assert!(result.is_ok());
    }

    #[test]
    fn test_static_provider_minimal_creation() {
        let result =
            StaticProvider::new_minimal("fake-key-2".to_owned(), "fake-secret-2".to_owned())
                .credentials().wait();
        assert!(result.is_ok());
    }

    #[test]
    fn test_static_provider_custom_time_expiration() {
        let start_time = Utc::now();
        let result = StaticProvider::new(
            "fake-key".to_owned(),
            "fake-secret".to_owned(),
            None,
            Some(10000),
        ).credentials().wait();
        assert!(result.is_ok());
        let finalized = result.unwrap();
        let expires_at = finalized.expires_at().unwrap().clone();

        // Give a wide range of time, just incase there's somehow an immense amount of lag.
        assert!(start_time + Duration::minutes(100) < expires_at);
        assert!(expires_at < start_time + Duration::minutes(200));
    }
}
