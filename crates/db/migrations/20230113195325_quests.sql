CREATE TABLE IF NOT EXISTS quests (
  ID UUID PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  definition bytea NOT NULL,
  creator_address TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS quest_instances (
  ID UUID PRIMARY KEY NOT NULL,
  quest_id UUID references quests(ID), 
  user_address TEXT NOT NULL,
  start_timestamp TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS events ( 
  ID UUID PRIMARY KEY NOT NULL,
  quest_instance_id UUID references quest_instances(ID),
  user_address TEXT NOT NULL,
  event bytea NOT NULL,
  timestamp TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS abandoned_quests (
  ID UUID PRIMARY KEY NOT NULL,
  quest_instance_id UUID references quest_instances(ID), 
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  UNIQUE (quest_instance_id)
);

CREATE TABLE IF NOT EXISTS deactivated_quests (
  ID UUID PRIMARY KEY NOT NULL,
  quest_id UUID references quests(ID), 
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  UNIQUE (quest_id)
);

CREATE TABLE IF NOT EXISTS quest_updates (
  ID UUID PRIMARY KEY NOT NULL,
  quest_id UUID references quests(ID), 
  previous_quest_id UUID references quests(ID),
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  UNIQUE (quest_id)
);
