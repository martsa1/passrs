use crate::errors::Error;
use anyhow::{anyhow, Result};
use log::{error, trace, warn, debug};
use pgp::{
    composed::{Deserializable, Message},
    types::KeyTrait,
    PublicOrSecret, SignedSecretKey,
};
use std::path::Path;

pub struct KeyAndPassphrasePair<'a> {
    passphrase: &'a str,
    key: &'a SignedSecretKey,
}

impl<'a> KeyAndPassphrasePair<'a> {
    pub fn new(passphrase: &'a str, key: &'a SignedSecretKey) -> Self {
        KeyAndPassphrasePair { passphrase, key }
    }
}

/** Deserialize and de-armour a signing key from disk.
*/
pub fn load_signing_key(key_path: &Path) -> Result<SignedSecretKey> {
    let armoured_key = std::fs::File::open(key_path)?;

    let from_armor_res = pgp::composed::signed_key::parse::from_armor_many(armoured_key);
    if let Ok((items, _)) = from_armor_res {
        for elem in items {
            match elem {
                Ok(data) => match data {
                    PublicOrSecret::Public(pub_key) => {
                        warn!("Public key fingerprint: {:?}", pub_key.fingerprint());
                        return Err(anyhow!("incorrect key type, expected secret, got public"));
                    }
                    PublicOrSecret::Secret(sec_key) => {
                        warn!("Private key fingerprint: {:?}", sec_key.fingerprint());
                        return Ok(sec_key);
                    }
                },
                Err(err) => {
                    error!(
                        "from_armor hit an error with an element of the armoured data: {}",
                        err
                    );
                }
            }
        }
    }

    Err(anyhow!("Failed to load key"))
}

/** Deserialize a GPG file into memory.
*/
pub fn deserialise_message(message_path: &Path) -> Result<Message, Error> {
    let sample_message = std::fs::read(message_path.to_owned())?;
    let message_cursor = std::io::Cursor::new(sample_message);

    match Message::from_bytes(message_cursor) {
        Ok(msg) => Ok(msg),
        Err(err) => Err(err.into()),
    }
}

