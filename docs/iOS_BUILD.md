# Better IPTV+Kev — Build iOS

## Vue d'ensemble

L'app est installée sur iPhone via **GitHub Actions → .ipa non signé → SideLoadly/SideStore**. Aucun compte Apple Developer payant requis. La signature est faite localement par SideLoadly avec l'Apple ID gratuit.

---

## Architecture de la solution

```
GitHub Actions (macos-latest, Xcode 16.4)
  └── npm run tauri ios build
        ├── cargo build --target aarch64-apple-ios --release
        ├── xcodebuild archive (sans signature)
        └── .xcarchive → BetterIPTV-unsigned.ipa

iPhone
  └── SideLoadly (Mac) → signe avec Apple ID gratuit → installe
  └── SideStore (iPhone) → se re-signe automatiquement toutes les 7 jours
```

---

## Stack technique

- **Framework** : Tauri 2 + React/TypeScript + Rust + SQLite
- **Lecture vidéo desktop** : MPV (processus externe)
- **Lecture vidéo iOS** : `<video>` HTML5 natif dans WKWebView (HLS supporté nativement)
- **Séparation plateforme** : `#[cfg(target_os = "ios")]` côté Rust, `isIOS()` côté frontend
- **Flux HTTP/RTSP** : autorisés via `NSAllowsArbitraryLoads` dans `Info.plist`
- **Bundle ID** : `com.m0s.better-ip-tv`
- **iOS minimum** : 16.0

---

## Workflow GitHub Actions (`.github/workflows/ios.yml`)

### Ce qui fonctionne

Le workflow tourne sur `macos-latest` (macOS 15 arm64, Xcode 16.4).

Étapes validées dans l'ordre :
1. Checkout du repo
2. Setup Node.js 20
3. Setup Rust stable + cible `aarch64-apple-ios`
4. Cache Cargo
5. Installation CocoaPods (`sudo gem install cocoapods`)
6. `npm ci`
7. `npm run tauri ios init`
8. Patch `Info.plist` (script `scripts/ios-setup-macos.sh`)
9. Patch `project.pbxproj` (désactiver signature Xcode)
10. `npm run tauri ios build -- --target aarch64`
11. Extraction du `.app` depuis le `.xcarchive`
12. Packaging en `.ipa`
13. Upload en artifact GitHub Actions (30 jours de rétention)

### Points critiques — erreurs rencontrées et fixes

#### 1. `apple.development-team is empty`

**Symptôme** : `tauri ios init` et `tauri ios xcode-script` échouent avec ce message.

**Cause** : Tauri refuse de générer la config Apple sans un `developmentTeam` non vide. La variable d'env `APPLE_DEVELOPMENT_TEAM` n'est **pas** transmise aux scripts de build Xcode quand `xcodebuild` les appelle en sous-processus.

**Fix** : Mettre `"developmentTeam": "PLACEHOLDER"` directement dans `src-tauri/tauri.conf.json`. Pas de variable d'env — elle ne sera pas visible par les scripts de build phase Xcode.

```json
"iOS": {
  "minimumSystemVersion": "16.0",
  "developmentTeam": "PLACEHOLDER"
}
```

`PLACEHOLDER` n'est pas un vrai Team ID Apple. Xcode génère le projet avec cette valeur mais le build fonctionne car on désactive la signature dans l'étape suivante.

---

#### 2. `tauri ios xcode-script` panique — WebSocket introuvable

**Symptôme** :
```
failed to read missing addr file /var/folders/.../T/com.m0s.better-ip-tv-server-addr
```
puis après workaround :
```
failed to build WebSocket client: failed to lookup address information
```

**Cause** : Le script de build Xcode `tauri ios xcode-script` (qui compile le Rust) n'est pas un binaire autonome — il doit communiquer via WebSocket avec le processus `tauri ios build` parent. Appeler `xcodebuild` directement sans `tauri ios build` rompt ce lien.

**Workarounds échoués** :
- Créer manuellement le fichier addr (`echo "http://127.0.0.1:1420" > $TMPDIR/com.m0s.better-ip-tv-server-addr`) → résout la première panique mais pas la connexion WebSocket
- Appeler `xcodebuild` directement avec `CODE_SIGNING_REQUIRED=NO` → xcodebuild démarre mais xcode-script ne peut pas se connecter → `Abort trap: 6`

