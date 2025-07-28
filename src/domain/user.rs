#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    /// The user's preferred language in english
    pub preferred_language: Option<String>,
}
