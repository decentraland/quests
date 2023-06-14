const USER_ADDRESS: &str = "{user_address}";
const QUEST_ID: &str = "{quest_id}";

/// Parse webhook url and request body to replace {user_address} and {quest_id} with the actual values
/// to give rewards to an user when a quest is completed
pub fn rewards_parser(to_be_parsed: &str, quest_id: &str, user_address: &str) -> String {
    let mut parsed = to_be_parsed.to_string();
    parsed = parsed.replace(USER_ADDRESS, user_address);
    parsed = parsed.replace(QUEST_ID, quest_id);
    parsed
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_parse() {
        let url = "http://localhost:8080/rewards/{quest_id}/{user_address}";
        let quest_id = "123";
        let user_address = "0xB";

        let parsed = super::rewards_parser(url, quest_id, user_address);

        assert_eq!(parsed, "http://localhost:8080/rewards/123/0xB");
    }

    #[test]
    fn should_not_change() {
        let url = "http://localhost:8080/rewards/quest_id";
        let quest_id = "123";
        let user_address = "0xB";

        let parsed = super::rewards_parser(url, quest_id, user_address);

        assert_eq!(parsed, "http://localhost:8080/rewards/quest_id");
    }
}
