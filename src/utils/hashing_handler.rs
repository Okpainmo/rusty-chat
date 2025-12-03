use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};

pub async fn hashing_handler(string_to_hash: &str) -> Result<String, argon2::password_hash::Error> {
    // Generate a random 16-byte salt
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2.hash_password(string_to_hash.as_bytes(), &salt)?;

    Ok(password_hash.to_string())
}
