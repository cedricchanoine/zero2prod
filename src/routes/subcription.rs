use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use tracing::Instrument;

#[tracing::instrument(
    name = "addind a new subscriber",
    skip(data, _connection),
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_email = %data.email,
        subscriber_name = %data.name
        )
    )]
pub async fn subscription(data: web::Form<SubscriptionInfos>,
                          _connection: web::Data<PgPool>
                          ) -> HttpResponse {
    let query_span = tracing::info_span!("saving a new subscriber in the database");

        match insert_subscriber(&data, &_connection).await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(_) => HttpResponse::InternalServerError().finish()
        }
}

#[derive(serde::Deserialize)]
pub struct SubscriptionInfos{
    email: String,
    name: String,
}
pub async fn insert_subscriber(data: &SubscriptionInfos, _connection: &PgPool) -> Result<(), sqlx::Error>{

    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        data.email,
        data.name,
        Utc::now()
        ).execute(_connection)
        .await
        .map_err(|e| {
            tracing::error!("failed to execute query {:?}", e);
            e
        })?;

        Ok(())
}
