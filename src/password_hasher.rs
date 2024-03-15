use argon2::Argon2;

pub struct PasswordHasher<'a> {
    argon2: Argon2<'a>,
    salt: Box<[u8]>,
}

impl<'a> PasswordHasher<'a> {
    #[inline]
    pub(crate) fn new(salt: Box<[u8]>) -> Self {
        Self {
            argon2: Argon2::default(),
            salt,
        }
    }

    #[inline]
    pub(crate) fn hash(&self, password: &str) -> PasswordHash {
        let mut output = [0u8; 32];

        self.argon2
            .hash_password_into(password.as_bytes(), &self.salt, &mut output)
            .unwrap();

        PasswordHash(output)
    }

    #[inline]
    pub(crate) fn is_match(&self, hash: PasswordHash, password: &str) -> bool {
        let mut output = [0u8; 32];

        self.argon2
            .hash_password_into(password.as_bytes(), &self.salt, &mut output)
            .unwrap();

        output == hash.0
    }
}

pub struct PasswordHash([u8; 32]);
