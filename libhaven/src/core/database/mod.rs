use openmls::prelude::SignaturePublicKey;
use rusqlite::params;
use rusqlite::Connection;

use crate::common::prelude::*;

// pub fn get_signature_pubkeys(connection: &Connection, provider_version: u16) -> Result<Vec<SignaturePublicKey>> {
//             let signature_key = connection
//             .query_row(
//                 "SELECT signature_key
//                 FROM openmls_signature_keys
//                 WHERE public_key = ?1
//                     AND provider_version = ?2",
//                 params![
//                     KeyRefWrapper::<C, _>(public_key, PhantomData),
//                     STORAGE_PROVIDER_VERSION
//                 ],
//                 |row| {
//                     let EntityWrapper::<C, _>(signature_key, ..) = row.get(0)?;
//                     Ok(signature_key)
//                 },
//             )
//             .optional()?;
//         Ok(signature_key)
//     todo!()
// }
