-- Your SQL goes here
CREATE TABLE `projects`(
	`id` BINARY NOT NULL PRIMARY KEY,
	`data` BINARY NOT NULL
);

CREATE TABLE `artifacts`(
	`project_id` BINARY NOT NULL,
	`run_id` BINARY,
	`artifact_id` BINARY NOT NULL,
	`version_id` BINARY NOT NULL,
	`artifact_type` TEXT NOT NULL,
	`version_data` BINARY NOT NULL,
	`client_creation_time` TEXT NOT NULL,
	`path` TEXT NOT NULL,
	`series_id` BINARY,
	`series_value` TEXT,
	`series_point` BINARY,
	PRIMARY KEY(`project_id`, `artifact_id`, `version_id`)
);

CREATE TABLE `payloads`(
	`project_id` BINARY NOT NULL,
	`artifact_id` BINARY NOT NULL,
	`version_id` BINARY NOT NULL,
	`payload` BINARY NOT NULL,
	PRIMARY KEY(`project_id`, `artifact_id`, `version_id`)
);

