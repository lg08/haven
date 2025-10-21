use crate::error::Error;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct DbConnection<'conn> {
    connection: UnderlyingDbConnection<'conn>,
}

#[derive(Debug)]
enum UnderlyingDbConnection<'conn> {
    Connection(rusqlite::Connection),
    Savepoint(rusqlite::Savepoint<'conn>),
}

// Implement Deref to expose &Connection methods
impl<'conn> Deref for UnderlyingDbConnection<'conn> {
    type Target = rusqlite::Connection;

    fn deref(&self) -> &Self::Target {
        match self {
            UnderlyingDbConnection::Connection(conn) => conn,
            UnderlyingDbConnection::Savepoint(sp) => sp.deref(),
        }
    }
}

impl<'conn> Deref for DbConnection<'conn> {
    type Target = rusqlite::Connection;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl DbConnection<'_> {
    // Now all these methods become trivial!
    pub fn execute<P: rusqlite::Params>(&self, sql: &str, params: P) -> Result<usize, Error> {
        self.connection
            .execute(sql, params)
            .map_err(|e| Error::DatabaseError(e.to_string()))
    }

    pub fn execute_batch(&self, sql: &str) -> Result<(), Error> {
        self.connection
            .execute_batch(sql)
            .map_err(|e| Error::DatabaseError(e.to_string()))
    }

    pub fn query_row<T, P, F>(&self, sql: &str, params: P, f: F) -> Result<T, Error>
    where
        P: rusqlite::Params,
        F: FnOnce(&rusqlite::Row<'_>) -> Result<T, rusqlite::Error>,
    {
        self.connection
            .query_row(sql, params, f)
            .map_err(|e| Error::DatabaseError(e.to_string()))
    }

    pub fn prepare(&self, sql: &str) -> Result<rusqlite::Statement<'_>, Error> {
        self.connection
            .prepare(sql)
            .map_err(|_| Error::DatabaseError("Error preparing db connection.".to_string()))
    }

    pub fn last_insert_rowid(&self) -> i64 {
        self.connection.last_insert_rowid()
    }

    // Only new_transaction still needs special handling
    pub fn new_transaction(&mut self) -> Result<DbConnection<'_>, Error> {
        let sp = match &mut self.connection {
            UnderlyingDbConnection::Connection(conn) => conn.savepoint(),
            UnderlyingDbConnection::Savepoint(sp) => sp.savepoint(),
        }
        .map_err(|_| Error::DatabaseError("Error starting transaction.".to_string()))?;

        Ok(DbConnection {
            connection: UnderlyingDbConnection::Savepoint(sp),
        })
    }

    pub fn commit(self) -> Result<(), Error> {
        match self.connection {
            UnderlyingDbConnection::Connection(_) => Err(Error::DatabaseError(String::from(
                "Transaction has not been started. Cannot commit unstarted transaction.",
            ))),
            UnderlyingDbConnection::Savepoint(savepoint) => savepoint
                .commit()
                .map_err(|_| Error::DatabaseError("Error committing transaction.".to_string())),
        }
    }
}
