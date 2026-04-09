#!/usr/bin/env bash
#
# Build a macOS .app bundle for Hieroglyphic.
# Requires: brew install gtk4 libadwaita onnxruntime meson desktop-file-utils
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
APP_NAME="Hieroglyphic"
APP_BUNDLE="$PROJECT_ROOT/$APP_NAME.app"
BUILDDIR="$PROJECT_ROOT/builddir"
INSTALLDIR="$PROJECT_ROOT/_macos_install"

# Detect Homebrew prefix
if [ -d "/opt/homebrew" ]; then
    HOMEBREW_PREFIX="/opt/homebrew"
elif [ -d "/usr/local/Cellar" ]; then
    HOMEBREW_PREFIX="/usr/local"
else
    echo "Error: Homebrew not found" >&2
    exit 1
fi

echo "==> Building Hieroglyphic for macOS..."

# --- Build with meson/ninja ---
rm -rf "$BUILDDIR" "$INSTALLDIR"
meson setup "$BUILDDIR" --prefix="$INSTALLDIR" "$PROJECT_ROOT"
ninja -C "$BUILDDIR"
ninja -C "$BUILDDIR" install

echo "==> Creating .app bundle..."

# --- Create .app structure ---
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources/share"

# --- Copy binary ---
cp "$INSTALLDIR/bin/hieroglyphic" "$APP_BUNDLE/Contents/MacOS/hieroglyphic-bin"

# --- Copy resources ---
cp -R "$INSTALLDIR/share/hieroglyphic" "$APP_BUNDLE/Contents/Resources/share/hieroglyphic"
cp -R "$INSTALLDIR/share/glib-2.0"    "$APP_BUNDLE/Contents/Resources/share/glib-2.0"
cp -R "$INSTALLDIR/share/locale"       "$APP_BUNDLE/Contents/Resources/share/locale"

# --- Generate .icns icon ---
echo "==> Generating app icon..."
ICONSET_DIR=$(mktemp -d)/Hieroglyphic.iconset
mkdir -p "$ICONSET_DIR"

SVG="$PROJECT_ROOT/data/icons/io.github.finefindus.Hieroglyphic.svg"
for SIZE in 16 32 128 256 512; do
    rsvg-convert -w "$SIZE" -h "$SIZE" "$SVG" -o "$ICONSET_DIR/icon_${SIZE}x${SIZE}.png"
    DOUBLE=$((SIZE * 2))
    rsvg-convert -w "$DOUBLE" -h "$DOUBLE" "$SVG" -o "$ICONSET_DIR/icon_${SIZE}x${SIZE}@2x.png"
done
iconutil -c icns -o "$APP_BUNDLE/Contents/Resources/hieroglyphic.icns" "$ICONSET_DIR"
rm -rf "$(dirname "$ICONSET_DIR")"

# --- Read version from meson.build ---
VERSION=$(grep "version:" "$PROJECT_ROOT/meson.build" | head -1 | sed "s/.*'\(.*\)'.*/\1/")

# --- Create Info.plist ---
cat > "$APP_BUNDLE/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>Hieroglyphic</string>
  <key>CFBundleDisplayName</key>
  <string>Hieroglyphic</string>
  <key>CFBundleIdentifier</key>
  <string>io.github.finefindus.Hieroglyphic</string>
  <key>CFBundleVersion</key>
  <string>${VERSION}</string>
  <key>CFBundleShortVersionString</key>
  <string>${VERSION}</string>
  <key>CFBundleExecutable</key>
  <string>Hieroglyphic</string>
  <key>CFBundleIconFile</key>
  <string>hieroglyphic</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleSignature</key>
  <string>????</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
</dict>
</plist>
PLIST

# --- Create PkgInfo ---
echo -n "APPL????" > "$APP_BUNDLE/Contents/PkgInfo"

# --- Create launcher script ---
cat > "$APP_BUNDLE/Contents/MacOS/Hieroglyphic" <<'LAUNCHER'
#!/usr/bin/env bash
#
# Launcher for Hieroglyphic.app — sets up GTK/GLib environment
# and runs the actual binary.
#
DIR="$(cd "$(dirname "$0")" && pwd)"
RESOURCES="$(dirname "$DIR")/Resources"

# Detect Homebrew prefix
if [ -d "/opt/homebrew" ]; then
    HOMEBREW_PREFIX="/opt/homebrew"
elif [ -d "/usr/local/Cellar" ]; then
    HOMEBREW_PREFIX="/usr/local"
else
    osascript -e 'display alert "Homebrew not found" message "Hieroglyphic requires Homebrew with gtk4 and libadwaita installed.\n\nInstall from https://brew.sh" as critical'
    exit 1
fi

export GSETTINGS_SCHEMA_DIR="$RESOURCES/share/glib-2.0/schemas"
export XDG_DATA_DIRS="$RESOURCES/share:$HOMEBREW_PREFIX/share:/usr/share"
export GDK_PIXBUF_MODULE_FILE="$HOMEBREW_PREFIX/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache"
export DYLD_FALLBACK_LIBRARY_PATH="$HOMEBREW_PREFIX/lib"

exec "$DIR/hieroglyphic-bin" "$@"
LAUNCHER
chmod +x "$APP_BUNDLE/Contents/MacOS/Hieroglyphic"

# --- Clean up build artifacts ---
rm -rf "$INSTALLDIR"

echo ""
echo "==> Built: $APP_BUNDLE"
echo "    Run with: open $APP_BUNDLE"
