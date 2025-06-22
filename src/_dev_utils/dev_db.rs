use crate::Ctx;
use crate::ModelManager;
use crate::model::user::{User, UserBmc};
use log::info;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::{fs, path::PathBuf, time::Duration};

type Db = Pool<Postgres>;

// Hardcoded to prevent accidentally updating a real production db;

const PG_DEV_POSTGRES_URL: &str = "postgres://postgres:welcome@localhost/postgres";
const PG_DEV_APP_URL: &str = "postgres://app_user:dev_only_pwd@localhost/app_db";

// sql files

const SQL_RECREATE_DB: &str = "sql/dev_initial/00-recreate-db.sql";
const SQL_DIR: &str = "sql/dev_initial";

const DEMO_PWD: &str = "hello";

pub async fn init_dev_db() -> Result<(), Box<dyn std::error::Error>> {
    info!("{:<12} - init_dev_db()", "FOR-DEV-ONLY");

    // -- Create app_db/app_user with the postgres user
    {
        let root_db = new_db_pool(PG_DEV_POSTGRES_URL).await?;
        pexec(&root_db, SQL_RECREATE_DB).await?;
        //drop(root_db); // root_db only in scope during parenthesis, can remove and uncomment here to
        //do the same thing
    }
    // -- Get sql files
    let mut paths: Vec<PathBuf> = fs::read_dir(SQL_DIR)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    paths.sort();

    // -- SQL Execute files
    let app_db = new_db_pool(PG_DEV_APP_URL).await?;
    for path in paths {
        if let Some(path) = path.to_str() {
            let path = path.replace('\\', "/"); // for Windows systems

            // Only use .sql and skip SQL_RECREATE_DB
            if path.ends_with(".sql") && path != SQL_RECREATE_DB {
                pexec(&app_db, &path).await?;
            }
        }
    }

    // Initialize model layer
    let mm = ModelManager::new().await?;
    let ctx = Ctx::root_ctx();

    // Set demo1 pwd
    let demo_user: User = UserBmc::first_by_username(&ctx, &mm, "dallas")
        .await?
        .unwrap();
    UserBmc::update_pwd(&ctx, &mm, demo_user.id, DEMO_PWD).await?;
    info!("{:<12} - init_dev_db - set demo pwd", "FOR-DEV-ONLY");

    Ok(())
}

async fn pexec(db: &Db, file: &str) -> Result<(), sqlx::Error> {
    info!("{:<12} - pexec: {file}", "FOR-DEV-ONLY");

    // Read the file
    let content = fs::read_to_string(file)?;

    //FIXME:Make the split more SQL proof
    let sqls: Vec<&str> = content.split(';').collect();

    for sql in sqls {
        sqlx::query(sql).execute(db).await?;
    }
    Ok(())
}

async fn new_db_pool(db_con_url: &str) -> Result<Db, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_millis(500))
        .connect(db_con_url)
        .await
}
