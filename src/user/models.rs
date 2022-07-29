use diesel::prelude::*;
use diesel::result::Error;

use crate::schema::users;
use crate::user::UserId;

#[derive(Insertable, Queryable, Identifiable)]
#[table_name = "users"]
pub struct User {
    pub id: UserId,
    pub password_hash: String,
}

pub fn get_user(db_conn: &PgConnection, user_id: &str) -> Result<User, Error> {
    use crate::schema::users::dsl::users;
    let result = users.find(user_id).first(db_conn);
    result
}

pub fn create_user(db_conn: &PgConnection, user: User) -> Result<(), Error> {
    diesel::insert_into(users::table)
        .values(user)
        .execute(db_conn)?;
    Ok(())
}

pub fn update_password_hash(
    db_con: &PgConnection,
    user_id: &str,
    password_hash: &str,
) -> Result<(), Error> {
    // using get_result() so that we get NotFound if the user doesn't exist
    let _: User = diesel::update(users::table.find(user_id))
        .set(users::password_hash.eq(password_hash))
        .get_result(db_con)?;
    Ok(())
}

pub fn delete_user(db_conn: &PgConnection, user_id: &str) -> Result<(), Error> {
    // using get_result() so that we get NotFound if the user doesn't exist
    let _: User = diesel::delete(users::table.find(user_id)).get_result(db_conn)?;
    Ok(())
}
