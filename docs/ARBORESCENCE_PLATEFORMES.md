# Arborescence des fichiers nécessaires par plateforme

> **Ce fichier doit TOUJOURS être tenu à jour.** Dès qu'un fichier propre à une plateforme est ajouté, déplacé, renommé ou supprimé (build, bundle, CI, ressource, config native), ou qu'une dépendance système change, mettre à jour ce fichier **dans le même commit**. À relire avec [`CLAUDE.md`](../CLAUDE.md) (racine), à tenir à jour avec lui.
> Dernière mise à jour : 2026-11-21 (version de l'app : 2.6.1 ; compte universel étape 1). Chemins relatifs à la racine du repo.

## 1. Socle commun (toutes plateformes)

```
package.json, package-lock.json        Dépendances/scripts npm — SOURCE DE VÉRITÉ de la version
index.html, vite.config.ts, vitest.config.ts
tsconfig.json, tsconfig.node.json
tailwind.config.js, postcss.config.js
eslint.config.js, .prettierrc
src/                                   Frontend React/TS (App.tsx, main.tsx, components/, hooks/, lib/, stores/, types/)
public/                                Assets statiques Vite
src-tauri/Cargo.toml, Cargo.lock       Dépendances Rust (version à synchroniser à la main avec package.json)
src-tauri/build.rs, clippy.toml
src-tauri/tauri.conf.json              Config Tauri (version, bundle, CSP, ressources) — version à synchroniser
src-tauri/capabilities/default.json    Permissions desktop (core, opener)
src-tauri/icons/                       32x32, 128x128, @2x, icon.icns (macOS), icon.ico (Windows), icon.png, Square*Logo.png
src-tauri/src/                         Backend Rust (voir CLAUDE.md > Structure)
src-tauri/src/sync_domain/             Compte universel (#20) : crypto.rs, snapshot.rs, merge.rs (même code Linux/Windows)
src-tauri/src/commands/account.rs      Commandes export_account / import_account / default_account_file_path
src/components/settings/AccountTab.tsx Onglet Paramètres > Compte (masqué sur iOS)
prochaine_maj.json                     Backlog (champs distributions / statut / statut_windows)
.github/workflows/test.yml             CI tests (Ubuntu 24.04) : lint, prettier, vitest, clippy, cargo test, build
```

Dépendances Rust ajoutées pour le compte universel : `chacha20poly1305`, `zeroize` (+ `argon2`, `rand` déjà présents). Aucune permission Tauri ajoutée : pas de plugin dialog, le chemin du fichier se saisit à la main (défaut : dossier Documents).

Générés, **non versionnés** : `node_modules/`, `dist/` (frontend compilé), `src-tauri/target/`, `src-tauri/gen/schemas/`.

## 2. Linux (référence — Kubuntu 24.04)

Cibles Tauri : `deb`, `appimage`, `rpm` (`bundle.targets = "all"`).

```
src-tauri/templates/better-iptv.desktop   Lanceur .desktop (deb + rpm) ; force WEBKIT_DISABLE_DMABUF_RENDERER=1
src-tauri/tauri.conf.json > bundle.linux  deb/rpm desktopTemplate, appimage bundleMediaFramework=false
src-tauri/src/lib.rs                      #[cfg(target_os = "linux")] : variable WebKit DMABUF
src-tauri/src/playback/mpv.rs             MPV système (`mpv` dans le PATH) + socket IPC dans le dossier temp
.github/workflows/release.yml             Job ubuntu-24.04 : AppImage + variante `-arch.AppImage` (libs WebKit retirées)
.SRCINFO                                  AUR (vide dans le repo ; paquet `aur/` et `dev-scripts/` gitignorés)
```

- **Dépendances de build** : `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf` (+ `libfuse2` pour l'AppImage), Node 20+, Rust stable.
- **Dépendances d'exécution** : `libwebkit2gtk-4.1-0 libgtk-3-0` et **`mpv` installé sur le système** (non embarqué).
- **Build local** : `npm run tauri build -- --bundles deb` → `src-tauri/target/release/bundle/deb/`.
- **Données** : `~/.local/share/com.m0s.better-ip-tv/` (base `better-ip-tv.db`, `logs/`).

## 3. Windows (KEVIN-MSI — compilation Rust validée, lecture MPV à valider)

Cibles Tauri : `msi` et `nsis` (`bundle.targets = "all"`).

```
resources/mpv/                            DOIT contenir mpv.exe + ses DLL au moment du build (seul .gitkeep est versionné)
resources/.gitignore                      Ignore mpv/* sauf .gitkeep et .gitignore
src-tauri/tauri.conf.json > bundle.resources   ["../resources/mpv/*"] → embarque MPV dans l'installeur
src-tauri/src/playback/mpv.rs             get_mpv_path() : <dossier de l'exe>/_up_/resources/mpv/mpv.exe (emplacement réel de l'installeur), puis <exe>/mpv/mpv.exe, sinon mpv.exe du PATH ; IPC par named pipe \\.\pipe\better-iptv-mpv-<pid>
.github/workflows/release.yml             Job windows-latest : télécharge mpv (shinchiro/mpv-winbuild-cmake) dans resources/mpv avant le build
```

- **Prérequis build** : Node 20+, Rust stable (toolchain MSVC + Visual Studio Build Tools 2022), WebView2 (présent sur Windows 11), `npm ci`. Installés sur KEVIN-MSI le 2026-11-21 (`winget install Rustlang.Rustup OpenJS.NodeJS.LTS`) ; `cargo test` : 108 tests OK.
- **MPV en local** : récupérer `mpv-x86_64-*.7z` (shinchiro), extraire dans `resources/mpv/` (reste ignoré par git), ou installer mpv dans le PATH. L'extraction exige 7-Zip (filtre BCJ2 : `tar`/`py7zr` échouent).
- **Données** : `%APPDATA%\com.m0s.better-ip-tv\` (`logs\`, base SQLite).
- **Premier build (2026-11-21)** : `npm run tauri build -- --bundles nsis` produit `src-tauri/target/release/bundle/nsis/Better IPTV+Kev_<version>_x64-setup.exe` (≈ 40 Mo, MPV embarqué) ; le code IPC par named pipe compile. (b) résolu : Tauri remplace `..` par `_up_`, MPV est installé dans `<exe>/_up_/resources/mpv/` (corrigé dans `get_mpv_path()`). **Reste à valider** : l'IPC MPV par named pipe en conditions réelles (pause, seek, reprise) ; le MPV embarqué lit bien les flux (testé en ligne de commande : live h264, film mkv hevc, épisode mkv).

## 4. macOS (non testé sur le fork)

Cibles Tauri : `app`, `dmg`.

```
src-tauri/icons/icon.icns                 Icône du bundle
.github/workflows/release.yml             Job macos-latest : .dmg / .app
src-tauri/src/playback/mpv.rs             MPV système (`mpv`, ex. `brew install mpv`), socket IPC Unix
```

- **Données** : `~/Library/Application Support/com.m0s.better-ip-tv/`.
- Pas de signature ni de notarisation configurées.

## 5. iOS (sideload, `.ipa` non signé via GitHub Actions)

Pas de MPV : lecture par `<video>` HTML5 (WKWebView), code Rust MPV exclu par `#[cfg(not(target_os = "ios"))]`.

```
.github/workflows/ios.yml                 macos-latest : tauri ios init → patch Info.plist/pbxproj → tauri ios build → .ipa
scripts/ios-setup-macos.sh                Ajoute NSAppTransportSecurity (flux HTTP/RTSP) à Info.plist après `tauri ios init`
docs/iOS_BUILD.md                         Procédure complète, erreurs rencontrées, installation SideLoadly/SideStore
src-tauri/tauri.conf.json > bundle.iOS    minimumSystemVersion 16.0, developmentTeam "PLACEHOLDER" (obligatoire, ne pas retirer)
src-tauri/capabilities/mobile.json        Permissions mobile (core:default)
src-tauri/src/commands/playback.rs        Variantes #[cfg(target_os = "ios")] (lecture déléguée au frontend)
src-tauri/src/commands/series.rs, state.rs, playback/mod.rs   Exclusions/variantes iOS
src/lib/platform.ts                       isIOS() (via la commande Tauri get_platform)
src/components/IOSVideoPlayer.tsx         Lecteur <video> iOS
```

Compte universel : onglet Compte masqué sur iOS (pas de sélecteur de fichier ; hors première version).

Générés par la CI, **non versionnés** : `src-tauri/gen/apple/` (projet Xcode, recréé par `tauri ios init`).
Build local impossible sans Xcode complet : passer par GitHub Actions. Identifiant : `com.m0s.better-ip-tv`.

## 6. Android

Non pris en charge : aucun `src-tauri/gen/android`, aucun workflow. Seuls vestiges : `src-tauri/capabilities/mobile.json` (cite `android`) et `get_platform` qui renvoie `"android"`.

## 7. Fichiers orphelins / à ne pas confondre

- `release.yml`, `test.yml` **à la racine** : anciennes copies inactives (GitHub ne lit que `.github/workflows/`).
- `dev-scripts/` et `aur/` : référencés par `package.json` (`release`, `version:sync`, `aur:update`, `downloads`) mais gitignorés et absents → ces scripts npm ne fonctionnent pas en l'état.
- `VERSION.md`, `CLAUDE.md` figurent dans le `.gitignore` upstream ; `CLAUDE.md` est suivi sur ce fork (ajouté avec `git add -f`).
