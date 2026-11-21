//! Compte universel (#20), étape 1 : export / import manuel d'un fichier chiffré.
//!
//! - `crypto`   : dérivation de clé (Argon2id) et chiffrement du fichier (XChaCha20-Poly1305).
//! - `snapshot` : contenu synchronisable (profils, favoris, progression, presets, réglages).
//! - `merge`    : fusion d'un snapshot dans la base locale (le plus récent gagne).
//!
//! La logique est indépendante de l'OS (mêmes commandes Tauri sous Linux et Windows) et sera
//! réutilisée telle quelle par la synchronisation automatique (étape 2).

pub mod crypto;
pub mod merge;
pub mod snapshot;
