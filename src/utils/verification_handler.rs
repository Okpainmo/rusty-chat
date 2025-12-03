use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};

pub async fn verification_handler(
    string_to_compare: &str,
    hashed_string: &str,
) -> Result<bool, argon2::password_hash::Error> {
    // Parse the stored hash
    let parsed_hash = PasswordHash::new(hashed_string)?;

    // Verify the password
    let is_valid = Argon2::default()
        .verify_password(string_to_compare.as_bytes(), &parsed_hash)
        .is_ok(); // returns true if verification succeeded

    Ok(is_valid)
}
