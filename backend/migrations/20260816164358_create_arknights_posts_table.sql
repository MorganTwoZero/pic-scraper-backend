-- Add migration script here
CREATE TABLE arknights_posts(
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    post_link TEXT NOT NULL UNIQUE,
    preview_link TEXT NOT NULL,
    images_number INT NOT NULL,
    created TEXT NOT NULL,
    author TEXT NOT NULL,
    author_link TEXT NOT NULL,
    author_profile_image TEXT NULL,
    source post_source NOT NULL,
    tags TEXT[] NULL
);
