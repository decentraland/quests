-- Add down migration script here
ALTER TABLE quest_reward_items ADD CONSTRAINT quest_reward_items UNIQUE (quest_id);

DROP INDEX quest_reward_items_quest_id_idx;