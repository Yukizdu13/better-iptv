# 📋 Arbetsflöde: Manuell Versionsuppdatering

## Översikt

Detta arbetsflöde beskriver processen för att manuellt uppdatera versionen i Better IPTV-projektet, inklusive uppdatering av båda changelog-filer.

---

## 🔧 Befintliga Verktyg

| Script | Syfte | Kommando |
|--------|-------|----------|
| `release.sh` | Interaktivt release-skapande | `npm run release` |
| `sync-version.cjs` | Synkronisera version till Cargo.toml & tauri.conf.json | `npm run version:sync` |
| `update-aur.sh` | Uppdatera AUR-paketet | `npm run aur:update` |

---

## 📑 Filer som Behöver Uppdateras

### Automatiskt (via scripts)
```
package.json           → Källan till sanningen
src-tauri/Cargo.toml   → Synkas via sync-version.cjs
src-tauri/tauri.conf.json → Synkas via sync-version.cjs
```

### Manuellt
```
CHANGELOG.md           → Teknisk changelog (utvecklare)
CHANGELOG_USER.md      → Användarvänlig changelog (slutanvändare)
VERSION.md             → Version History sektion (valfritt)
```

---

## 🚀 Komplett Arbetsflöde

### Steg 1: Förberedelse
```bash
# Kontrollera git-status
git status && git branch

# Säkerställ att du är på rätt branch
git checkout main  # eller feature-branch
```

### Steg 2: Välj Versionstyp

| Typ | När | Exempel |
|-----|-----|---------|
| **PATCH** | Buggfixar, små korrigeringar | 2.2.0 → 2.2.1 |
| **MINOR** | Nya funktioner (bakåtkompatibla) | 2.2.0 → 2.3.0 |
| **MAJOR** | Stora ändringar, API-brytande | 2.2.0 → 3.0.0 |

### Steg 3: Uppdatera Version

**Alternativ A - Interaktivt (rekommenderat):**
```bash
npm run release
```
> Följ de interaktiva promptarna. Scriptet hanterar version bump, sync och git tag.

**Alternativ B - Manuellt:**
```bash
# Bumpa version i package.json
npm version patch --no-git-tag-version  # eller minor/major

# Synka till Rust-filer
npm run version:sync
```

### Steg 4: Uppdatera CHANGELOG.md (Teknisk)

**Format:**
```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- **Feature Name** - Brief description
  - Technical detail 1
  - Technical detail 2

### Changed
- Description of changed behavior

### Fixed
- **Bug Name** - What was fixed

### Technical Debt
- Internal improvements

[X.Y.Z]: https://github.com/mewset/better-ip-tv/compare/vPREV...vX.Y.Z
```

**Innehåll att inkludera:**
- Alla kodfiler som ändrats (med sökvägar)
- Nya funktioner, metoder, databastabeller
- API-ändringar
- Tester som lagts till
- Prestandaoptimeringar

### Steg 5: Uppdatera CHANGELOG_USER.md (Användarvänlig)

**Format:**
```markdown
## Version X.Y.Z (Month DD, YYYY)

### New Features

**Feature Name**
- What it does in simple terms
- How users benefit from it

### Improvements
- Better [something]
- Faster [something]

### Fixes
- Fixed [issue] that caused [problem]
```

**Riktlinjer:**
- Skriv för icke-tekniska användare
- Förklara NYTTan, inte implementationen
- Undvik teknisk jargong
- Använd svenska om det är konsekvent

### Steg 6: Skapa Commit & Tag

**Om du använde `npm run release`:**
> Scriptet skapade redan commit och tag. Hoppa till Steg 7.

**Manuellt:**
```bash
# Lägg till alla ändringar
git add package.json package-lock.json \
        src-tauri/Cargo.toml src-tauri/tauri.conf.json \
        src-tauri/Cargo.lock CHANGELOG.md CHANGELOG_USER.md

# Skapa commit
git commit -m "chore: bump version to X.Y.Z

- Add feature A
- Fix issue B
- Improve C"

# Skapa annoterad tag
git tag -a "vX.Y.Z" -m "Release X.Y.Z

What's new:
- Feature A
- Fix B"
```

