use crate::crypt::EncryptContent;
use crate::crypt::pwd;
use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::model::Result;
use crate::model::base::{self, DbBmc};
use crate::web::rpc::ParamsForCreate;
use serde::{Deserialize, Serialize};
use sqlb::{Fields, HasFields};
use sqlx::FromRow;
use sqlx::Row;
use sqlx::postgres::PgRow;
use uuid::Uuid;

#[derive(Clone, Fields, FromRow, Debug, Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct UsernameOnly {
    pub username: String,
}

#[derive(Deserialize, Fields)]
pub struct UserForCreate {
    pub username: String,
    pub pwd_clear: String,
}

#[derive(Deserialize, Fields)]
pub struct Friendship {
    pub user1_id: i64,
    pub user2_id: i64,
}

#[derive(Fields)]
struct UserForInsert {
    username: String,
    pwd: String,
    pwd_salt: Uuid,
    token_salt: Uuid,
}

#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForLogin {
    pub id: i64,
    pub username: String,

    // pwd and token info
    pub pwd: Option<String>, //encrypted with #_scheme_id_#
    pub pwd_salt: Uuid,
    pub token_salt: Uuid,
}

#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForAuth {
    pub id: i64,
    pub username: String,

    // token info
    pub token_salt: Uuid,
}

#[derive(FromRow, Fields, Deserialize)]
pub struct FriendForCreate {
    pub id: i64,
}

pub trait UserBy: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl UserBy for User {}
impl UserBy for UserForLogin {}
impl UserBy for UserForAuth {}
impl UserBy for FriendForCreate {}

pub struct UserBmc;

impl DbBmc for UserBmc {
    const TABLE: &'static str = "users";
}

impl UserBmc {
    pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
    where
        E: UserBy,
    {
        base::get::<Self, _>(ctx, mm, id).await
    }

    pub async fn find_username_by_id(
        _ctx: Ctx,
        mm: ModelManager,
        id: i64,
    ) -> Result<Option<UsernameOnly>> {
        let db = mm.db();

        let username = sqlb::select()
            .table(Self::TABLE)
            .columns(&["username"])
            .and_where("id", "=", id)
            .fetch_optional::<_, UsernameOnly>(db)
            .await?;

        Ok(username)
    }

    pub async fn first_by_username<E>(
        _ctx: &Ctx,
        mm: &ModelManager,
        username: &str,
    ) -> Result<Option<E>>
    where
        E: UserBy,
    {
        let db = mm.db();

        let user = sqlb::select()
            .table(Self::TABLE)
            .and_where("username", "=", username)
            .fetch_optional::<_, E>(db)
            .await?;

        Ok(user)
    }

    pub async fn create_user(mm: &ModelManager, username: &str, pwd_clear: &str) -> Result<()> {
        let db = mm.db();

        let pwd_salt = Uuid::new_v4();
        let pwd = pwd::encrypt_pwd(&EncryptContent {
            content: pwd_clear.to_string(),
            salt: pwd_salt.to_string(),
        })?;

        let user = UserForInsert {
            username: username.to_string(),
            pwd: pwd,
            pwd_salt: pwd_salt,
            token_salt: Uuid::new_v4(),
        };
        sqlb::insert()
            .table(Self::TABLE)
            .data(user.all_fields())
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn add_friend(
        ctx: Ctx,
        mm: ModelManager,
        params: ParamsForCreate<FriendForCreate>,
    ) -> Result<()> {
        let ParamsForCreate { data } = params;
        let db = mm.db();

        let user: User = Self::get(&ctx, &mm, ctx.user_id()).await?;

        let friendship = Friendship {
            user1_id: user.id,
            user2_id: data.id,
        };

        sqlb::insert()
            .table("friends")
            .data(friendship.all_fields())
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn get_friends(ctx: Ctx, mm: ModelManager) -> Result<Vec<String>> {
        let db = mm.db();

        let user: User = Self::get(&ctx, &mm, ctx.user_id()).await?;

        let rows = sqlx::query("SELECT DISTINCT user2_id FROM friends WHRE user1_id= $1")
            .bind(&user.id)
            .fetch_all(db)
            .await?;

        let mut usernames = Vec::new();
        for row in rows {
            if let Ok(username) = row.try_get::<String, _>("user2_id") {
                let _rows = sqlx::query("SELECT user2_id FROM users")
                    .bind(&user.username)
                    .fetch_all(db)
                    .await?;
                usernames.push(username);
            }
        }

        Ok(usernames)
    }

    pub async fn update_pwd(ctx: &Ctx, mm: &ModelManager, id: i64, pwd_clear: &str) -> Result<()> {
        let db = mm.db();
        let user: UserForLogin = Self::get(ctx, mm, id).await?;

        let pwd = pwd::encrypt_pwd(&EncryptContent {
            content: pwd_clear.to_string(),
            salt: user.pwd_salt.to_string(),
        })?;

        sqlb::update()
            .table(Self::TABLE)
            .and_where("id", "=", id)
            .data(vec![("pwd", pwd.to_string()).into()])
            .exec(db)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::_dev_utils;
    use anyhow::{Context, Result};
    use serial_test::serial;

    #[serial]
    #[tokio::main]
    async fn test_first_ok_demo1() -> Result<()> {
        // Setup
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_username = "demo1";

        // Execute
        let user: User = UserBmc::first_by_username(&ctx, &mm, fx_username)
            .await?
            .context("Should have user 'demo1'")?;

        // Check
        assert_eq!(user.username, fx_username);

        Ok(())
    }
}