/** Deccrypt a Message using the provided signing keys.
 *
 * Consumes a `Message` instance when decrypting.
*/
pub fn decrypt_message(
    message: Message,
    signing_keys: &[KeyAndPassphrasePair],
) -> Result<String, Error> {
    // Iterate provided key&pw pairs (to support multi-key password-stores), return on first
    // success.
    //
    // TODO: I believe we should be able to identify precisely which key can decrypt a message
    // using metadata, rather than having to iterate them all...
    // Or - we just pass all the keys to the decrypt function and it'll figure that out for us..
    for key_and_pw in signing_keys {
        match &message {
            Message::Encrypted { .. } => {
                // TODO: Pretty sure this array could be used instead of the top-level for loop
                // here...
                let (mut decrypter, key_ids) = match message
                    .decrypt(|| key_and_pw.passphrase.to_string(), &[&key_and_pw.key])
                {
                    Ok(data) => data,
                    Err(..) => continue,
                };

                // MessageDecrypter doesn't seem to resolve, but its in
                // pgp::composed::message::decrypt, in-essence its an iterator of Results...
                //
                // TODO: Could this be neeater using "if let"?
                let decrypted = match decrypter.next() {
                    Some(dec_res) => match dec_res {
                        Ok(decrypted_m) => {
                            decrypted_m
                        }
                        Err(err) => {
                            debug!("Failed to decrypt message: {:?}", err);
                            continue;
                        }
                    },
                    None => continue,
                };

                match decrypted {
                    Message::Literal(data) => {
                        trace!("Final message: {:?}", data);

                        if data.is_binary() {
                            return match String::from_utf8(data.data().to_vec()) {
                                Err(err) => Err(err.into()),
                                Ok(output) => Ok(output),
                            };
                        } else {
                            return data
                                .to_string()
                                .ok_or_else(|| Error::UnsupportedMessageType {
                                    err: "Failed to decode message data from Str-type Message."
                                        .to_string(),
                                });
                        }
                    }
                    _ => {
                        return Err(Error::UnsupportedMessageType { err: String::new() });
                    }
                }
            }
            _ => {
                return Err(Error::UnsupportedMessageType {
                    err: "Unsupported Message type, only Encrypted messages currently supported."
                        .to_string(),
                });
            }
        }
    }
    Err(Error::NoKey {
        err: "No suitable keys to decrypt message".to_string(),
    })
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;
    use crate::test_util::{init_logs, TmpTree};
    use log::{error, info, warn};
    use pgp::from_armor_many;

    const KEYPHRASE: &str = "sample";

    const SAMPLE_CONTENT: &str = concat!(
        "4gG2y&9?-]AAE(wUnD]v22zs\"nx}ad\n",
        "---\n",
        "username: sample@example.com\n",
        "some_key: foobar\n",
    );

    const SAMPLE_ENTRY: &str = "./src/pgp/sample_entry.gpg";

    const SAMPLE_KEY_ID: &str = "f711232219df6593";
    const SAMPLE_ARMOURED_KEY: &str = "./src/pgp/sample_key.asc";
    const SAMPLE_ARMOURED_PUB_KEY: &str = "./src/pgp/sample_key.pub.asc";
    const ALT_ARMOURED_KEY: &str = "./src/pgp/invalid_key.asc";

    #[test]
    fn test_from_armor() -> Result<()> {
        use pgp::composed::PublicOrSecret;
        use pgp::types::KeyTrait as _;

        init_logs();

        let armoured_key =
            std::fs::File::open(SAMPLE_ARMOURED_KEY).expect("error loading key file");
        let armoured_pub_key =
            std::fs::File::open(SAMPLE_ARMOURED_PUB_KEY).expect("error loading key file");

        let mut public_key = None;
        let mut secret_key = None;

        for target in [armoured_key, armoured_pub_key] {
            info!("Attempting to load key from armoured file");
            let from_armor_res = pgp::composed::signed_key::parse::from_armor_many(target);
            if let Ok((items, mapping)) = from_armor_res {
                warn!("mapping: {:?}", mapping);
                let mut item_count = 0;
                for elem in items {
                    item_count += 1;
                    warn!("elem: {:?}", elem);
                    match elem {
                        Ok(data) => match data {
                            PublicOrSecret::Public(pub_key) => {
                                warn!("Public key fingerprint: {:?}", pub_key.fingerprint());
                                public_key = Some(pub_key);
                            }
                            PublicOrSecret::Secret(sec_key) => {
                                warn!("Private key fingerprint: {:?}", sec_key.fingerprint());
                                secret_key = Some(sec_key);
                            }
                        },
                        Err(err) => {
                            warn!(
                                "from_armor hit an error with an element of the armoured data: {}",
                                err
                            );
                        }
                    }
                }
                warn!("item count: {}", item_count);
            } else {
                error!("failed to de-armour key: {:?}", from_armor_res.err());
            }
        }

        assert!(public_key.is_some());
        assert!(secret_key.is_some());

        Ok(())
    }

    #[test]
    fn test_dearmour_private_key() -> Result<()> {
        //use pgp::types::SecretKeyTrait as _;
        use pgp::types::KeyTrait;

        init_logs();
        let armoured_key =
            std::fs::File::open(SAMPLE_ARMOURED_KEY).expect("error loading key file");

        info!("Parsing armoured key contents.");
        let dearmour = from_armor_many(armoured_key);
        if let Ok(parsed_data) = dearmour {
            info!("Parsed data successfully, extracting secret key.");
            let (items, _) = parsed_data;
            for elem in items {
                if let Ok(data) = elem {
                    if data.is_public() {
                        error!("Found a public key...");
                        return Err(anyhow!(
                            "Incorrect key type, expected private key, got public."
                        ));
                    }
                    if data.is_secret() {
                        info!("Found secret key.");
                        let key = data.into_secret();

                        info!("Validating Key with ID: {}", hex::encode(key.key_id()));
                        key.verify().expect("Invalid key");
                        assert_eq!(SAMPLE_KEY_ID, hex::encode(key.key_id()));

                        //let wrong_key = key.unlock(|| "foobar".to_string(), "");
                        //info!("Err from wrong key: {:?}", wrong_key);
                        //assert!(wrong_key.is_err());
                    }
                } else {
                    warn!("Hit issues de-armouring key: {}", elem.err().unwrap());
                }
            }
        } else {
            error!(
                "Failed to dearmour target key: {}",
                dearmour.as_ref().err().unwrap()
            );
            assert!(dearmour.is_ok());
        }
        Ok(())
    }

    #[test]
    fn test_decrypt_message() -> Result<()> {
        init_logs();

        let message = deserialise_message(&PathBuf::from(SAMPLE_ENTRY))?;

        let signing_key = load_signing_key(&PathBuf::from(SAMPLE_ARMOURED_KEY))?;
        let signing_pairs = [KeyAndPassphrasePair::new(KEYPHRASE, &signing_key)];

        let decrypted_message = decrypt_message(message, &signing_pairs)?;
        assert_eq!(decrypted_message, SAMPLE_CONTENT);
        Ok(())
    }

    #[test]
    fn test_decrypt_message_fails() -> Result<()> {
        init_logs();

        use crate::errors::Error;

        // An incorrect path yields an error.
        let missing_file = deserialise_message(&PathBuf::from("doesnt_exist")).unwrap_err();
        info!("{:?}", missing_file);
        assert!(matches!(missing_file, Error::IOError { .. }));

        // A real file, thats not a GPG un-armoured message, yields an error
        let sample_file_tree = TmpTree::new();
        let not_gpg_file = deserialise_message(&sample_file_tree.expected_files[0]).unwrap_err();
        info!("{:?}", not_gpg_file);
        assert!(matches!(not_gpg_file, Error::PGPError { .. }));

        //GPG message with wrong key
        let message = deserialise_message(&PathBuf::from(SAMPLE_ENTRY)).unwrap();
        let incorrect_key = load_signing_key(&PathBuf::from(ALT_ARMOURED_KEY)).unwrap();
        let key_pw = KeyAndPassphrasePair::new(KEYPHRASE, &incorrect_key);
        let wrong_key = decrypt_message(message, &[key_pw]).unwrap_err();
        info!("{:?}", wrong_key);
        assert!(matches!(wrong_key, Error::NoKey { .. }));

        Ok(())
    }
}
