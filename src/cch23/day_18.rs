use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, post, State};
use sqlx::{FromRow, QueryBuilder};

use crate::cch23::GiftDatabase;

#[derive(Debug, FromRow, Serialize, Deserialize)]
struct Order {
    id: i64,
    region_id: i64,
    gift_name: String,
    quantity: i64,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
struct Region {
    id: i64,
    name: String,
}

macro_rules! server_err {
    ($err:expr) => {
        (Status::InternalServerError, $err.to_string())
    };
}

#[post("/reset")]
async fn reset(gift_db: &State<GiftDatabase>) -> Result<(), (Status, String)> {
    let mut transaction = (gift_db.pool.begin())
        .await
        .map_err(|err| server_err!(err))?;
    let _result = sqlx::query("DROP TABLE IF EXISTS regions")
        .execute(&mut *transaction)
        .await
        .map_err(|err| server_err!(err))?;
    let _result = sqlx::query("DROP TABLE IF EXISTS orders")
        .execute(&mut *transaction)
        .await
        .map_err(|err| server_err!(err))?;
    let _result = sqlx::query(
        r#"CREATE TABLE regions (
            id INT PRIMARY KEY,
            name VARCHAR(50)
        )"#,
    )
    .execute(&mut *transaction)
    .await
    .map_err(|err| server_err!(err));
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
    if orders.len() == 0 {
        return Ok(());
    }

    let _result = QueryBuilder::new("INSERT INTO orders (id, region_id, gift_name, quantity) ")
        .push_values(orders.iter(), |mut binder, order| {
            binder
                .push_bind(order.id)
                .push_bind(order.region_id)
                .push_bind(order.gift_name.as_str())
                .push_bind(order.quantity);
        })
        .build()
        .execute(&gift_db.pool)
        .await
        .map_err(|err| server_err!(err))?;
    Ok(())
}

#[post("/regions", data = "<regions>")]
async fn regions(
    regions: Json<Vec<Region>>,
    gift_db: &State<GiftDatabase>,
) -> Result<(), (Status, String)> {
    if regions.len() == 0 {
        return Ok(());
    }

    let _result = QueryBuilder::new("INSERT INTO regions (id, name) ")
        .push_values(regions.iter(), |mut binder, region| {
            binder.push_bind(region.id).push_bind(region.name.as_str());
        })
        .build()
        .execute(&gift_db.pool)
        .await
        .map_err(|err| server_err!(err))?;
    Ok(())
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
struct RegionTotal {
    region: String,
    total: i64,
}

#[get("/regions/total")]
async fn total(gift_db: &State<GiftDatabase>) -> Result<Json<Vec<RegionTotal>>, (Status, String)> {
    let total: Vec<RegionTotal> = sqlx::query_as(
        r#"SELECT r.name AS region, SUM(o.quantity) AS total
        FROM regions r
        JOIN orders o ON (o.region_id = r.id)
        GROUP BY r.name
        ORDER BY r.name ASC
        "#,
    )
    .fetch_all(&gift_db.pool)
    .await
    .map_err(|err| server_err!(err))?;

    Ok(Json(total))
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
struct TopGift {
    region: String,
    top_gifts: Vec<String>,
}

#[get("/regions/top_list/<count>")]
async fn top_list(
    count: i64,
    gift_db: &State<GiftDatabase>,
) -> Result<Json<Vec<TopGift>>, (Status, String)> {
    let gifts: Vec<TopGift> = sqlx::query_as(
        r#"SELECT r.name AS region, array_remove(array_agg(oo.gift_name), NULL) AS top_gifts
        FROM regions r
        LEFT JOIN LATERAL (
            SELECT *
            FROM orders o
            WHERE o.region_id = r.id
            ORDER BY o.quantity DESC, o.gift_name ASC
            LIMIT 2
        ) oo ON TRUE
        GROUP BY r.name
        "#,
    )
    .bind(count)
    .fetch_all(&gift_db.pool)
    .await
    .map_err(|err| server_err!(err))?;

    Ok(Json(gifts))
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![reset, orders, regions, total, top_list]
}
