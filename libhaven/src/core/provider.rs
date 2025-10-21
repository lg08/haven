use crate::common::prelude::*;
use openmls::prelude::{tls_codec::*, *};
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::{OpenMlsRustCrypto, RustCrypto};
use openmls_sqlite_storage::{Codec, SqliteStorageProvider};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

pub const CIPHERSUITE: Ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

#[derive(Default)]
pub struct JsonCodec;

impl Codec for JsonCodec {
    type Error = serde_json::Error;

    fn to_vec<T: Serialize>(value: &T) -> std::result::Result<Vec<u8>, Self::Error> {
        serde_json::to_vec(value)
    }

    fn from_slice<T: serde::de::DeserializeOwned>(
        slice: &[u8],
    ) -> std::result::Result<T, Self::Error> {
        serde_json::from_slice(slice)
    }
}

pub struct SqliteOpenMlsProvider<'a> {
    pub crypto: RustCrypto,
    pub key_store: SqliteStorageProvider<JsonCodec, &'a Connection>,
    pub db_connection: rusqlite::Connection,
}

impl SqliteOpenMlsProvider {
    pub fn new(db_path: &str) -> Result<Self> {
        let connection = rusqlite::Connection::open(db_path)?;
        let mut storage =
            openmls_sqlite_storage::SqliteStorageProvider::<JsonCodec, &mut Connection>::new(
                &mut connection,
            );
        storage.run_migrations().expect("Failed to run migrations.");
        let provider = SqliteOpenMlsProvider {
            crypto: RustCrypto::default(),
            key_store: storage,
            db_connection: connection,
        };
        Ok(provider)
    }
}

impl OpenMlsProvider for SqliteOpenMlsProvider {
    type CryptoProvider = RustCrypto;
    type RandProvider = RustCrypto;
    type StorageProvider = SqliteStorageProvider<JsonCodec, Connection>;

    fn storage(&self) -> &Self::StorageProvider {
        &self.key_store
    }

    fn crypto(&self) -> &Self::CryptoProvider {
        &self.crypto
    }

    fn rand(&self) -> &Self::RandProvider {
        &self.crypto
    }
}
