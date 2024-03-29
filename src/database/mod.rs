mod samples;
mod users;

use std::collections::HashMap;
use std::sync::Arc;

use base64::prelude::{Engine, BASE64_STANDARD};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgConnection, PgTextExpressionMethods, QueryDsl,
    RunQueryDsl, SelectableHelper,
};

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
        owner_id: uuid::Uuid,
        sample_id: uuid::Uuid,
    ) -> messages::samples::SampleImageResult {
        use crate::schema::samples;

        let mut connection = self.pool.get().expect("Unable to connect to database");

        match samples::table
            .filter(
                samples::id
                    .eq(sample_id)
                    .and(samples::owner_id.eq(owner_id)),
            )
            .select(samples::bytes)
            .first::<Vec<u8>>(&mut connection)
        {
            Ok(bytes) => messages::samples::SampleImageResult::Success { bytes },
            Err(diesel::result::Error::NotFound) => messages::samples::SampleImageResult::NotFound,
            Err(_) => messages::samples::SampleImageResult::ServerError,
        }
    }

    #[inline]
    pub(crate) async fn delete_sample_image(
        &self,

        sample_id: uuid::Uuid,
    ) -> messages::samples::SampleInferResult {
        use crate::schema::samples;

        let mut connection = self.pool.get().expect("Unable to connect to database");

        match diesel::update(samples::table.filter(samples::id.eq(sample_id)))
            .set(samples::deleted.eq(true))
            .execute(&mut connection)
        {
            Ok(_) => messages::samples::SampleInferResult::Success,
            Err(diesel::result::Error::NotFound) => messages::samples::SampleInferResult::NotFound,
            Err(_) => messages::samples::SampleInferResult::ServerError,
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
                samples::deleted.eq(false).and(
                    samples::id
                        .eq(sample_id)
                        .and(samples::owner_id.eq(owner_id)),
                ),
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

        if boxes.is_empty() {
            return messages::samples::SampleInferResult::Reject;
        }

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
        use crate::schema::{results, samples};

        let mut connection = self.pool.get().expect("Unable to connect to database");

        match samples::table
            .left_outer_join(results::table)
            .filter(
                samples::deleted
                    .eq(false)
                    .and(samples::owner_id.eq(user_id).and(samples::label.ilike(
                        if let Some(search) = desc.keyword {
                            format!("%{}%", search)
                        } else {
                            "%".to_string()
                        },
                    ))),
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

    #[inline]
    pub(crate) async fn get_inferred_list(
        &self,
        user_id: uuid::Uuid,
        desc: messages::samples::SampleInferredList,
    ) -> messages::samples::InferredListResult {
        use crate::schema::{results, samples};

        let mut connection = self.pool.get().expect("Unable to connect to database");

        match results::table
            .inner_join(samples::table)
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
            .select((
                self::samples::Result::as_select(),
                self::samples::Sample::as_select(),
            ))
            .limit(10)
            .offset(desc.page as i64)
            .order(samples::created_at.desc())
            .get_results::<(self::samples::Result, self::samples::Sample)>(&mut connection)
        {
            Ok(r) => {
                let list: Vec<(self::samples::Result, self::samples::Sample)> = r;

                let map = list.into_iter().fold(
                    HashMap::<uuid::Uuid, messages::samples::InferredListEntry>::new(),
                    |mut buffer, (result, sample)| {
                        let result_entry = messages::samples::InferredResultListEntry {
                            id: result.id,
                            certainty: result.certainty,
                            is_normal: result.is_normal,
                            x: result.x,
                            y: result.y,
                            width: result.width,
                            height: result.height,
                            iris_x: result.iris_x,
                            iris_y: result.iris_y,
                            iris_a: result.iris_a,
                            iris_b: result.iris_b,
                            coverage: result.coverage,
                            created_at: result.created_at,
                            updated_at: result.updated_at,
                        };

                        if let Some(entry) = buffer.get_mut(&result.sample_id) {
                            entry.results.push(result_entry);

                            buffer
                        } else {
                            buffer.insert(
                                result.sample_id,
                                messages::samples::InferredListEntry {
                                    id: sample.id,
                                    label: sample.label,
                                    pet_id: sample.pet_id,
                                    created_at: sample.created_at,
                                    updated_at: sample.updated_at,
                                    results: vec![result_entry],
                                },
                            );

                            buffer
                        }
                    },
                );

                let mut items: Vec<messages::samples::InferredListEntry> =
                    map.into_values().collect();

                items.sort_by(|a, b| b.created_at.cmp(&a.created_at));

                messages::samples::InferredListResult::Success {
                    items,
                    has_next: false,
                }
            }
            _ => messages::samples::InferredListResult::Failed,
        }
    }
}
