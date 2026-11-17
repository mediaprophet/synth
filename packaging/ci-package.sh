#!/usr/bin/env bash
# Build a platform installer next to the signed portable binary.
# Usage: packaging/ci-package.sh windows|macos|linux VERSION ARTIFACT
set -euo pipefail
OS="${1:?os}"
VERSION="${2:?version}"
ARTIFACT="${3:?artifact}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
VERSION="${VERSION#v}"
DIST="$ROOT/dist"
BIN="$DIST/$ARTIFACT"
ICON="$ROOT/assets/branding/icon.ico"
PNG256="$ROOT/assets/branding/icon-256.png"
test -f "$BIN"

vi_version="$VERSION"
# NSIS VIProductVersion needs exactly four integers.
dots="${vi_version//[^.]}"; dots="${#dots}"
if [ "$dots" -eq 0 ]; then
  vi_version="${vi_version}.0.0.0"
elif [ "$dots" -eq 1 ]; then
  vi_version="${vi_version}.0.0"
elif [ "$dots" -eq 2 ]; then
  vi_version="${vi_version}.0"
fi

case "$OS" in
  windows)
    installer="WebizenAI-Synth-Setup-windows-x86_64.exe"
    MAKENSIS="$(command -v makensis || true)"
    if [ -z "$MAKENSIS" ]; then
      for candidate in \
        "/c/Program Files (x86)/NSIS/makensis.exe" \
        "/c/Program Files/NSIS/makensis.exe" \
        "/mingw64/bin/makensis"
      do
        if [ -x "$candidate" ]; then MAKENSIS="$candidate"; break; fi
      done
    fi
    test -n "$MAKENSIS"
    "$MAKENSIS" \
      -DSYNTH_VERSION="$VERSION" \
      -DSYNTH_VI_VERSION="$vi_version" \
      -DSYNTH_BIN="dist\\$ARTIFACT" \
      -DSYNTH_OUT="dist\\$installer" \
      -DSYNTH_ICON="assets\\branding\\icon.ico" \
      -DSYNTH_SIDEBAR="assets\\branding\\installer-sidebar.bmp" \
      -DSYNTH_HEADER="assets\\branding\\installer-header.bmp" \
      "packaging\\windows\\synth.nsi"
    echo "$installer"
    ;;
  macos)
    installer="WebizenAI-Synth-macos-aarch64.dmg"
    app="$DIST/Webizen AI Synth.app"
    rm -rf "$app"
    mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
    cp "$BIN" "$app/Contents/MacOS/synth"
    chmod +x "$app/Contents/MacOS/synth"
    sed "s/SYNTH_VERSION/$VERSION/g" "$ROOT/packaging/macos/Info.plist" > "$app/Contents/Info.plist"
    iconset="$DIST/AppIcon.iconset"
    rm -rf "$iconset"
    mkdir -p "$iconset"
    sips -z 16 16 "$PNG256" --out "$iconset/icon_16x16.png" >/dev/null
    sips -z 32 32 "$PNG256" --out "$iconset/icon_16x16@2x.png" >/dev/null
    sips -z 32 32 "$PNG256" --out "$iconset/icon_32x32.png" >/dev/null
    sips -z 64 64 "$PNG256" --out "$iconset/icon_32x32@2x.png" >/dev/null
    sips -z 128 128 "$PNG256" --out "$iconset/icon_128x128.png" >/dev/null
    sips -z 256 256 "$PNG256" --out "$iconset/icon_128x128@2x.png" >/dev/null
    sips -z 256 256 "$PNG256" --out "$iconset/icon_256x256.png" >/dev/null
    sips -z 512 512 "$PNG256" --out "$iconset/icon_256x256@2x.png" >/dev/null
    iconutil -c icns "$iconset" -o "$app/Contents/Resources/AppIcon.icns"
    rm -rf "$iconset"
    rm -f "$DIST/$installer"
    hdiutil create -volname "Webizen AI Synth" -srcfolder "$app" -ov -format UDZO "$DIST/$installer"
    echo "$installer"
    ;;
  linux)
    installer="webizen-ai-synth_${VERSION}_amd64.deb"
    stage="$DIST/deb"
    rm -rf "$stage"
    mkdir -p "$stage/DEBIAN" "$stage/usr/bin" \
      "$stage/usr/share/applications" \
      "$stage/usr/share/icons/hicolor/256x256/apps"
    cp "$BIN" "$stage/usr/bin/synth"
    chmod 755 "$stage/usr/bin/synth"
    cp "$ROOT/packaging/linux/synth.desktop" "$stage/usr/share/applications/webizen-ai-synth.desktop"
    cp "$PNG256" "$stage/usr/share/icons/hicolor/256x256/apps/webizen-ai-synth.png"
    sed "s/SYNTH_VERSION/$VERSION/g" "$ROOT/packaging/linux/control" > "$stage/DEBIAN/control"
    echo "Installed-Size: $(du -sk "$stage/usr" | cut -f1)" >> "$stage/DEBIAN/control"
    dpkg-deb --root-owner-group --build "$stage" "$DIST/$installer"
    echo "$installer"
    ;;
  *)
    echo "unknown os $OS" >&2
    exit 1
    ;;
esac
