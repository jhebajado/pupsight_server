mod users;

use std::sync::Arc;

use base64::prelude::{Engine, BASE64_STANDARD};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

use crate::messages::users::LoginUserResult;
use crate::password_hasher::PasswordHasher;
use crate::{messages, schema};

pub(crate) struct Database {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl Database {
    #[inline]
    pub(crate) async fn new(url: &str) -> Self {
        let manager = ConnectionManager::new(url);
        let pool = Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .expect("Could not build connection pool");

        Self { pool }
    }

    #[inline]
    pub(crate) async fn register(
        &self,
        hasher: Arc<PasswordHasher<'static>>,
        desc: messages::users::RegisterUser,
    ) -> messages::users::RegisterUserResult {
        let record = users::UserInsert {
            login_name: desc.login_name,
            first_name: desc.first_name,
            last_name: desc.last_name,
            argon2: hasher.hash(&desc.password),
        };

        let mut connection = self.pool.get().unwrap();

        let result = diesel::insert_into(schema::users::table)
            .values(&record)
            .returning(schema::users::id)
            .get_result::<uuid::Uuid>(&mut connection);

        match result {
            Ok(id) => messages::users::RegisterUserResult::Success { id },
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )) => messages::users::RegisterUserResult::LoginNameAlreadyExists,
            Err(_) => messages::users::RegisterUserResult::ServerError,
        }
    }

    #[inline]
    pub(crate) async fn login(
        &self,
        hasher: Arc<PasswordHasher<'static>>,
        desc: messages::users::LoginUser,
    ) -> LoginUserResult {
        use crate::schema::{session, users};

        let mut connection = self.pool.get().expect("Unable to connect to database");

        let hash = hasher.hash(&desc.password);

        match users::table
            .filter(users::login_name.eq(desc.login_name))
            .select((users::id, users::argon2.eq(hash)))
            .first::<(uuid::Uuid, bool)>(&mut connection)
        {
            Ok((user_id, is_matched)) => {
                if is_matched {
                    match diesel::insert_into(session::table)
                        .values(session::user_id.eq(user_id))
                        .returning((session::id, session::access_token))
                        .get_result::<(uuid::Uuid, Vec<u8>)>(&mut connection)
                    {
                        Ok((session_id, access_token)) => LoginUserResult::Success {
                            id: session_id,
                            access_token: BASE64_STANDARD.encode(access_token),
                        },
                        Err(_) => LoginUserResult::ServerError,
                    }
                } else {
                    LoginUserResult::Invalid
                }
            }
            Err(diesel::result::Error::NotFound) => LoginUserResult::Invalid,
            Err(_) => LoginUserResult::ServerError,
        }
    }

    #[inline]
    pub(crate) async fn get_user_session(
        &self,
        session_id: uuid::Uuid,
        access_token: Vec<u8>,
    ) -> Result<users::UserSession, users::AuthorizationError> {
        use crate::schema::{session, users};

        let mut connection = self.pool.get().expect("Unable to connect to database");

        let (matched, user_id, login_name, first_name, last_name): (
            bool,
            uuid::Uuid,
            String,
            String,
            String,
        ) = users::table
            .inner_join(session::table)
            .filter(session::id.eq(session_id))
            .select((
                session::access_token.eq(access_token),
                users::id,
                users::login_name,
                users::first_name,
                users::last_name,
            ))
            .get_result(&mut connection)
            .or(Err(self::users::AuthorizationError))?;

        if !matched {
            return Err(self::users::AuthorizationError);
        }

        Ok(self::users::UserSession {
            user_id,
            login_name,
            first_name,
            last_name,
        })
    }
}
