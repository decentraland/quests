-- Create quests table
CREATE TABLE IF NOT EXISTS quests (
  ID UUID PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  definition bytea NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
  -- TODO: should we add the creator id (address)?
);

-- Create quest_instances table
CREATE TABLE IF NOT EXISTS quest_instances (
  ID UUID PRIMARY KEY NOT NULL,
  quest_id UUID references quests(ID) ON DELETE CASCADE,  -- TODO: should not be a FK?
  user_address TEXT NOT NULL,
  start_timestamp TIMESTAMP NOT NULL DEFAULT now()
);

-- Create events table
CREATE TABLE IF NOT EXISTS events ( 
  ID UUID PRIMARY KEY NOT NULL,
  quest_instance_id UUID references quest_instances(ID) ON DELETE CASCADE, -- TODO: should not be a FK?
  user_address TEXT NOT NULL,
  event bytea NOT NULL,
  timestamp TIMESTAMP NOT NULL DEFAULT now()
);

