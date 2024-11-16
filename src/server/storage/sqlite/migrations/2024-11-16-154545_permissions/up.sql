-- Your SQL goes here



CREATE TABLE `permissions`(
	`principal_id` TEXT NOT NULL,
	`resource_id` TEXT NOT NULL PRIMARY KEY,
	`relation` TEXT NOT NULL
);

