create table user_profile (
	user_id uuid PRIMARY KEY,
	email TEXT NOT NULL UNIQUE
);
