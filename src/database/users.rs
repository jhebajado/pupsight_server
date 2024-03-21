use diesel::Insertable;

use crate::schema::users;

use crate::password_hasher::PasswordHash;

#[derive(Clone, Debug, PartialEq, Eq, Insertable)]
#[diesel(table_name = users)]
pub(crate) struct UserInsert {
    pub(crate) login_name: String,
    pub(crate) first_name: String,
    pub(crate) last_name: String,
    pub(crate) argon2: PasswordHash,
}
