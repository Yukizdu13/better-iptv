# CLAUDE.md — better-iptv (fork Yukizdu13)

> **Ce fichier doit TOUJOURS être tenu à jour.** Toute modification qui change la structure du repo, les commandes, les plateformes, le flux git/release ou une règle ci-dessous impose de mettre à jour ce fichier **dans le même commit**. Lire ce fichier = le corriger s'il est périmé.
> Fichier compagnon, à tenir à jour avec lui : [`docs/ARBORESCENCE_PLATEFORMES.md`](docs/ARBORESCENCE_PLATEFORMES.md) (fichiers nécessaires par plateforme).
> Dernière mise à jour : 2026-11-21 (version de l'app : 2.6.1 ; compte universel étape 1 ajoutée ; intégration Atlas Pro livrée).

Les règles universelles de l'utilisateur sont chargées via son `~/.claude/CLAUDE.md` (hors repo) ; ce fichier ne contient que ce qui est propre au projet.

## Projet

Lecteur IPTV multiplateforme : UI web (React/TypeScript) dans Tauri 2, backend Rust, SQLite, lecture vidéo via **MPV** (processus externe) sur desktop, `<video>` HTML5 sur iOS.
Playlists M3U et Xtream Codes (live, VOD, séries, EPG XMLTV), profils, favoris, contrôle parental (PIN Argon2), play aléatoire, reprise des séries.

- Produit : « Better IPTV+Kev » — identifiant `com.m0s.better-ip-tv` — crate/paquet npm `better-ip-tv`
- `mewset/better-iptv` = upstream (**on n'y pousse jamais**) ; `Yukizdu13/better-iptv` = le fork (**on pousse ici**). Le nom du remote dépend du clone : sur kevin-kubuntu `fork` = Yukizdu13 (`git push fork main`) ; sur KEVIN-MSI (clone Windows) `origin` = Yukizdu13 (`git push origin main`). Vérifier avec `git remote -v` avant de pousser.
- Backlog : `prochaine_maj.json` à la racine (les issues GitHub sont désactivées sur le fork). Y noter toute nouvelle idée/bug avec les champs `distributions`, `statut` (Linux), `statut_windows` (`fait` | `a_faire` | `a_valider`) et, si besoin, `notes_windows` ; les décisions d'une entrée vont dans `decisions` (cf. #20, compte universel).
- **Reste à faire du clone Windows (KEVIN-MSI)** : fichier local `RESTE_A_FAIRE.local.md` à la racine (validations, nettoyage d'ATLAS-PRO promis après les tests de l'utilisateur, backlog code). **Non versionné** : exclu via `.git/info/exclude`, absent des autres clones (kevin-kubuntu) et du dépôt public. S'il existe, le lire en début de session et le tenir à jour ; ne jamais le committer ni y mettre d'identifiants.

## Stack

| Couche | Technos |
|---|---|
| Frontend | React 19, TypeScript 5.8, Vite 7, Tailwind 3, Zustand, Radix Tabs, TanStack Virtual, Vitest |
| Backend | Rust (édition 2021), Tauri 2, rusqlite (bundled) + r2d2, reqwest, tokio, quick-xml, argon2 |
| Lecture | MPV via IPC (socket Unix / named pipe Windows) ; iOS : WKWebView `<video>` |

## Structure

```
src/                  Frontend (components/, hooks/, lib/, stores/, types/, test/)
src-tauri/src/        Backend Rust : commands/ (API Tauri), db/, playback/ (mpv.rs),
                      playlist/ (parser M3U + xtream.rs), epg/, *_domain/ (dont sync_domain/ :
                      compte universel, #20), state.rs, lib.rs
src-tauri/            Cargo.toml, tauri.conf.json, capabilities/, icons/, templates/ (.desktop)
resources/mpv/        mpv.exe + DLL embarqués (Windows uniquement, vide dans le repo)
.github/workflows/    release.yml (3 OS), test.yml (Ubuntu), ios.yml (.ipa non signé)
docs/, scripts/       Doc iOS, scripts CI locale / patch Info.plist iOS
prochaine_maj.json    Backlog
```

Détail par plateforme : voir `docs/ARBORESCENCE_PLATEFORMES.md`.

## Commandes

```bash
npm run tauri dev                              # dev (Vite + Tauri)
npm run lint && npm run format:check           # frontend (CI : prettier --check src)
npm run test:run                               # Vitest
cd src-tauri && cargo test                     # tests Rust
cd src-tauri && cargo clippy --all-targets -- -D warnings -A dead_code   # exigé par la CI
scripts/ci-test-local.sh [--with-build]        # rejoue la CI en local
npm run tauri build -- --bundles deb           # paquet Linux
```

## Plateformes (résumé)

| Plateforme | Statut | Machine de build |
|---|---|---|
| Linux (Kubuntu 24.04) | utilisé au quotidien, référence du champ `statut` | kevin-kubuntu (MacBook T2) |
| Windows 11 | build NSIS OK, `cargo test` OK (108 tests) ; lecture depuis l'UI et IPC MPV à valider | KEVIN-MSI |
| macOS | prévu en CI upstream, non testé sur le fork | — |
| iOS | `.ipa` non signé via GitHub Actions (sideload) | CI uniquement |
| Android | non pris en charge | — |

## Règles de travail

- Réponses et commits **en français** ; commits au format conventionnel (`feat:`, `fix:`, `docs:`).
- **Ne jamais mentionner Claude comme co-auteur** dans un commit (pas de ligne `Co-Authored-By`).
- Ne committer que les fichiers de la tâche (l'arbre de travail contient souvent d'autres modifications en cours) : `git add <chemins>`, jamais `git add -A`.
- **Version** : source de vérité `package.json`, à reporter à la main dans `src-tauri/Cargo.toml` et `src-tauri/tauri.conf.json` (les scripts `dev-scripts/` cités par `package.json` sont gitignorés et absents).
- Toute fonctionnalité qui touche la lecture ou les chemins de fichiers doit dire explicitement ce qui change sous **Linux / Windows / iOS** et mettre à jour `prochaine_maj.json` (`statut_windows`).
- Doc à jour : toute modification de build, de dépendance ou de fichier propre à une plateforme met à jour `docs/ARBORESCENCE_PLATEFORMES.md` ; tout changement de commande ou de flux met à jour ce fichier.
- Le `.gitignore` upstream liste `CLAUDE.md` (« NEVER commit ») : sur ce fork, il a été ajouté avec `git add -f`. Il est donc suivi ; les modifications suivantes se committent normalement.

## Pièges connus

- **Fournisseur Atlas Pro** (1 seule connexion simultanée : fermer l'appli ATLAS-PRO officielle avant de lire) : API Xtream standard, mais `season.id` entier, `rating` tantôt chaîne tantôt nombre, `info: []` quand vide, et VOD en `.mkv`/`.avi` (l'URL doit porter le vrai `container_extension`, sinon 404). Tout est géré par la désérialisation tolérante de `xtream.rs` ; tests dans `atlas_pro_tests`. Les URL VOD sont stockées en base à l'import : rafraîchir le profil après une mise à jour de ce code. Les fiches film envoient **les deux variantes** de chaque champ (`plot`+`description`, `cast`+`actors`, `movie_image`+`cover_big`) : ne jamais utiliser `#[serde(alias)]` pour ça (erreur « duplicate field », fiche vide) mais des champs séparés fusionnés (`VodMetadataRaw`). `de_object_or_default` ne masque que les valeurs non-objet (`[]`/null), jamais une erreur sur un objet. La 1re connexion après inactivité met ~8 s (puis 0,1 s) : `connect_timeout` = 20 s dans `http.rs` ; avec 10 s le 1er essai échouait et était refait. Tout test sur données réelles doit **vérifier le contenu**, pas seulement l'absence d'erreur.
- **Formats Xtream** : les fournisseurs renvoient des types variables (`season_number` int/string, `episodes` en liste de listes quand une saison 0 existe). Toujours désérialiser de façon tolérante (`de_string_or_int`, `de_episodes` dans `playlist/xtream.rs`) plutôt que du code par fournisseur.
- **Dépôt public** : `fork` est public. Aucune info d'infrastructure perso dans le repo ni dans les messages de commit (hôtes, domaine, IP, ports, chemins serveur, config serveur) : les détails vont dans les notes privées de l'utilisateur, le backlog reste générique (« serveur personnel »). Relire le diff avant chaque commit de docs.
- **Compte universel (#20, étape 1)** : Paramètres > Compte exporte/importe un fichier `.biptv` chiffré (Argon2id + XChaCha20-Poly1305, phrase ≥ 25 caractères) ; code dans `src-tauri/src/sync_domain/` (crypto, snapshot, fusion « le plus récent gagne ») et `commands/account.rs`. Favoris/progression/presets sont identifiés par une clé stable `(uuid du profil, clé de chaîne)`, jamais par `channel_id`. L'import est aussi dans un 3e onglet « Compte » de la page d'accueil (Setup.tsx), pour un appareil vide. Linux : testé en unitaire ; Windows : à valider ; iOS : onglet masqué.
- **Images** : `ChannelCard` se replie sur la pastille à initiale si l'image échoue (liens morts chez le fournisseur ≈ 6 %, films sans icône ≈ 6 %) ; `SeriesView`/`VodView` n'ont pas encore ce repli.
- **Données sensibles** : la base SQLite de l'utilisateur contient les identifiants Xtream **en clair** → ne jamais les afficher, logger, ni les mettre dans un fichier ou une mémoire. Les URL sont masquées dans les logs (`utils::mask_credentials`).
- **Linux/Wayland** : `WEBKIT_DISABLE_DMABUF_RENDERER=1` est forcé (`lib.rs` et `src-tauri/templates/better-iptv.desktop`) — ne pas retirer.
- **Menu KDE « Better IPTV+Kev »** = binaire **installé** (`/usr/bin/better-ip-tv`, paquet `better-iptv-kev`), pas le working tree : pour tester du code neuf, `npm run tauri dev` ou rebuild + `sudo dpkg -i` du `.deb`.
- **Node.js absent sur kevin-kubuntu** (purgé) : `npm`, le build frontend et le `.deb` se font depuis KEVIN-MSI ou un Node installé à la demande. `cargo` est disponible ici. Pour rejouer prettier/eslint/vitest sans Node système : Node 20 portable (tarball officiel nodejs.org, checksum vérifié via `SHASUMS256.txt`) décompressé dans un dossier temporaire et lancé directement (`node node_modules/prettier/bin/prettier.cjs --check src`) ; l'utilisateur doit l'autoriser.
- **CI ≠ machine locale** : la CI (`Test Build`) utilise Rust `stable` (1.100 le 2026-11-21) alors que kevin-kubuntu a 1.96 : un nouveau lint clippy peut casser la CI sans changement de code (cas réel : `needless_bool`, `unnecessary_sort_by`). Si tout passe en local mais pas en CI : `gh run view <id> --repo Yukizdu13/better-iptv --log-failed`. La CI s'arrête à la première étape en échec (prettier avait masqué clippy). Un push annule le run en cours du même `main` (concurrency).
- **Logs Linux** : `~/.local/share/com.m0s.better-ip-tv/logs/better-ip-tv.log` ; **DB** : `~/.local/share/com.m0s.better-ip-tv/better-ip-tv.db`.
- **Windows (KEVIN-MSI)** : toolchain installée via winget le 2026-11-21 (Rust stable MSVC, Node LTS) ; un shell déjà ouvert doit ajouter `%USERPROFILE%\.cargo\bin` et `C:\Program Files\nodejs` au PATH. Le checkout est en CRLF (`core.autocrlf=true`) : `npm run format:check` signale alors tous les fichiers, ce qui est sans rapport avec le code. Extraire l'archive MPV `.7z` demande 7-Zip (`tar` et `py7zr` ne gèrent pas le filtre BCJ2).
- Les `release.yml` et `test.yml` **à la racine** sont d'anciennes copies inactives ; les workflows actifs sont dans `.github/workflows/`.
