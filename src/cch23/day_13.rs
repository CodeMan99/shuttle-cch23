use std::fmt::{self, Display};

use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, post, State};
use sqlx::{FromRow, PgPool, QueryBuilder};

macro_rules! server_err {
    ($err:expr) => {
        (Status::InternalServerError, $err.to_string())
    };
}

#[repr(transparent)]
pub struct GiftDatabase {
    pool: PgPool,
}

#[derive(FromRow)]
struct Example(i32);

impl Display for Example {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[get("/sql")]
async fn sql(gift_db: &State<GiftDatabase>) -> Result<String, (Status, String)> {
    let example: Example = sqlx::query_as("SELECT 20231213")
        .fetch_one(&gift_db.pool)
        .await
        .map_err(|err| server_err!(err))?;
    Ok(example.to_string())
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
struct Order {
    id: i64,
    region_id: i64,
    gift_name: String,
    quantity: i64,
}

#[post("/reset")]
async fn reset(gift_db: &State<GiftDatabase>) -> Result<(), (Status, String)> {
    let mut transaction = (gift_db.pool.begin())
        .await
        .map_err(|err| server_err!(err))?;
    let _result = sqlx::query("DROP TABLE IF EXISTS orders")
        .execute(&mut *transaction)
        .await
        .map_err(|err| server_err!(err))?;
    let _result = sqlx::query(
        r#"CREATE TABLE orders (
            id INT PRIMARY KEY,
            region_id INT,
            gift_name VARCHAR(50),
            quantity INT
        )"#,
    )
    .execute(&mut *transaction)
    .await
    .map_err(|err| server_err!(err))?;

    transaction.commit().await.map_err(|err| server_err!(err))?;

    Ok(())
}

#[post("/orders", data = "<orders>")]
async fn orders(
    orders: Json<Vec<Order>>,
    gift_db: &State<GiftDatabase>,
) -> Result<(), (Status, String)> {
    let _result = QueryBuilder::new("INSERT INTO orders (id, region_id, gift_name, quantity) ")
        .push_values(orders.iter(), |mut binder, order| {
            binder
                .push_bind(order.id)
                .push_bind(order.region_id)
                .push_bind(order.gift_name.clone())
                .push_bind(order.quantity);
        })
        .build()
        .execute(&gift_db.pool)
        .await
        .map_err(|err| server_err!(err))?;
    Ok(())
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
struct OrderTotal {
    total: i64,
}

#[get("/orders/total")]
async fn total(gift_db: &State<GiftDatabase>) -> Result<Json<OrderTotal>, (Status, String)> {
    let order_total: OrderTotal = sqlx::query_as("SELECT SUM(quantity) AS total FROM orders")
        .fetch_one(&gift_db.pool)
        .await
        .map_err(|err| server_err!(err))?;

    Ok(Json(order_total))
}

#[derive(Debug, Default, FromRow, Deserialize, Serialize)]
struct PopularGift {
    popular: Option<String>,
}

#[get("/orders/popular")]
async fn popular(gift_db: &State<GiftDatabase>) -> Result<Json<PopularGift>, (Status, String)> {
    let popular_gift: Option<PopularGift> = sqlx::query_as(
        r#"
            SELECT gift_name AS popular
            FROM orders
            GROUP BY gift_name
            ORDER BY SUM(quantity) DESC
            LIMIT 1
        "#,
    )
    .fetch_optional(&gift_db.pool)
    .await
    .map_err(|err| server_err!(err))?;

    Ok(Json(popular_gift.unwrap_or_default()))
}

pub fn create_gift_db(pool: PgPool) -> GiftDatabase {
    GiftDatabase { pool }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![sql, reset, orders, total, popular]
}
