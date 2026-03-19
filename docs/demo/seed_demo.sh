#!/bin/bash
# Seed a demo Claude projects directory with realistic multi-model session data
# Usage: ./seed_demo.sh /tmp/aitop-demo

set -e

DEMO_DIR="${1:-/tmp/aitop-demo}"
PROJECTS_DIR="$DEMO_DIR/projects"

rm -rf "$DEMO_DIR"
mkdir -p "$PROJECTS_DIR"

# Helper: generate a JSONL session file
gen_session() {
    local project_dir="$1"
    local session_id="$2"
    local model="$3"
    local base_ts="$4"       # base timestamp (epoch seconds)
    local num_turns="$5"     # number of assistant turns
    local input_base="$6"    # base input tokens
    local output_base="$7"   # base output tokens
    local cache_pct="$8"     # cache read percentage (0-100)

    local file="$project_dir/${session_id}.jsonl"

    # First user message (session start)
    local ts=$(date -u -r "$base_ts" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -d "@$base_ts" +%Y-%m-%dT%H:%M:%SZ)
    echo "{\"uuid\":\"${session_id}-u0\",\"sessionId\":\"${session_id}\",\"type\":\"user\",\"timestamp\":\"${ts}\",\"parentUuid\":null,\"message\":{\"role\":\"user\"}}" > "$file"

    local parent="${session_id}-u0"
    for i in $(seq 1 "$num_turns"); do
        local offset=$((i * 30))
        local msg_ts=$((base_ts + offset))
        ts=$(date -u -r "$msg_ts" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -d "@$msg_ts" +%Y-%m-%dT%H:%M:%SZ)

        # Vary tokens a bit
        local input=$((input_base + RANDOM % (input_base / 2)))
        local output=$((output_base + RANDOM % (output_base / 2)))
        local cache_read=$((input * cache_pct / 100))
        local cache_creation=$((input / 10))

        # Assistant response
        echo "{\"uuid\":\"${session_id}-a${i}\",\"sessionId\":\"${session_id}\",\"type\":\"assistant\",\"timestamp\":\"${ts}\",\"message\":{\"model\":\"${model}\",\"role\":\"assistant\",\"usage\":{\"input_tokens\":${input},\"output_tokens\":${output},\"cache_read_input_tokens\":${cache_read},\"cache_creation_input_tokens\":${cache_creation}}}}" >> "$file"

        # Follow-up user message
        local user_ts=$((msg_ts + 15))
        ts=$(date -u -r "$user_ts" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -d "@$user_ts" +%Y-%m-%dT%H:%M:%SZ)
        echo "{\"uuid\":\"${session_id}-u${i}\",\"sessionId\":\"${session_id}\",\"type\":\"user\",\"timestamp\":\"${ts}\",\"parentUuid\":\"${session_id}-a${i}\",\"message\":{\"role\":\"user\"}}" >> "$file"
        parent="${session_id}-u${i}"
    done
}

NOW=$(date +%s)
DAY=86400

# --- Project: aitop (this project, heavy opus usage) ---
proj="$PROJECTS_DIR/-Users-demo-Dev-aitop"
mkdir -p "$proj"
gen_session "$proj" "sess-aitop-1" "claude-opus-4-6-20250514" $((NOW - 1 * DAY)) 25 8000 2000 90
gen_session "$proj" "sess-aitop-2" "claude-opus-4-6-20250514" $((NOW - 3 * DAY)) 40 10000 3000 92
gen_session "$proj" "sess-aitop-3" "claude-sonnet-4-6-20250514" $((NOW - 1800)) 15 3000 1500 85
gen_session "$proj" "sess-aitop-4" "claude-opus-4-6-20250514" $((NOW - 5 * DAY)) 20 9000 2500 91
gen_session "$proj" "sess-aitop-5" "claude-sonnet-4-6-20250514" $((NOW - 7 * DAY)) 30 4000 2000 86

# --- Project: webapp (sonnet-heavy web development) ---
proj="$PROJECTS_DIR/-Users-demo-Dev-webapp"
mkdir -p "$proj"
gen_session "$proj" "sess-webapp-1" "claude-sonnet-4-6-20250514" $((NOW - 2 * DAY)) 30 4000 2000 88
gen_session "$proj" "sess-webapp-2" "claude-sonnet-4-6-20250514" $((NOW - 4 * DAY)) 20 3500 1800 80
gen_session "$proj" "sess-webapp-3" "claude-3-5-haiku-20241022" $((NOW - 900)) 50 1000 500 75
gen_session "$proj" "sess-webapp-4" "claude-sonnet-4-6-20250514" $((NOW - 6 * DAY)) 25 5000 2200 83
gen_session "$proj" "sess-webapp-5" "claude-sonnet-4-6-20250514" $((NOW - 9 * DAY)) 35 4500 2100 85

# --- Project: api-server (mixed models) ---
proj="$PROJECTS_DIR/-Users-demo-Dev-api-server"
mkdir -p "$proj"
gen_session "$proj" "sess-api-1" "claude-opus-4-6-20250514" $((NOW - 3 * DAY)) 10 12000 4000 95
gen_session "$proj" "sess-api-2" "claude-sonnet-4-6-20250514" $((NOW - 5 * DAY)) 35 5000 2500 87
gen_session "$proj" "sess-api-3" "claude-3-5-haiku-20241022" $((NOW - 600)) 60 800 400 70
gen_session "$proj" "sess-api-4" "claude-opus-4-6-20250514" $((NOW - 8 * DAY)) 18 11000 3500 93
gen_session "$proj" "sess-api-5" "claude-3-5-haiku-20241022" $((NOW - 10 * DAY)) 45 900 450 72

# --- Project: ml-pipeline (data science work) ---
proj="$PROJECTS_DIR/-Users-demo-Dev-ml-pipeline"
mkdir -p "$proj"
gen_session "$proj" "sess-ml-1" "claude-opus-4-6-20250514" $((NOW - 2 * DAY)) 15 15000 5000 93
gen_session "$proj" "sess-ml-2" "claude-sonnet-4-6-20250514" $((NOW - 4 * DAY)) 25 6000 3000 82
gen_session "$proj" "sess-ml-3" "claude-opus-4-6-20250514" $((NOW - 11 * DAY)) 12 14000 4500 94
gen_session "$proj" "sess-ml-4" "claude-sonnet-4-6-20250514" $((NOW - 13 * DAY)) 20 5500 2800 80

# --- Project: docs (quick haiku tasks) ---
proj="$PROJECTS_DIR/-Users-demo-Dev-docs"
mkdir -p "$proj"
gen_session "$proj" "sess-docs-1" "claude-3-5-haiku-20241022" $((NOW - 1 * DAY)) 40 600 300 65
gen_session "$proj" "sess-docs-2" "claude-3-5-haiku-20241022" $((NOW - 6 * DAY)) 30 500 250 60
gen_session "$proj" "sess-docs-3" "claude-3-5-haiku-20241022" $((NOW - 12 * DAY)) 35 700 350 68

echo "Demo data created at: $DEMO_DIR/projects"
echo "Projects: aitop, webapp, api-server, ml-pipeline, docs"
echo ""
echo "Run aitop with:"
echo "  aitop  (after setting data_dir in config, or with default ~/.claude/projects)"
echo ""
echo "Or copy to ~/.claude/projects to use with default config."
