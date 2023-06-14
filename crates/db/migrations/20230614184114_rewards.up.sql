CREATE TABLE IF NOT EXISTS quest_reward_hooks (
  quest_id UUID references quests(ID), 
  webhook_url TEXT NOT NULL,
  request_body JSON NULL, 
  UNIQUE(quest_id)
);

CREATE TABLE IF NOT EXISTS quest_reward_items (
  quest_id UUID references quests(ID), 
  reward_name TEXT NOT NULL,
  reward_image TEXT NOT NULL,
  UNIQUE(quest_id)
);