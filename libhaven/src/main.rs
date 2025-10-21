use core::provider::{JsonCodec, SqliteOpenMlsProvider};
use openmls::prelude::{tls_codec::*, *};
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::{OpenMlsRustCrypto, RustCrypto};
use openmls_sqlite_storage::{Codec, SqliteStorageProvider};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

mod common;
mod core;

fn main() {
    // Define ciphersuite ...
    // ... and the crypto provider to use.
    let sasha_connection = SqliteOpenMlsProvider::new("sasha.db").unwrap();
    let maxim_connection = SqliteOpenMlsProvider::new("maxim.db").unwrap();

    // Now let's create two participants.

    // A helper to create and store credentials.
    fn generate_credential_with_key(
        identity: Vec<u8>,
        credential_type: CredentialType,
        signature_algorithm: SignatureScheme,
        provider: &impl OpenMlsProvider,
    ) -> (CredentialWithKey, SignatureKeyPair) {
        let credential = BasicCredential::new(identity);
        let signature_keys = SignatureKeyPair::new(signature_algorithm)
            .expect("Error generating a signature key pair.");

        // Store the signature key into the key store so OpenMLS has access
        // to it.
        signature_keys
            .store(provider.storage())
            .expect("Error storing signature keys in key store.");

        (
            CredentialWithKey {
                credential: credential.into(),
                signature_key: signature_keys.public().into(),
            },
            signature_keys,
        )
    }

    // A helper to create key package bundles.
    fn generate_key_package(
        ciphersuite: Ciphersuite,
        provider: &impl OpenMlsProvider,
        signer: &SignatureKeyPair,
        credential_with_key: CredentialWithKey,
    ) -> KeyPackageBundle {
        // Create the key package
        let key_package = KeyPackage::builder()
            .build(ciphersuite, provider, signer, credential_with_key)
            .unwrap();
        todo!()
    }

    // First they need credentials to identify them
    let (sasha_credential_with_key, sasha_signer) = generate_credential_with_key(
        "Sasha".into(),
        CredentialType::Basic,
        crate::core::provider::CIPHERSUITE.signature_algorithm(),
        &sasha_connection,
    );

    let (maxim_credential_with_key, maxim_signer) = generate_credential_with_key(
        "Maxim".into(),
        CredentialType::Basic,
        crate::core::provider::CIPHERSUITE.signature_algorithm(),
        &maxim_connection,
    );

    // Then they generate key packages to facilitate the asynchronous handshakes
    // in MLS

    // Generate KeyPackages
    let maxim_key_package = generate_key_package(
        crate::core::provider::CIPHERSUITE,
        &maxim_connection,
        &maxim_signer,
        maxim_credential_with_key,
    );

    // Now in practice, Maxim would need to upload this keypackage somewhere.
    // And Sasha would have to retrieve it.

    // Now Sasha starts a new group ...
    let mut sasha_group = MlsGroup::new(
        &sasha_connection,
        &sasha_signer,
        &MlsGroupCreateConfig::default(),
        sasha_credential_with_key,
    )
    .expect("An unexpected error occurred.");

    // ... and invites Maxim.
    // The key package has to be retrieved from Maxim in some way. Most likely
    // via a server storing key packages for users.
    // mls_message_out is the commit message that would need to be sent to all
    // existing group members if this wasn't a 1-on-1 chat.
    // welcome_out and group_info should be sent to the individual that is being added.
    let (mls_message_out, welcome_out, group_info) = sasha_group
        .add_members(
            &sasha_connection,
            &sasha_signer,
            std::slice::from_ref(maxim_key_package.key_package()),
        )
        .expect("Could not add members.");

    // Sasha merges the pending commit that adds Maxim.
    sasha_group
        .merge_pending_commit(&sasha_connection)
        .expect("error merging pending commit");

    // Sasha serializes the [`MlsMessageOut`] containing the [`Welcome`].
    let serialized_welcome = tls_codec::Serialize::tls_serialize_detached(&welcome_out)
        .expect("Error serializing welcome");

    // Somehow Sasha needs to send this to Maxim.

    // Maxim can now de-serialize the message as an [`MlsMessageIn`] ...
    let mls_message_in = <MlsMessageIn as tls_codec::Deserialize>::tls_deserialize(
        &mut serialized_welcome.as_slice(),
    )
    .expect("An unexpected error occurred.");

    // ... and inspect the message.
    let welcome = match mls_message_in.extract() {
        MlsMessageBodyIn::Welcome(welcome) => welcome,
        // We know it's a welcome message, so we ignore all other cases.
        _ => unreachable!("Unexpected message type."),
    };

    // Now Maxim can build a staged join for the group in order to inspect the welcome
    let maxim_staged_join = StagedWelcome::new_from_welcome(
        &maxim_connection,
        &MlsGroupJoinConfig::default(),
        welcome,
        // The public tree is needed and transferred out of band.
        // It is also possible to use the [`RatchetTreeExtension`]
        Some(sasha_group.export_ratchet_tree().into()),
    )
    .expect("Error creating a staged join from Welcome");

    // Finally, Maxim can create the group
    let mut maxim_group = maxim_staged_join
        .into_group(&maxim_connection)
        .expect("Error creating the group from the staged join");

    // Now sasha can send Maxim a message!
    let message_alice = b"Hi, I'm Alice!";
    let mls_message_out = sasha_group
        .create_message(&sasha_connection, &sasha_signer, message_alice)
        .expect("Error creating application message.");
    // Serialize the message.
    let serialized_message = tls_codec::Serialize::tls_serialize_detached(&mls_message_out)
        .expect("Error serializing message to Maxim.");

    // Now Maxim can deserialize it.
    let mls_message =
        <MlsMessageIn as tls_codec::Deserialize>::tls_deserialize_exact(serialized_message)
            .expect("Could not deserialize message.");
    let protocol_message = mls_message
        .try_into_protocol_message()
        .expect("Could not convert message to protocol message.");
    println!("{:?}", protocol_message);
    let processed_message = maxim_group
        .process_message(&maxim_connection, protocol_message)
        .expect("Could not process message.");
    println!("processed message: {:?}", processed_message);
    let message_content = processed_message.into_content();
    println!("message content: {:?}", message_content);
    if let ProcessedMessageContent::ApplicationMessage(application_message) = message_content {
        // Check the message
        assert_eq!(application_message.into_bytes(), b"Hi, I'm Alice!");
    }

    println!("Done!")
}
