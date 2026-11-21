//! Chiffrement du fichier de compte.
//!
//! Format (tout en big-endian) :
//! `MAGIC(8) | version(1) | m_cost(4) | t_cost(4) | p_cost(4) | salt(16) | nonce(24) | ciphertext`
//!
//! La phrase secrète passe par Argon2id (64 octets en sortie) : les 32 premiers octets forment la
//! clé de chiffrement, les 32 suivants sont réservés à la « preuve d'accès » de l'étape 2 (envoyée
//! au serveur, qui ne voit jamais la phrase ni la clé de chiffrement). L'en-tête est authentifié
//! (AAD) : toute modification fait échouer le déchiffrement.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand::RngCore;
use thiserror::Error;
use zeroize::Zeroize;

const MAGIC: &[u8; 8] = b"BIPTVACC";
const FORMAT_VERSION: u8 = 1;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;
const HEADER_LEN: usize = 8 + 1 + 4 + 4 + 4 + SALT_LEN + NONCE_LEN;

/// Longueur minimale d'une phrase secrète choisie par l'utilisateur (en caractères).
pub const MIN_PASSPHRASE_CHARS: usize = 25;

// Bornes lues dans un fichier : évitent qu'un fichier forgé exige des ressources démesurées.
const MAX_M_COST_KIB: u32 = 1024 * 1024; // 1 GiB
const MAX_T_COST: u32 = 10;
const MAX_P_COST: u32 = 8;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("La phrase secrète doit contenir au moins {MIN_PASSPHRASE_CHARS} caractères")]
    PassphraseTooShort,
    #[error("Ce n'est pas un fichier de compte Better IPTV (en-tête inconnu)")]
    BadMagic,
    #[error("Version de fichier non prise en charge : {0}")]
    UnsupportedVersion(u8),
    #[error("Fichier de compte tronqué ou corrompu")]
    Truncated,
    #[error("Paramètres de dérivation hors limites dans le fichier")]
    BadParams,
    #[error("Phrase secrète incorrecte ou fichier modifié")]
    Decrypt,
    #[error("Erreur de chiffrement : {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    pub m_cost_kib: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

impl KdfParams {
    /// Paramètres de production : 64 MiB, 3 passes, 1 voie (environ 0,3 à 1 s sur le MacBook T2).
    pub const DEFAULT: KdfParams = KdfParams {
        m_cost_kib: 64 * 1024,
        t_cost: 3,
        p_cost: 1,
    };

    /// Paramètres légers pour les tests unitaires uniquement.
    #[cfg(test)]
    pub const TEST: KdfParams = KdfParams {
        m_cost_kib: 64,
        t_cost: 1,
        p_cost: 1,
    };

    fn validate(&self) -> Result<(), CryptoError> {
        if self.m_cost_kib > MAX_M_COST_KIB
            || self.t_cost == 0
            || self.t_cost > MAX_T_COST
            || self.p_cost == 0
            || self.p_cost > MAX_P_COST
            || self.m_cost_kib < 8 * self.p_cost
        {
            return Err(CryptoError::BadParams);
        }
        Ok(())
    }
}

/// Clés dérivées de la phrase secrète. Effacées de la mémoire à la destruction.
pub struct DerivedKeys {
    pub enc_key: [u8; 32],
    /// Réservée à l'étape 2 (preuve d'accès envoyée au serveur).
    #[allow(dead_code)]
    pub access_proof: [u8; 32],
}

impl Drop for DerivedKeys {
    fn drop(&mut self) {
        self.enc_key.zeroize();
        self.access_proof.zeroize();
    }
}

/// Vérifie la longueur d'une phrase secrète (minimum 25 caractères).
pub fn validate_passphrase(passphrase: &str) -> Result<(), CryptoError> {
    if passphrase.chars().count() < MIN_PASSPHRASE_CHARS {
        return Err(CryptoError::PassphraseTooShort);
    }
    Ok(())
}

pub fn derive_keys(
    passphrase: &str,
    salt: &[u8],
    params: &KdfParams,
) -> Result<DerivedKeys, CryptoError> {
    params.validate()?;
    let argon_params = Params::new(params.m_cost_kib, params.t_cost, params.p_cost, Some(64))
        .map_err(|e| CryptoError::Internal(e.to_string()))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);

    let mut out = [0u8; 64];
    argon
        .hash_password_into(passphrase.as_bytes(), salt, &mut out)
        .map_err(|e| CryptoError::Internal(e.to_string()))?;

    let mut enc_key = [0u8; 32];
    let mut access_proof = [0u8; 32];
    enc_key.copy_from_slice(&out[..32]);
    access_proof.copy_from_slice(&out[32..]);
    out.zeroize();
    Ok(DerivedKeys {
        enc_key,
        access_proof,
    })
}

