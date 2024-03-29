use chrono::NaiveDateTime;
use diesel::{associations::Associations, Identifiable, Insertable, Queryable, Selectable};

#[derive(Clone, Debug, PartialEq, Eq, Insertable)]
#[diesel(table_name = crate::schema::samples)]
pub(crate) struct SampleInsert {
    pub(crate) label: String,
    pub(crate) bytes: Vec<u8>,
    pub(crate) owner_id: uuid::Uuid,
    pub(crate) deleted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Selectable, Queryable)]
#[diesel(table_name = crate::schema::samples)]
pub(crate) struct SampleEntry {
    pub(crate) id: uuid::Uuid,
    pub(crate) label: String,
    pub(crate) pet_id: Option<uuid::Uuid>,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = crate::schema::results)]
pub(crate) struct ResultInsert {
    pub(crate) sample_id: uuid::Uuid,
    pub(crate) certainty: f32,
    pub(crate) is_normal: bool,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

use crate::schema::{results, samples};

#[derive(Queryable, Identifiable, Selectable)]
#[table_name = "samples"]
pub struct Sample {
    pub id: uuid::Uuid,
    pub label: String,
    pub pet_id: Option<uuid::Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Identifiable, Associations, Selectable)]
#[table_name = "results"]
#[belongs_to(Sample)]
pub struct Result {
    pub id: uuid::Uuid,
    pub certainty: f32,
    pub is_normal: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub iris_x: Option<f32>,
    pub iris_y: Option<f32>,
    pub iris_a: Option<f32>,
    pub iris_b: Option<f32>,
    pub coverage: Option<f32>,
    pub sample_id: uuid::Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}
