create table users(
	user_id uuid PRIMARY KEY,
	email TEXT NOT NULL UNIQUE
);
