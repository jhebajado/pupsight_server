use argon2::Argon2;

use diesel::deserialize::FromSql;
use diesel::expression::AsExpression;
use diesel::pg::Pg;
use diesel::serialize::ToSql;
use diesel::sql_types::Binary;

use serde::Deserialize;

pub(crate) struct PasswordHasher<'a> {
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
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, AsExpression)]
#[diesel(sql_type = diesel::sql_types::Binary)]
pub(crate) struct PasswordHash([u8; 32]);

impl<'a> ToSql<Binary, Pg> for PasswordHash
where
    &'a [u8]: ToSql<Binary, Pg>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        <[u8] as ToSql<Binary, Pg>>::to_sql(self.0.as_ref(), out)
    }
}

impl FromSql<Binary, Pg> for PasswordHash
where
    Vec<u8>: FromSql<Binary, Pg>,
{
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        if let Ok(arr) = <Vec<u8> as FromSql<Binary, Pg>>::from_sql(bytes)?.try_into() {
            return Ok(Self(arr));
        }

        Err("Cannot convert to PasswordHash".into())
    }
}
