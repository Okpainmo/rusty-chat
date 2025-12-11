use sqlx::{PgPool, FromRow};
use serde_json::Value;
use std::collections::HashMap;

pub async fn update_user_fields<T>(
    db_pool: &PgPool,
    user_id: i64,
    fields: HashMap<String, Value>,
) -> Result<Option<T>, sqlx::Error>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    if fields.is_empty() {
        return Ok(None);
    }

    // Build SET field1 = $2, field2 = $3...
    let set_fragments: Vec<String> = fields
        .keys()
        .enumerate()
        .map(|(idx, key)| format!("{} = ${}", key, idx + 2))
        .collect();

    let query = format!(
        "UPDATE users SET {} WHERE id = $1
         RETURNING id, full_name, email, profile_image_url, password,
                   access_token, refresh_token, status, last_seen",
        set_fragments.join(", "),
    );

    let mut builder = sqlx::query_as::<_, T>(&query).bind(user_id);

    for value in fields.values() {
        builder = builder.bind(value);
    }

    builder.fetch_optional(db_pool).await
}