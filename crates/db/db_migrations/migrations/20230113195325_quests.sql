-- Create quests table
CREATE TABLE IF NOT EXISTS quests (
  ID SERIAL PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  definition bytea NOT NULL
);

-- Create quest_instances table
CREATE TABLE IF NOT EXISTS quest_instances (
  ID SERIAL PRIMARY KEY NOT NULL,
  quest_id INT references quests(ID) ON DELETE CASCADE,
  user_address TEXT NOT NULL,
  started timestamptz NOT NULL DEFAULT now()
);

-- Create events table
CREATE TABLE IF NOT EXISTS events ( 
  ID SERIAL PRIMARY KEY NOT NULL,
  user_address TEXT NOT NULL,
  event bytea NOT NULL,
  timestamp timestamptz NOT NULL DEFAULT now()
);