**Fix** : Toujours utiliser `npm run tauri ios build` — jamais `xcodebuild` directement. C'est la seule façon de lancer le serveur WebSocket que le xcode-script attend.

---

#### 3. `No profiles for 'com.m0s.better-ip-tv' were found`

**Symptôme** : `xcodebuild archive` échoue, Xcode cherche un profil de provisioning automatique.

**Cause** : Le projet Xcode généré par `tauri ios init` a le code signing en mode automatique, ce qui requiert un vrai compte développeur Apple.

**Fix** : Patcher `project.pbxproj` après `tauri ios init` pour désactiver la signature dans tous les `buildSettings` :

```python
# Extrait du step "Désactiver signature Xcode dans project.pbxproj"
content = re.sub(r'DEVELOPMENT_TEAM = \w+;', 'DEVELOPMENT_TEAM = "";', content)
content = re.sub(r'CODE_SIGN_IDENTITY = "[^"]*";', 'CODE_SIGN_IDENTITY = "";', content)
content = content.replace('buildSettings = {',
    'buildSettings = {\n\t\t\t\tCODE_SIGNING_REQUIRED = NO;\n\t\t\t\tCODE_SIGNING_ALLOWED = NO;')
```

Ce patch doit être fait **après** `tauri ios init` (qui génère le projet) et **avant** `tauri ios build` (qui compile). Si `tauri ios init` est relancé, le patch doit être réappliqué.

---

#### 4. `xcrun simctl list runtimes` — exit code 72

**Symptôme** : `tauri ios build` échoue immédiatement avec ce message en local (Mac sans Xcode complet).

**Cause** : `tauri ios build` interroge `simctl` pour lister les simulateurs. Avec seulement les Xcode Command Line Tools (pas le Xcode.app complet), `simctl` retourne 72.

**Contournement** : `--target aarch64` dans la commande réduit la portée mais ne supprime pas totalement la vérification.

**Solution** : Le build local n'est **pas utilisable** sans Xcode complet (~15 GB). Utiliser GitHub Actions exclusivement pour le build iOS.

---

#### 5. `setup-rust-toolchain` — flag `targets` invalide

**Symptôme** : Warning dans les logs CI, la cible iOS n'est pas ajoutée correctement.

**Fix** : Utiliser `target` (singulier) et non `targets` dans l'action :
```yaml
- uses: actions-rust-lang/setup-rust-toolchain@v1
  with:
    target: aarch64-apple-ios  # pas "targets"
```

---

#### 6. `scripts/ios-setup-macos.sh` — chemin Info.plist hardcodé

**Symptôme** : Le script échoue car il cherche `src-tauri/gen/apple/better-ip-tv/Info.plist` mais Tauri génère le dossier depuis `productName`.

**Fix** : Remplacer le chemin hardcodé par un `find` dynamique :
```bash
PLIST=$(find src-tauri/gen/apple -name "Info.plist" ! -path "*/Pods/*" ! -path "*/build/*" | head -1)
```

---

## Prérequis pour le build local (Mac)

> ⚠️ Le build local est limité — voir section "Ce qui ne fonctionne pas".

| Outil | Installation | Notes |
|---|---|---|
| Xcode CLT | `sudo softwareupdate --install` | Insuffisant seul pour `tauri ios build` |
| Xcode complet | Mac App Store | ~15 GB, nécessaire pour `simctl` |
| Homebrew | Script officiel | Nécessaire pour Node, CocoaPods Ruby |
| Node.js | `brew install node` | |
| Rust | `curl sh.rustup.rs \| sh` | Puis `rustup target add aarch64-apple-ios` |
| CocoaPods | `sudo gem install cocoapods` via Ruby 4 (`brew install ruby`) | Ruby système 2.6 trop vieux |
| gh CLI | `brew install gh` | Pour télécharger les artifacts CI |

---

## Installation sur iPhone (sans compte développeur payant)

### Outils requis sur le Mac

| Outil | Rôle | Lien |
|---|---|---|
| **SideLoadly** | Signe et installe le `.ipa` avec Apple ID gratuit | sideloadly.io |
| **iLoader** | Génère et place le pairing file dans SideStore | github.com/nab138/iloader |
| **SideStore** | App iPhone : re-signe automatiquement toutes les 7 jours | SideStore.ipa via iLoader |

