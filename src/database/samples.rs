use diesel::Insertable;

#[derive(Clone, Debug, PartialEq, Eq, Insertable)]
#[diesel(table_name = crate::schema::samples)]
pub(crate) struct SampleInsert {
    pub(crate) label: String,
    pub(crate) bytes: Vec<u8>,
    pub(crate) owner_id: uuid::Uuid,
}
