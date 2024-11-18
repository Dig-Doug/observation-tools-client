-- Your SQL goes here



CREATE TABLE `permissions`(
	`principal_id` TEXT NOT NULL,
	`resource_type` INTEGER NOT NULL,
	`relation` INTEGER NOT NULL,
	`project_id` BINARY NOT NULL,
	`artifact_id` BINARY,
	PRIMARY KEY(`principal_id`, `resource_type`, `relation`, `project_id`, `artifact_id`)
);

