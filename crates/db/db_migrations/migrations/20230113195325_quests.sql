-- Create quests table
CREATE TABLE IF NOT EXISTS quests (
  ID SERIAL PRIMARY KEY NOT NULL, -- TODO: Should be a UUID?
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  definition bytea NOT NULL
  -- TODO: Add timestamps? (created_at, updated_at)
);

-- Create quest_instances table
CREATE TABLE IF NOT EXISTS quest_instances (
  ID SERIAL PRIMARY KEY NOT NULL, -- TODO: Should be a UUID?
  quest_id INT references quests(ID) ON DELETE CASCADE,
  user_address TEXT NOT NULL,
  started timestamptz NOT NULL DEFAULT now()
);

-- Create events table
CREATE TABLE IF NOT EXISTS events ( 
  ID SERIAL PRIMARY KEY NOT NULL, -- TODO: Should be a UUID?
  quest_instance_id INT references quests(ID) ON DELETE CASCADE, -- TODO: should not be a FK?
  user_address TEXT NOT NULL,
  event bytea NOT NULL,
  timestamp timestamptz NOT NULL DEFAULT now()
);

