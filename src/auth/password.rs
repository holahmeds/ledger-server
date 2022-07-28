use argon2::Config;

pub fn encode_password(password: String) -> Result<String, argon2::Error> {
    let config = Config::default();
    let salt: [u8; 32] = rand::random();
    let password_hash = argon2::hash_encoded(password.as_bytes(), &salt, &config)?;
    Ok(password_hash)
}

pub fn verify_password(password: String, password_hash: String) -> Result<bool, argon2::Error> {
    argon2::verify_encoded(&password_hash, password.as_bytes())
}
