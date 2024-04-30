use tokio_postgres::{Config, Connection, NoTls, Error, Row, types::{ToSql, Type}};
use std::collections::HashMap;

use crate::models::user_models::User;

pub struct DbConnection {
    client: tokio_postgres::Client,
}
impl DbConnection {
    fn new(client: tokio_postgres::Client) -> Self {
        DbConnection { client }
    }

    pub async fn execute_delete_query_with_rollback(&mut self, user_id: &i32) -> Result<u64, Error> {
        let transaction = self.client.transaction().await?;

        // 在事务中执行删除操作
        let delete_result = transaction.execute("DELETE FROM public.user WHERE id = $1", &[user_id]).await;

        // 根据删除操作的结果决定是提交事务还是回滚事务
        match delete_result {
            Ok(rows_affected) => {
                // 提交事务
                transaction.commit().await?;
                Ok(rows_affected as u64)
            }
            Err(e) => {
                // 回滚事务
                transaction.rollback().await?;
                Err(e)
            }
        }
    }
}

pub async fn get_db_connection(connection_str: &str) -> Result<DbConnection, Error>{
    let (client, connection) = tokio_postgres::connect(connection_str, NoTls).await?;

    // test the connection
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(DbConnection::new(client))
}

pub async fn execute_query(connection: &DbConnection, query: &str) -> Result<Vec<Row>, Error> {
    let rows = connection.client
        .query(query, &[])
        .await?;
    Ok(rows)
}

pub async fn execute_query_user_by_email(connection: &DbConnection, email: &str) -> Result<Vec<Row>, Error> {
    let query = "select * from public.user where email = $1";
    let rows = connection.client
        .query(query, &[&email])
        .await?;
    Ok(rows)
}

pub async fn execute_insert_user(connection: &DbConnection, user: &User) -> Result<Vec<Row>, Error>
{
    let query = "insert into public.user (name, email, age, pwd) values ($1, $2, $3, $4) returning id";
    let rows = connection.client
        .query(query, &[
            &user.name,
            &user.email,
            &user.age,
            &user.pwd,
        ])
        .await?;
    Ok(rows)
}

pub async fn fetch_insert_id(rows: &Vec<Row>) -> Result<i32, Error> {
    let row = rows.get(0).unwrap();
    let id: i32 = row.get(0);
    Ok(id)
}

pub async fn execute_update_user(connection: &DbConnection, user: &User) -> Result<u64, Error> {
    let query = "update public.user set name = $1, age = $2 where id = $3";
    let rows = connection.client
        .execute(query, &[
            &user.name,
            &user.age,
            &user._id,
        ])
        .await?;
    Ok(rows)
}

pub async fn execute_delete_query(connection: &DbConnection, user_id: &i32) -> Result<u64, Error> {
    let query = "delete from public.user where id = $1";
    let rows = connection.client
        .execute(query, &[user_id])
        .await?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_db_connection() {
        let connection_str = "postgresql://postgres:root@localhost:5432/rust";
        let connection = get_db_connection(connection_str).await.unwrap();
        assert_eq!(connection.client.is_closed(), false);
    }

    #[tokio::test]
    async fn test_execute_query() {
        let connection_str = "postgresql://postgres:root@localhost:5432/rust";
        let connection = get_db_connection(connection_str).await.unwrap();
        let query = "select * from public.user u ";
        let rows = execute_query(&connection, query).await.unwrap();
        println!("{:?}", rows);
        assert_eq!(rows.len() > 0, true);
    }

    #[tokio::test]
    async fn test_execute_transaction_query() {
        let connection_str = "postgresql://postgres:root@localhost:5432/rust";
        let connection = get_db_connection(connection_str).await.unwrap();
        let user = User::new(0,"test".to_string(), "test@gmail".to_string(), 20, "test".to_string());
        let rows = execute_insert_user(&connection, &user).await.unwrap();
        let id = fetch_insert_id(&rows).await.unwrap();
        println!("{:?}", id);
        assert_eq!(id > 0, true);
        // update the user
        let user = User::new(id,"test2".to_string(), "test2@gmail".to_string(), 21, "test2".to_string());
        let rows = execute_update_user(&connection, &user).await.unwrap();
        assert_eq!(rows, 1);
        // delete the user
        let rows = execute_delete_query(&connection, &id).await.unwrap();
        assert_eq!(rows, 1);
    }

}


