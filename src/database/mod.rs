mod samples;
mod users;

use std::sync::Arc;

use base64::prelude::{Engine, BASE64_STANDARD};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgConnection, PgTextExpressionMethods, QueryDsl,
    RunQueryDsl,
};
use image::DynamicImage;

use crate::detector::Classification;
use crate::messages::samples::SampleUploadResult;
use crate::messages::users::LoginUserResult;
use crate::password_hasher::PasswordHasher;
pub(crate) use samples::SampleInsert;
pub(crate) use users::UserSession;

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

    #[inline]
    pub(crate) async fn upload_samples(
        &self,
        samples: Vec<samples::SampleInsert>,
    ) -> SampleUploadResult {
        use crate::schema::samples;

        let mut connection = self.pool.get().expect("Unable to connect to database");

        match diesel::insert_into(samples::table)
            .values(samples)
            .execute(&mut connection)
        {
            Ok(_) => SampleUploadResult::Success,
            Err(_) => SampleUploadResult::Failed,
        }
    }

    #[inline]
    pub(crate) async fn get_sample_image(
        &self,
        sample_id: uuid::Uuid,
    ) -> messages::samples::SampleImageResult {
        use crate::schema::samples;

        let mut connection = self.pool.get().expect("Unable to connect to database");

        match samples::table
            .filter(samples::id.eq(sample_id))
            .select(samples::bytes)
            .first::<Vec<u8>>(&mut connection)
        {
            Ok(bytes) => messages::samples::SampleImageResult::Success { bytes },
            Err(diesel::result::Error::NotFound) => messages::samples::SampleImageResult::NotFound,
            Err(_) => messages::samples::SampleImageResult::ServerError,
        }
    }

    #[inline]
    pub(crate) async fn infer_sample_image(
        &self,
        owner_id: uuid::Uuid,
        sample_id: uuid::Uuid,
        detector: &crate::Detector,
    ) -> messages::samples::SampleInferResult {
        use crate::schema::{results, samples};

        let mut connection = self.pool.get().expect("Unable to connect to database");

        let img = match samples::table
            .filter(
                samples::id
                    .eq(sample_id)
                    .and(samples::owner_id.eq(owner_id)),
            )
            .select(samples::bytes)
            .first::<Vec<u8>>(&mut connection)
        {
            Ok(bytes) => {
                if let Ok(img) =
                    image::load_from_memory_with_format(&bytes, image::ImageFormat::WebP)
                {
                    img
                } else {
                    return messages::samples::SampleInferResult::ImageLoadError;
                }
            }
            Err(diesel::result::Error::NotFound) => {
                return messages::samples::SampleInferResult::NotFound
            }
            Err(_) => return messages::samples::SampleInferResult::ServerError,
        };

        let boxes = detector.infer(&img).await;

        let result: Vec<self::samples::ResultInsert> = boxes
            .into_iter()
            .map(|entry| self::samples::ResultInsert {
                sample_id,
                certainty: entry.probability,
                is_normal: entry.classification == crate::detector::Classification::Normal,
                x: entry.x,
                y: entry.y,
                width: entry.width,
                height: entry.height,
            })
            .collect();

        match diesel::insert_into(results::table)
            .values(result)
            .execute(&mut connection)
        {
            Ok(_) => messages::samples::SampleInferResult::Success,
            Err(_) => messages::samples::SampleInferResult::ServerError,
        }
    }

    #[inline]
    pub(crate) async fn get_sample_list(
        &self,
        user_id: uuid::Uuid,
        desc: messages::samples::SamplePendingList,
    ) -> messages::samples::PendingListResult {
        use crate::schema::samples;

        let mut connection = self.pool.get().expect("Unable to connect to database");

        match samples::table
            .filter(
                samples::owner_id
                    .eq(user_id)
                    .and(samples::pet_id.is_null())
                    .and(samples::label.ilike(if let Some(search) = desc.keyword {
                        format!("%{}%", search)
                    } else {
                        "%".to_string()
                    })),
            )
            .select((samples::id, samples::label, samples::pet_id))
            .limit(10)
            .offset(desc.page as i64)
            .order(samples::created_at.desc())
            .get_results::<self::samples::SampleEntry>(&mut connection)
        {
            Ok(items) => {
                let items: Vec<self::samples::SampleEntry> = items;

                messages::samples::PendingListResult::Success {
                    items: items
                        .iter()
                        .map(|entry| self::messages::samples::PendingListEntry {
                            id: entry.id,
                            label: entry.label.clone(),
                            pet_id: entry.pet_id,
                        })
                        .collect(),
                    has_next: items.len() == 10,
                }
            }
            _ => messages::samples::PendingListResult::Failed,
        }
    }
}
