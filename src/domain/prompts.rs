use askama::Template;
use bon::Builder;

use super::user::User;

#[derive(Template, Builder)]
#[template(path = "system_prompt.md")]
pub struct SystemPrompt {
    #[builder(default = jiff::Zoned::now().in_tz("UTC").expect("should always be possible to construct a time in UTC"))]
    time: jiff::Zoned,
    user: User,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_snapshot() {
        // ARRANGE
        let prompt = SystemPrompt::builder()
            .time("2025-07-28T00:00:00[UTC]".parse().unwrap())
            .user(User {
                id: "1".into(),
                name: "User".into(),
                preferred_language: Some("Klingon".into()),
            })
            .build();

        // ACT
        let text = prompt.render().unwrap();

        // ASSERT
        insta::assert_snapshot!(text);
    }
}
