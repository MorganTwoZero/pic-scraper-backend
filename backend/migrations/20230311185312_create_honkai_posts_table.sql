CREATE TYPE post_source AS ENUM (
    'twitter',
    'mihoyo',
    'pixiv',
    'bcy',
    'lofter',
    'twitterhome'
);
CREATE TABLE honkai_posts(
    post_link TEXT NOT NULL,
    preview_link TEXT NOT NULL,
    images_number INT NOT NULL,
    created TEXT NOT NULL,
    author TEXT NOT NULL,
    author_link TEXT NOT NULL,
    author_profile_image TEXT NULL,
    source post_source NOT NULL,
    tags TEXT[] NULL,
    PRIMARY KEY (post_link)
);