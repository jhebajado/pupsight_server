// @generated automatically by Diesel CLI.

diesel::table! {
    pets (id) {
        id -> Uuid,
        #[max_length = 32]
        name -> Varchar,
        birthday -> Nullable<Timestamp>,
        owner_id -> Uuid,
        exact_birthday -> Bool,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    results (id) {
        id -> Uuid,
        certainty -> Float4,
        is_normal -> Bool,
        x -> Float4,
        y -> Float4,
        width -> Float4,
        height -> Float4,
        iris_x -> Nullable<Float4>,
        iris_y -> Nullable<Float4>,
        iris_a -> Nullable<Float4>,
        iris_b -> Nullable<Float4>,
        coverage -> Nullable<Float4>,
        sample_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    samples (id) {
        id -> Uuid,
        #[max_length = 32]
        label -> Varchar,
        bytes -> Bytea,
        owner_id -> Uuid,
        pet_id -> Nullable<Uuid>,
        deleted -> Nullable<Bool>,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    session (id) {
        id -> Uuid,
        access_token -> Bytea,
        user_id -> Uuid,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 24]
        login_name -> Varchar,
        #[max_length = 48]
        first_name -> Varchar,
        #[max_length = 48]
        last_name -> Varchar,
        argon2 -> Bytea,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(pets -> users (owner_id));
diesel::joinable!(results -> samples (sample_id));
diesel::joinable!(samples -> pets (pet_id));
diesel::joinable!(samples -> users (owner_id));
diesel::joinable!(session -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    pets,
    results,
    samples,
    session,
    users,
);
