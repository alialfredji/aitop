#!/bin/bash
# Take polished screenshots of aitop from your real terminal (Ghostty/tmux).
#
# Usage:
#   1. Run: ./docs/demo/take_screenshots.sh setup
#      → Seeds demo data, swaps config, deletes DB so aitop re-ingests
#   2. Launch aitop manually in your terminal
#   3. For each view, run: ./docs/demo/take_screenshots.sh capture <name>
#      → e.g. capture dashboard, capture sessions, capture models, capture trends
#      → Uses macOS screencapture interactive mode (crosshair) so you select the window
#   4. Run: ./docs/demo/take_screenshots.sh teardown
#      → Restores your real config and DB

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
SCREENSHOTS_DIR="$PROJECT_DIR/docs/screenshots"
CONFIG_DIR="$HOME/Library/Application Support/aitop"
BACKUP_DIR="/tmp/aitop-screenshot-backup"
DEMO_DIR="/tmp/aitop-demo"

case "${1:-}" in
  setup)
    echo "=== Setting up demo data ==="

    # Backup current config and DB
    mkdir -p "$BACKUP_DIR"
    cp "$CONFIG_DIR/config.toml" "$BACKUP_DIR/config.toml" 2>/dev/null || true
    cp "$CONFIG_DIR/sessions.db" "$BACKUP_DIR/sessions.db" 2>/dev/null || true
    echo "Backed up config and DB to $BACKUP_DIR"

    # Seed demo data
    bash "$SCRIPT_DIR/seed_demo.sh" "$DEMO_DIR"

    # Write demo config
    cat > "$CONFIG_DIR/config.toml" <<EOF
refresh = 2.0
theme = "ember"
data_dir = "$DEMO_DIR/projects"
weekly_budget = 500.0
EOF

    # Delete DB so aitop re-ingests from demo data
    rm -f "$CONFIG_DIR/sessions.db"

    echo ""
    echo "=== Ready! ==="
    echo "Now launch aitop in your Ghostty terminal:"
    echo "  ./target/release/aitop"
    echo ""
    echo "Then capture each view:"
    echo "  ./docs/demo/take_screenshots.sh capture dashboard"
    echo "  ./docs/demo/take_screenshots.sh capture sessions"
    echo "  ./docs/demo/take_screenshots.sh capture models"
    echo "  ./docs/demo/take_screenshots.sh capture trends"
    echo ""
    echo "When done: ./docs/demo/take_screenshots.sh teardown"
    ;;

  capture)
    NAME="${2:?Usage: $0 capture <dashboard|sessions|models|trends>}"
    mkdir -p "$SCREENSHOTS_DIR"
    OUTPUT="$SCREENSHOTS_DIR/${NAME}.png"

    echo "Click on the aitop window to capture it..."
    screencapture -w "$OUTPUT"

    if [ -f "$OUTPUT" ]; then
      echo "Saved: $OUTPUT ($(du -h "$OUTPUT" | cut -f1))"
    else
      echo "Capture cancelled."
    fi
    ;;

  teardown)
    echo "=== Restoring real config and DB ==="

    if [ -d "$BACKUP_DIR" ]; then
      cp "$BACKUP_DIR/config.toml" "$CONFIG_DIR/config.toml" 2>/dev/null || true
      cp "$BACKUP_DIR/sessions.db" "$CONFIG_DIR/sessions.db" 2>/dev/null || true
      rm -rf "$BACKUP_DIR"
      echo "Restored from backup."
    else
      echo "No backup found at $BACKUP_DIR"
    fi

    # Clean up demo data
    rm -rf "$DEMO_DIR"
    echo "Cleaned up demo data."
    echo "Done!"
    ;;

  *)
    echo "Usage: $0 <setup|capture|teardown>"
    echo ""
    echo "  setup     - Seed demo data and swap config"
    echo "  capture   - Take a screenshot (e.g. capture dashboard)"
    echo "  teardown  - Restore real config and clean up"
    ;;
esac
