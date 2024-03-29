use diesel::{Insertable, Queryable, Selectable};

#[derive(Clone, Debug, PartialEq, Eq, Insertable)]
#[diesel(table_name = crate::schema::samples)]
pub(crate) struct SampleInsert {
    pub(crate) label: String,
    pub(crate) bytes: Vec<u8>,
    pub(crate) owner_id: uuid::Uuid,
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
