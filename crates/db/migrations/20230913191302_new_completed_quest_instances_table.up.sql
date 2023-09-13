CREATE TABLE IF NOT EXISTS completed_quest_instances (
  ID UUID PRIMARY KEY NOT NULL,
  quest_instance_id UUID references quest_instances(ID), 
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  UNIQUE (quest_instance_id)
);