### Procédure initiale (une seule fois)

1. **Télécharger SideStore.ipa** :
   ```bash
   curl -L -o ~/Downloads/SideStore.ipa \
     "https://github.com/SideStore/SideStore/releases/latest/download/SideStore.ipa"
   ```

2. **Installer SideStore via SideLoadly** :
   - Ouvrir SideLoadly → glisser `SideStore.ipa` → entrer Apple ID → installer

3. **Sur l'iPhone** : Paramètres → Général → VPN et gestion des appareils → ton Apple ID → **Faire confiance**

4. **Activer le mode développeur** : Paramètres → Confidentialité et sécurité → Mode développeur → Activer (reboot requis)

5. **Générer le pairing file avec iLoader** :
   - iPhone branché en USB
   - `idevicepair pair` (vérifie que libimobiledevice est installé)
   - Ouvrir iLoader → **Pair in all apps**

6. **Installer Better IPTV via SideLoadly** :
   - Télécharger l'artifact depuis GitHub Actions
   - Glisser `BetterIPTV-unsigned.ipa` dans SideLoadly → Apple ID → installer

### Re-signature (toutes les 7 jours)

Avec SideStore installé : ouvrir SideStore sur l'iPhone → **Refresh All**. Pas besoin du Mac, pas besoin d'être branché en USB.

Sans SideStore : rebrancher l'iPhone, rouvrir SideLoadly, reglisser le `.ipa`. 30 secondes.

---

## Récupérer le .ipa depuis GitHub Actions

```bash
# Lister les derniers builds
gh run list --repo Yukizdu13/better-iptv --limit 5

# Télécharger l'artifact du dernier build réussi
gh run download <RUN_ID> --repo Yukizdu13/better-iptv --name "BetterIPTV-iOS-<SHA>" --dir ~/Downloads/
```

Ou via l'interface web : github.com/Yukizdu13/better-iptv → onglet **Actions** → dernier build "Build iOS IPA" → section **Artifacts**.

---

## Ce qui ne fonctionne PAS

- **Build local sans Xcode complet** : `tauri ios build` échoue sur `xcrun simctl` (exit 72). Avec Xcode complet ça devrait fonctionner mais non testé.
- **AltServer** : bloqué par Gatekeeper sur macOS Sonoma même après `xattr -cr` et re-signature ad-hoc. AltServer n'est plus nécessaire — SideLoadly le remplace.
- **`xcodebuild` direct** (sans `tauri ios build`) : le script de build Xcode (`tauri ios xcode-script`) nécessite impérativement la connexion WebSocket avec le processus `tauri ios build` parent. Impossible à contourner proprement.
- **Variable d'env `APPLE_DEVELOPMENT_TEAM` dans le workflow CI** : non transmise aux scripts de build phase Xcode. Doit être dans `tauri.conf.json`.

---

## Limitations connues

- L'app se désactive après **7 jours** sans re-signature (contrainte Apple ID gratuit)
- Le pairing file SideStore peut expirer après une mise à jour ou réinitialisation iPhone — relancer iLoader si SideStore refuse de se rafraîchir
- Le `.ipa` généré est non signé — SideLoadly ou SideStore font la signature à l'installation
- `PLACEHOLDER` comme `developmentTeam` dans `tauri.conf.json` est un artifice — sans vraie Team ID, Xcode génère un projet sans signature valide, ce qu'on veut intentionnellement

---

## Historique des décisions

| Décision | Raison |
|---|---|
| GitHub Actions plutôt que build local | `tauri ios build` local nécessite Xcode complet (~15 GB) absent |
| `tauri ios build` plutôt que `xcodebuild` direct | Architecture WebSocket de tauri-cli, impossible à contourner |
| SideLoadly plutôt qu'AltServer | AltServer bloqué par Gatekeeper sur macOS Sonoma |
| `developmentTeam: "PLACEHOLDER"` dans tauri.conf.json | Env var non propagée aux build phase scripts Xcode |
| SideStore installé via SideLoadly | Permet l'auto-refresh des signatures sans Mac |

---

*Dernière mise à jour : 2026-06-19 — Session de mise en place initiale*
