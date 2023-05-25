use anyhow::{anyhow, Result};
use log::{error, warn};
use pgp::{types::KeyTrait, PublicOrSecret, SignedSecretKey};
use std::path::Path;

fn load_signing_key(key_path: &Path) -> Result<SignedSecretKey> {
    let armoured_key = std::fs::File::open(key_path).expect("error loading key file");

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

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;
    use crate::test_util::init_logs;
    use log::{error, info, warn};
    use pgp::from_armor_many;

    const KEYPHRASE: &str = "sample";

    const SAMPLE_CONTENT: &str = concat!(
        "4gG2y&9?-]AAE(wUnD]v22zs\"nx}ad\n",
        "---\n",
        "username: sample@example.com\n",
        "some_key: foobar",
    );

    const SAMPLE_ENTRY: &str = "./src/pgp/sample_entry.gpg";

    const SAMPLE_KEY_ID: &str = "f711232219df6593";
    const SAMPLE_ARMOURED_KEY: &str = "./src/pgp/sample_key.asc";
    const SAMPLE_ARMOURED_PUB_KEY: &str = "./src/pgp/sample_key.pub.asc";

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
    fn decrypt_message() -> Result<()> {
        use pgp::composed::{Deserializable, Message};

        init_logs();

        let sample_message =
            std::fs::read(PathBuf::from(SAMPLE_ENTRY)).expect("failed to read message.");
        let message_cursor = std::io::Cursor::new(sample_message);

        let message = Message::from_bytes(message_cursor).expect("failed to load messge");

        let signing_key = load_signing_key(&PathBuf::from(SAMPLE_ARMOURED_KEY))?;

        match &message {
            Message::Encrypted { esk, edata } => {
                info!("{:?}", esk);
                info!("{:?}", edata);

                let (mut decrypter, key_ids) = message
                    .decrypt(|| KEYPHRASE.to_string(), &[&signing_key])
                    .expect("Failed to init decrypter");
                let decrypted = decrypter.next().unwrap().unwrap();

                info!("-------------------------------");
                info!("{:?}", decrypted);
                info!("{:?}", key_ids);
                match decrypted {
                    Message::Literal(data) => {
                        info!("Final message: {:?}", data);

                        let raw_data_res = String::from_utf8(data.data().to_vec().into());
                        assert!(raw_data_res.is_ok());
                        info!("{}", raw_data_res?);
                    }
                    _ => panic!("unsupported message type:"),
                }
            }
            _ => panic!("Oh shit..."),
        }
        Ok(())
    }
}
