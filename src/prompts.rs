use askama::Template;

#[derive(Template)]
#[template(path = "system_prompt.md")]
pub struct SystemPrompt {
    time: jiff::Zoned,
}

impl Default for SystemPrompt {
    fn default() -> Self {
        Self {
            time: jiff::Zoned::now()
                .in_tz("UTC")
                .expect("should always be able to construct a timezone in UTC"),
        }
    }
}
