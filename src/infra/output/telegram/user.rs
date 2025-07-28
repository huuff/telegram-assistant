use crate::domain::user::User;

impl TryFrom<teloxide::types::User> for User {
    type Error = anyhow::Error;

    fn try_from(telegram_user: teloxide::types::User) -> Result<Self, Self::Error> {
        Ok(User {
            id: telegram_user.id.0.to_string(),
            name: telegram_user.full_name(),
            preferred_language: telegram_user
                .language_code
                .and_then(|lang| isolang::Language::from_639_1(&lang))
                .map(|lang| lang.to_name().to_owned()),
        })
    }
}
