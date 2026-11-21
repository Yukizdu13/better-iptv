# CLAUDE.md — better-iptv (fork Yukizdu13)

> **Ce fichier doit TOUJOURS être tenu à jour.** Toute modification qui change la structure du repo, les commandes, les plateformes, le flux git/release ou une règle ci-dessous impose de mettre à jour ce fichier **dans le même commit**. Lire ce fichier = le corriger s'il est périmé.
> Fichier compagnon, à tenir à jour avec lui : [`docs/ARBORESCENCE_PLATEFORMES.md`](docs/ARBORESCENCE_PLATEFORMES.md) (fichiers nécessaires par plateforme).
> Dernière mise à jour : 2026-11-21 (version de l'app : 2.6.1).

Les règles universelles de l'utilisateur sont chargées via son `~/.claude/CLAUDE.md` (hors repo) ; ce fichier ne contient que ce qui est propre au projet.

## Projet

Lecteur IPTV multiplateforme : UI web (React/TypeScript) dans Tauri 2, backend Rust, SQLite, lecture vidéo via **MPV** (processus externe) sur desktop, `<video>` HTML5 sur iOS.
Playlists M3U et Xtream Codes (live, VOD, séries, EPG XMLTV), profils, favoris, contrôle parental (PIN Argon2), play aléatoire, reprise des séries.

- Produit : « Better IPTV+Kev » — identifiant `com.m0s.better-ip-tv` — crate/paquet npm `better-ip-tv`
- `origin` = `mewset/better-iptv` (upstream, **on n'y pousse jamais**) ; `fork` = `Yukizdu13/better-iptv` (**on pousse ici** : `git push fork main`)
- Backlog : `prochaine_maj.json` à la racine (les issues GitHub sont désactivées sur le fork). Y noter toute nouvelle idée/bug avec les champs `distributions`, `statut` (Linux), `statut_windows` (`fait` | `a_faire` | `a_valider`) et, si besoin, `notes_windows` ; les décisions d'une entrée vont dans `decisions` (cf. #20, compte universel).

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
                      playlist/ (parser M3U + xtream.rs), epg/, *_domain/, state.rs, lib.rs
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
| Windows 11 | premier build à venir, rien de validé | KEVIN-MSI |
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

- **Formats Xtream** : les fournisseurs renvoient des types variables (`season_number` int/string, `episodes` en liste de listes quand une saison 0 existe). Toujours désérialiser de façon tolérante (`de_string_or_int`, `de_episodes` dans `playlist/xtream.rs`) plutôt que du code par fournisseur.
- **Données sensibles** : la base SQLite de l'utilisateur contient les identifiants Xtream **en clair** → ne jamais les afficher, logger, ni les mettre dans un fichier ou une mémoire. Les URL sont masquées dans les logs (`utils::mask_credentials`).
- **Linux/Wayland** : `WEBKIT_DISABLE_DMABUF_RENDERER=1` est forcé (`lib.rs` et `src-tauri/templates/better-iptv.desktop`) — ne pas retirer.
- **Menu KDE « Better IPTV+Kev »** = binaire **installé** (`/usr/bin/better-ip-tv`, paquet `better-iptv-kev`), pas le working tree : pour tester du code neuf, `npm run tauri dev` ou rebuild + `sudo dpkg -i` du `.deb`.
- **Node.js absent sur kevin-kubuntu** (purgé) : `npm`, le build frontend et le `.deb` se font depuis KEVIN-MSI ou un Node installé à la demande. `cargo` est disponible ici. Pour rejouer prettier/eslint/vitest sans Node système : Node 20 portable (tarball officiel nodejs.org, checksum vérifié via `SHASUMS256.txt`) décompressé dans un dossier temporaire et lancé directement (`node node_modules/prettier/bin/prettier.cjs --check src`) ; l'utilisateur doit l'autoriser.
- **CI ≠ machine locale** : la CI (`Test Build`) utilise Rust `stable` (1.100 le 2026-11-21) alors que kevin-kubuntu a 1.96 : un nouveau lint clippy peut casser la CI sans changement de code (cas réel : `needless_bool`, `unnecessary_sort_by`). Si tout passe en local mais pas en CI : `gh run view <id> --repo Yukizdu13/better-iptv --log-failed`. La CI s'arrête à la première étape en échec (prettier avait masqué clippy). Un push annule le run en cours du même `main` (concurrency).
- **Logs Linux** : `~/.local/share/com.m0s.better-ip-tv/logs/better-ip-tv.log` ; **DB** : `~/.local/share/com.m0s.better-ip-tv/better-ip-tv.db`.
- Les `release.yml` et `test.yml` **à la racine** sont d'anciennes copies inactives ; les workflows actifs sont dans `.github/workflows/`.