/// Chiffre `plaintext` avec une phrase secrète (la longueur minimale doit déjà être validée).
pub fn encrypt(
    plaintext: &[u8],
    passphrase: &str,
    params: &KdfParams,
) -> Result<Vec<u8>, CryptoError> {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    let mut rng = rand::rng();
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut nonce);

    let keys = derive_keys(passphrase, &salt, params)?;

    let mut out = Vec::with_capacity(HEADER_LEN + plaintext.len() + 16);
    out.extend_from_slice(MAGIC);
    out.push(FORMAT_VERSION);
    out.extend_from_slice(&params.m_cost_kib.to_be_bytes());
    out.extend_from_slice(&params.t_cost.to_be_bytes());
    out.extend_from_slice(&params.p_cost.to_be_bytes());
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce);

    let cipher = XChaCha20Poly1305::new((&keys.enc_key).into());
    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: plaintext,
                aad: &out[..HEADER_LEN],
            },
        )
        .map_err(|_| CryptoError::Internal("chiffrement impossible".into()))?;
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Déchiffre un fichier produit par [`encrypt`].
pub fn decrypt(data: &[u8], passphrase: &str) -> Result<Vec<u8>, CryptoError> {
    if data.len() < 8 || &data[..8] != MAGIC {
        return Err(CryptoError::BadMagic);
    }
    if data.len() < HEADER_LEN + 16 {
        return Err(CryptoError::Truncated);
    }
    let version = data[8];
    if version != FORMAT_VERSION {
        return Err(CryptoError::UnsupportedVersion(version));
    }
    let read_u32 = |at: usize| u32::from_be_bytes(data[at..at + 4].try_into().unwrap());
    let params = KdfParams {
        m_cost_kib: read_u32(9),
        t_cost: read_u32(13),
        p_cost: read_u32(17),
    };
    let salt = &data[21..21 + SALT_LEN];
    let nonce = &data[21 + SALT_LEN..HEADER_LEN];

    let keys = derive_keys(passphrase, salt, &params)?;
    let cipher = XChaCha20Poly1305::new((&keys.enc_key).into());
    cipher
        .decrypt(
            XNonce::from_slice(nonce),
            Payload {
                msg: &data[HEADER_LEN..],
                aad: &data[..HEADER_LEN],
            },
        )
        .map_err(|_| CryptoError::Decrypt)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASS: &str = "une phrase secrete de test tres longue";

    #[test]
    fn roundtrip() {
        let data = b"{\"hello\":\"monde\"}";
        let enc = encrypt(data, PASS, &KdfParams::TEST).unwrap();
        assert_eq!(&enc[..8], MAGIC);
        assert_eq!(decrypt(&enc, PASS).unwrap(), data);
    }

    #[test]
    fn two_encryptions_differ() {
        let a = encrypt(b"x", PASS, &KdfParams::TEST).unwrap();
        let b = encrypt(b"x", PASS, &KdfParams::TEST).unwrap();
        assert_ne!(a, b, "sel et nonce doivent être aléatoires");
    }

    #[test]
    fn wrong_passphrase_fails() {
        let enc = encrypt(b"secret", PASS, &KdfParams::TEST).unwrap();
        assert!(matches!(
            decrypt(&enc, "une autre phrase secrete tres longue"),
            Err(CryptoError::Decrypt)
        ));
    }

    #[test]
    fn tampering_fails() {
        let enc = encrypt(b"secret", PASS, &KdfParams::TEST).unwrap();
        // Octet du texte chiffré
        let mut a = enc.clone();
        let last = a.len() - 1;
        a[last] ^= 1;
        assert!(matches!(decrypt(&a, PASS), Err(CryptoError::Decrypt)));
        // Octet du sel (en-tête authentifié)
        let mut b = enc.clone();
        b[25] ^= 1;
        assert!(matches!(decrypt(&b, PASS), Err(CryptoError::Decrypt)));
    }

    #[test]
    fn rejects_garbage_and_forged_params() {
        assert!(matches!(decrypt(b"nope", PASS), Err(CryptoError::BadMagic)));
        let enc = encrypt(b"secret", PASS, &KdfParams::TEST).unwrap();
        assert!(matches!(
            decrypt(&enc[..HEADER_LEN], PASS),
            Err(CryptoError::Truncated)
        ));
        // m_cost forgé à une valeur démesurée : refusé avant toute dérivation
        let mut forged = enc.clone();
        forged[9..13].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(matches!(decrypt(&forged, PASS), Err(CryptoError::BadParams)));
        let mut v2 = enc.clone();
        v2[8] = 2;
        assert!(matches!(
            decrypt(&v2, PASS),
            Err(CryptoError::UnsupportedVersion(2))
        ));
    }

    #[test]
    fn passphrase_minimum_is_25_chars() {
        assert!(validate_passphrase(&"a".repeat(24)).is_err());
        assert!(validate_passphrase(&"a".repeat(25)).is_ok());
        // Compte des caractères, pas des octets : 25 accents = 50 octets
        assert!(validate_passphrase(&"é".repeat(25)).is_ok());
        assert!(validate_passphrase(&"é".repeat(24)).is_err());
    }

    #[test]
    fn keys_depend_on_passphrase_and_salt_and_are_distinct() {
        let k1 = derive_keys(PASS, &[1u8; 16], &KdfParams::TEST).unwrap();
        let k2 = derive_keys(PASS, &[2u8; 16], &KdfParams::TEST).unwrap();
        let k3 = derive_keys("autre phrase secrete de test", &[1u8; 16], &KdfParams::TEST).unwrap();
        assert_ne!(k1.enc_key, k2.enc_key);
        assert_ne!(k1.enc_key, k3.enc_key);
        assert_ne!(k1.enc_key, k1.access_proof);
    }
}
