use sqlx::PgPool;

pub mod day_01;
pub mod day_04;
pub mod day_05;
pub mod day_06;
pub mod day_07;
pub mod day_08;
pub mod day_11;
pub mod day_12;
pub mod day_13;
pub mod day_14;
pub mod day_15;
pub mod day_18;
pub mod day_19;
pub mod day_20;
pub mod day_21;
pub mod day_22;

#[repr(transparent)]
pub struct GiftDatabase {
    pool: PgPool,
}

pub fn create_gift_db(pool: PgPool) -> GiftDatabase {
    GiftDatabase { pool }
}
