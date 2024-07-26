-- Add migration script here
-- Step 1: Add the new column (not as SERIAL initially)
ALTER TABLE honkai_posts ADD COLUMN id INTEGER;

-- Step 2: Populate it with a sequence for existing rows
CREATE SEQUENCE honkai_posts_id_seq;

UPDATE honkai_posts
SET id = nextval('honkai_posts_id_seq');

-- Step 3: Set the column to use the sequence and make it NOT NULL
ALTER TABLE honkai_posts ALTER COLUMN id SET NOT NULL;
ALTER TABLE honkai_posts ALTER COLUMN id SET DEFAULT nextval('honkai_posts_id_seq');
ALTER SEQUENCE honkai_posts_id_seq OWNED BY honkai_posts.id;
