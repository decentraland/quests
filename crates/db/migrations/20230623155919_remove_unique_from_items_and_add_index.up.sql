-- remove uniqueness --
ALTER TABLE quest_reward_items DROP CONSTRAINT quest_reward_items_quest_id_key;

CREATE INDEX quest_reward_items_quest_id_idx ON quest_reward_items(quest_id);
