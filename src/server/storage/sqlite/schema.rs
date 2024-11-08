// @generated automatically by Diesel CLI.

diesel::table! {
    artifacts (project_id, artifact_id, version_id) {
        project_id -> Binary,
        run_id -> Nullable<Binary>,
        artifact_id -> Binary,
        version_id -> Binary,
        artifact_type -> Text,
        version_data -> Binary,
        client_creation_time -> Text,
        path -> Text,
        series_id -> Nullable<Binary>,
        series_value -> Nullable<Text>,
        series_point -> Nullable<Binary>,
    }
}

diesel::table! {
    payloads (project_id, artifact_id, version_id) {
        project_id -> Binary,
        artifact_id -> Binary,
        version_id -> Binary,
        payload -> Binary,
    }
}

diesel::table! {
    projects (id) {
        id -> Binary,
        data -> Binary,
    }
}

diesel::allow_tables_to_appear_in_same_query!(artifacts, payloads,);
