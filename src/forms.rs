use regex::Regex;
use rocket::{
    serde::{Deserialize, Serialize},
    FromForm,
};
use sha2::{Digest, Sha256};

#[derive(FromForm, Serialize, Deserialize)]
pub struct UserCredentials<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

impl<'a> UserCredentials<'a> {
    /// Check if the given email appears to conform to the address format for RFC5322
    pub fn is_valid_email(&self) -> bool {
        let email_regex = Regex::new("\
(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|\"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\
\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*\")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\\.\
)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\\.){3}(?:\
(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\
-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\\])").expect("Input is valid regex");

        email_regex.is_match(self.email)
    }

    pub fn check_password_for_issues(&self) -> Option<&'static str> {
        let character_count = self.password.chars().count();
        if character_count < 8 {
            return Some("Password must be at least 8 characters");
        }

        if character_count > 512 {
            return Some("Password can not be more than 512 characters");
        }

        None
    }

    pub fn password_hash(&self) -> [u8; 32] {
        const SALT: [u8; 8] = [242, 94, 145, 122, 201, 1, 131, 203];

        let mut hasher = Sha256::new();
        hasher.update(SALT);
        hasher.update(self.password);

        hasher.finalize().into()
    }
}
