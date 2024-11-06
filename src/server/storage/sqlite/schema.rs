// @generated automatically by Diesel CLI.

diesel::table! {
    artifacts (project_id, artifact_id, version_id) {
        project_id -> Binary,
        run_id -> Nullable<Binary>,
        artifact_id -> Binary,
        version_id -> Binary,
        artifact_type -> Integer,
        proto_data -> Binary,
        client_creation_time -> Text,
        path -> Text,
        series_id -> Nullable<Binary>,
        series_value -> Nullable<Text>,
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