### Steg 7: Push till GitHub
```bash
git push && git push --tags
```

### Steg 8: Uppdatera AUR (om Linux)
```bash
# Vänta på att GitHub Actions bygger klart
# Gå till: https://github.com/mewset/better-ip-tv/actions

# När release publicerad:
npm run aur:update
```

---

## 📊 Visuellt Flödesschema

```
┌─────────────────────────────────────────────────────────────────┐
│                    VERSION UPDATE WORKFLOW                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  1. FÖRBEREDELSE                                                 │
│  ─────────────────                                               │
│  • git status && git branch                                      │
│  • Säkerställ ren working directory                              │
│  • Bestäm versionstyp (PATCH/MINOR/MAJOR)                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  2. VERSION BUMP                                                 │
│  ───────────────                                                 │
│  npm run release    ← Interaktivt (rekommenderat)                │
│       ELLER                                                      │
│  npm version [type] --no-git-tag-version                         │
│  npm run version:sync                                            │
│                                                                  │
│  Uppdaterar automatiskt:                                         │
│  ✓ package.json                                                  │
│  ✓ src-tauri/Cargo.toml                                          │
│  ✓ src-tauri/tauri.conf.json                                     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  3. UPPDATERA CHANGELOGS (MANUELLT)                              │
│  ──────────────────────────────────                              │
│                                                                  │
│  CHANGELOG.md (Teknisk)          CHANGELOG_USER.md (Användare)   │
│  ├── [X.Y.Z] - YYYY-MM-DD        ├── Version X.Y.Z (Date)        │
│  ├── ### Added                   ├── ### New Features            │
│  │   └── Tekniska detaljer       │   └── Enkel beskrivning       │
│  ├── ### Changed                 ├── ### Improvements            │
│  ├── ### Fixed                   ├── ### Fixes                   │
│  └── ### Technical Debt          └── (Skip tekniska detaljer)    │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  4. GIT COMMIT & TAG                                             │
│  ───────────────────                                             │
│  git add [files]                                                 │
│  git commit -m "chore: bump version to X.Y.Z"                    │
│  git tag -a "vX.Y.Z" -m "Release notes..."                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  5. PUSH & DEPLOY                                                │
│  ────────────────                                                │
│  git push && git push --tags                                     │
│                                                                  │
│  → GitHub Actions bygger automatiskt:                            │
│    • Linux (AppImage, .deb, .rpm)                                │
│    • Windows (.msi, .exe)                                        │
│    • macOS (.dmg)                                                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  6. POST-RELEASE (Valfritt)                                      │
│  ──────────────────────────                                      │
│  • Publicera draft release på GitHub                             │
│  • npm run aur:update  (för Arch Linux)                          │
│  • Meddela användare om uppdateringen                            │
└─────────────────────────────────────────────────────────────────┘
```

---

## ✅ Checklista

```
□ git status är ren
□ Korrekt branch
□ Version bumpas (package.json)
□ Version synkad (Cargo.toml, tauri.conf.json)
□ CHANGELOG.md uppdaterad med tekniska detaljer
□ CHANGELOG_USER.md uppdaterad med användarvänlig text
□ Git commit skapad
□ Git tag skapad
□ Pushat till remote
□ GitHub Actions build lyckas
□ Release publicerad på GitHub
□ AUR uppdaterat (om Linux)
```

---

## 🎯 Snabbreferens

| Uppgift | Kommando |
|---------|----------|
| Full release (interaktiv) | `npm run release` |
| Endast bumpa patch | `npm run version:patch` |
| Endast bumpa minor | `npm run version:minor` |
| Endast bumpa major | `npm run version:major` |
| Synka version till Rust | `npm run version:sync` |
| Uppdatera AUR | `npm run aur:update` |
| Visa commits sedan senaste tag | `git log $(git describe --tags --abbrev=0)..HEAD --oneline` |
