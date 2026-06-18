#!/usr/bin/env bash
# À exécuter UNE FOIS sur macOS après "npm run tauri ios init"
# Ajoute NSAppTransportSecurity dans Info.plist pour autoriser les flux HTTP (IPTV)
set -e

PLIST=$(find src-tauri/gen/apple -name "Info.plist" ! -path "*/Pods/*" ! -path "*/build/*" | head -1)

if [ -z "$PLIST" ] || [ ! -f "$PLIST" ]; then
  echo "Erreur : Info.plist introuvable dans src-tauri/gen/apple/. Lance d'abord : npm run tauri ios init"
  exit 1
fi

# Ajouter NSAppTransportSecurity si absent
if ! grep -q "NSAppTransportSecurity" "$PLIST"; then
  /usr/libexec/PlistBuddy -c "Add :NSAppTransportSecurity dict" "$PLIST"
  /usr/libexec/PlistBuddy -c "Add :NSAppTransportSecurity:NSAllowsArbitraryLoads bool true" "$PLIST"
  echo "NSAppTransportSecurity ajouté (flux HTTP/RTSP autorisés)"
else
  echo "NSAppTransportSecurity déjà présent"
fi

echo "Info.plist OK : $PLIST"
