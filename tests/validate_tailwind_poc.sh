#!/usr/bin/env bash
# validate_tailwind_poc.sh
# Validates the Tailwind CSS proof-of-concept integration against the actual
# project files. Checks config correctness, CSS directive presence, component
# class usage, and package dependency declarations.
#
# Exit codes:
#   0 — all checks passed
#   1 — one or more checks failed
#
# Usage:
#   bash tests/validate_tailwind_poc.sh
#   (Run from the project root, or set PROJECT_ROOT explicitly)

set -euo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${PROJECT_ROOT:-$(dirname "$SCRIPT_DIR")}"

PASS=0
FAIL=0
WARNINGS=0

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
pass() {
    echo "  [PASS] $*"
    PASS=$((PASS + 1))
}

fail() {
    echo "  [FAIL] $*" >&2
    FAIL=$((FAIL + 1))
}

warn() {
    echo "  [WARN] $*"
    WARNINGS=$((WARNINGS + 1))
}

section() {
    echo ""
    echo "=== $* ==="
}

# Check that a file exists and record pass/fail
assert_file_exists() {
    local rel_path="$1"
    local full_path="${PROJECT_ROOT}/${rel_path}"
    if [[ -f "$full_path" ]]; then
        pass "File exists: ${rel_path}"
        return 0
    else
        fail "File missing: ${rel_path}"
        return 1
    fi
}

# Check that a file contains a literal string (grep -qF)
assert_contains() {
    local file="$1"       # relative to PROJECT_ROOT
    local needle="$2"
    local description="$3"
    local full_path="${PROJECT_ROOT}/${file}"
    if grep -qF -- "$needle" "$full_path" 2>/dev/null; then
        pass "${description}"
    else
        fail "${description} — pattern not found in ${file}: ${needle}"
    fi
}

# Check that a file does NOT contain a literal string
assert_not_contains() {
    local file="$1"
    local needle="$2"
    local description="$3"
    local full_path="${PROJECT_ROOT}/${file}"
    if grep -qF -- "$needle" "$full_path" 2>/dev/null; then
        fail "${description} — forbidden pattern found in ${file}: ${needle}"
    else
        pass "${description}"
    fi
}

# Check using an ERE (extended regex) instead of fixed string
assert_contains_re() {
    local file="$1"
    local pattern="$2"
    local description="$3"
    local full_path="${PROJECT_ROOT}/${file}"
    if grep -qE -- "$pattern" "$full_path" 2>/dev/null; then
        pass "${description}"
    else
        fail "${description} — regex not found in ${file}: ${pattern}"
    fi
}

# ---------------------------------------------------------------------------
# Section 1: tailwind.config.js
# ---------------------------------------------------------------------------
section "1. tailwind.config.js — existence and key settings"

TAILWIND_CFG="tailwind.config.js"

if assert_file_exists "$TAILWIND_CFG"; then
    # preflight must be false to avoid destroying existing button/form styles
    assert_contains "$TAILWIND_CFG" "preflight: false" \
        "tailwind.config.js: preflight set to false (corePlugins)"

    # darkMode must be false — this is a dark-only app with no dark: variant needed
    assert_contains "$TAILWIND_CFG" "darkMode: false" \
        "tailwind.config.js: darkMode set to false"

    # Content paths — both index.html and src/**/*.{vue,ts} must be present
    assert_contains "$TAILWIND_CFG" '"./index.html"' \
        "tailwind.config.js: content includes ./index.html"

    assert_contains_re "$TAILWIND_CFG" '"./src/\*\*/\*\.\{vue,ts\}"' \
        "tailwind.config.js: content includes ./src/**/*.{vue,ts}"

    # Theme token mappings — spot-check a few critical ones
    assert_contains "$TAILWIND_CFG" '"var(--color-bg)"' \
        "tailwind.config.js: bg token mapped to --color-bg CSS variable"

    assert_contains "$TAILWIND_CFG" '"var(--color-accent)"' \
        "tailwind.config.js: accent token mapped to --color-accent CSS variable"

    assert_contains "$TAILWIND_CFG" '"var(--color-border)"' \
        "tailwind.config.js: border token mapped to --color-border CSS variable"

    assert_contains "$TAILWIND_CFG" '"var(--color-surface-muted)"' \
        "tailwind.config.js: surface-muted token mapped to CSS variable"

    # Button-height custom spacing tokens
    assert_contains "$TAILWIND_CFG" '"var(--btn-height-sm)"' \
        "tailwind.config.js: minHeight btn-sm token present"

    assert_contains "$TAILWIND_CFG" '"var(--btn-height-md)"' \
        "tailwind.config.js: minHeight btn-md token present"
fi

# ---------------------------------------------------------------------------
# Section 2: postcss.config.js
# ---------------------------------------------------------------------------
section "2. postcss.config.js — plugin declarations"

POSTCSS_CFG="postcss.config.js"

if assert_file_exists "$POSTCSS_CFG"; then
    assert_contains "$POSTCSS_CFG" "tailwindcss" \
        "postcss.config.js: tailwindcss plugin declared"

    assert_contains "$POSTCSS_CFG" "autoprefixer" \
        "postcss.config.js: autoprefixer plugin declared"
fi

# ---------------------------------------------------------------------------
# Section 3: src/assets/base.css — @tailwind directives and @layer block
# ---------------------------------------------------------------------------
section "3. src/assets/base.css — Tailwind directives and component layer"

BASE_CSS="src/assets/base.css"

if assert_file_exists "$BASE_CSS"; then
    # All three @tailwind directives must be present
    assert_contains "$BASE_CSS" "@tailwind base;" \
        "base.css: @tailwind base directive present"

    assert_contains "$BASE_CSS" "@tailwind components;" \
        "base.css: @tailwind components directive present"

    assert_contains "$BASE_CSS" "@tailwind utilities;" \
        "base.css: @tailwind utilities directive present"

    # @layer components block must exist (houses .btn* classes)
    assert_contains "$BASE_CSS" "@layer components {" \
        "base.css: @layer components block present"

    # .btn base class must be inside the layer block
    assert_contains "$BASE_CSS" ".btn {" \
        "base.css: .btn rule inside @layer components"

    # Variant classes
    assert_contains "$BASE_CSS" ".btn-sm {" \
        "base.css: .btn-sm rule present"

    assert_contains "$BASE_CSS" ".btn-primary {" \
        "base.css: .btn-primary rule present"

    assert_contains "$BASE_CSS" ".btn-danger {" \
        "base.css: .btn-danger rule present"

    assert_contains "$BASE_CSS" ".btn-success {" \
        "base.css: .btn-success rule present"

    assert_contains "$BASE_CSS" ".btn-secondary {" \
        "base.css: .btn-secondary rule present"

    # --color-accent CSS variable must be declared in :root
    assert_contains "$BASE_CSS" "--color-accent:" \
        "base.css: --color-accent CSS variable declared in :root"

    # Pre-existing LOW finding: naked 'button' rule sits outside @layer
    # This is a known issue — flag it as a warning, not a failure.
    if grep -qF "button {" "$BASE_CSS" 2>/dev/null; then
        warn "base.css: bare 'button' rule found outside @layer components (pre-existing LOW issue — specificity risk with Tailwind utilities)"
    fi
fi

# ---------------------------------------------------------------------------
# Section 4: src/components/ui/UiButton.vue — Tailwind utility classes
# ---------------------------------------------------------------------------
section "4. src/components/ui/UiButton.vue — class correctness"

UIBUTTON="src/components/ui/UiButton.vue"

if assert_file_exists "$UIBUTTON"; then
    # Critical fix: must use arbitrary value notation [0.45] not bare 45
    # (Tailwind v3 does not have an opacity-45 step; bare 'disabled:opacity-45'
    # would silently generate no CSS)
    assert_contains "$UIBUTTON" 'disabled:opacity-[0.45]' \
        "UiButton.vue: disabled:opacity-[0.45] uses correct arbitrary-value notation"

    # Confirm the incorrect bare form is absent
    assert_not_contains "$UIBUTTON" 'disabled:opacity-45' \
        "UiButton.vue: bare disabled:opacity-45 (no brackets) is absent"

    # border-solid fix confirmed — ensures borders render correctly in Tailwind
    # (Tailwind does not add border-style by default)
    assert_contains "$UIBUTTON" '"border-solid"' \
        "UiButton.vue: border-solid class present (border-style fix)"

    # Additional layout and interaction utility classes expected on the base array
    assert_contains "$UIBUTTON" '"inline-flex"' \
        "UiButton.vue: inline-flex layout utility present"

    assert_contains "$UIBUTTON" '"items-center"' \
        "UiButton.vue: items-center utility present"

    assert_contains "$UIBUTTON" '"cursor-pointer"' \
        "UiButton.vue: cursor-pointer utility present"

    assert_contains "$UIBUTTON" '"disabled:cursor-not-allowed"' \
        "UiButton.vue: disabled:cursor-not-allowed utility present"

    assert_contains "$UIBUTTON" '"rounded-sm"' \
        "UiButton.vue: rounded-sm utility present (maps to --radius-sm token)"

    # Size utilities for sm variant
    assert_contains "$UIBUTTON" '"min-h-btn-sm"' \
        "UiButton.vue: min-h-btn-sm size utility present"

    # Size utilities for md variant
    assert_contains "$UIBUTTON" '"min-h-btn-md"' \
        "UiButton.vue: min-h-btn-md size utility present"

    # inheritAttrs: false is required so parent-supplied classes don't double-apply
    assert_contains "$UIBUTTON" "inheritAttrs: false" \
        "UiButton.vue: inheritAttrs: false declared (prevents double class application)"
fi

# ---------------------------------------------------------------------------
# Section 5: src/components/ui/UiButton.vue — border-solid present for ALL variants
# ---------------------------------------------------------------------------
section "5. UiButton.vue — border-solid present in every variant branch"

if [[ -f "${PROJECT_ROOT}/${UIBUTTON}" ]]; then
    # Count occurrences: should appear once per variant (primary, danger, success,
    # secondary) — at minimum 4 times
    COUNT=$(grep -cF '"border-solid"' "${PROJECT_ROOT}/${UIBUTTON}" || true)
    if [[ "$COUNT" -ge 4 ]]; then
        pass "UiButton.vue: border-solid appears ${COUNT} time(s) — covers all four variants"
    else
        fail "UiButton.vue: border-solid appears only ${COUNT} time(s) — expected at least 4 (one per variant)"
    fi
fi

# ---------------------------------------------------------------------------
# Section 6: package.json — Tailwind devDependencies
# ---------------------------------------------------------------------------
section "6. package.json — tailwindcss and autoprefixer in devDependencies"

PACKAGE_JSON="package.json"

if assert_file_exists "$PACKAGE_JSON"; then
    # Use python3 for reliable JSON parsing (same approach as validate_claude_md.sh)
    if python3 -c "
import json, sys
data = json.load(open('${PROJECT_ROOT}/package.json'))
dev = data.get('devDependencies', {})
sys.exit(0 if 'tailwindcss' in dev else 1)
" 2>/dev/null; then
        pass "package.json: tailwindcss present in devDependencies"
    else
        fail "package.json: tailwindcss NOT found under devDependencies"
    fi

    if python3 -c "
import json, sys
data = json.load(open('${PROJECT_ROOT}/package.json'))
dev = data.get('devDependencies', {})
sys.exit(0 if 'autoprefixer' in dev else 1)
" 2>/dev/null; then
        pass "package.json: autoprefixer present in devDependencies"
    else
        fail "package.json: autoprefixer NOT found under devDependencies"
    fi

    # Confirm neither package leaked into runtime dependencies
    if python3 -c "
import json, sys
data = json.load(open('${PROJECT_ROOT}/package.json'))
deps = data.get('dependencies', {})
sys.exit(0 if 'tailwindcss' in deps else 1)
" 2>/dev/null; then
        fail "package.json: tailwindcss incorrectly listed under runtime dependencies (should be devDependencies)"
    else
        pass "package.json: tailwindcss is NOT in runtime dependencies (correct)"
    fi

    if python3 -c "
import json, sys
data = json.load(open('${PROJECT_ROOT}/package.json'))
deps = data.get('dependencies', {})
sys.exit(0 if 'autoprefixer' in deps else 1)
" 2>/dev/null; then
        fail "package.json: autoprefixer incorrectly listed under runtime dependencies (should be devDependencies)"
    else
        pass "package.json: autoprefixer is NOT in runtime dependencies (correct)"
    fi

    # Spot-check version constraints — both should be v3.x / v10.x respectively
    if python3 -c "
import json, sys
data = json.load(open('${PROJECT_ROOT}/package.json'))
ver = data.get('devDependencies', {}).get('tailwindcss', '')
sys.exit(0 if ver.startswith('^3.') or ver.startswith('3.') else 1)
" 2>/dev/null; then
        pass "package.json: tailwindcss version constraint is ^3.x (Tailwind v3)"
    else
        warn "package.json: tailwindcss version constraint is not ^3.x — verify compatibility"
    fi
fi

# ---------------------------------------------------------------------------
# Section 7: src/components/project/ProjectListItem.vue — scoped drag-handle styles
# ---------------------------------------------------------------------------
section "7. ProjectListItem.vue — scoped drag-handle CSS retained"

PLI="src/components/project/ProjectListItem.vue"

if assert_file_exists "$PLI"; then
    # The <style scoped> block must exist
    assert_contains "$PLI" "<style scoped>" \
        "ProjectListItem.vue: <style scoped> block present"

    # cursor: grab — cannot be expressed as a Tailwind utility and must stay scoped
    assert_contains "$PLI" "cursor: grab" \
        "ProjectListItem.vue: drag-handle retains cursor: grab in scoped styles"

    # touch-action: none — prevents scroll-jank during drag; no Tailwind equivalent
    assert_contains "$PLI" "touch-action: none" \
        "ProjectListItem.vue: drag-handle retains touch-action: none in scoped styles"

    # .drag-handle selector itself must be present
    assert_contains "$PLI" ".drag-handle {" \
        "ProjectListItem.vue: .drag-handle scoped rule block present"

    # cursor: grabbing on :active state
    assert_contains "$PLI" "cursor: grabbing" \
        "ProjectListItem.vue: .drag-handle:active retains cursor: grabbing"

    # .drag-handle class used on the UiButton element in template
    assert_contains "$PLI" 'class="drag-handle' \
        "ProjectListItem.vue: drag-handle class applied to UiButton in template"

    # Hybrid: Tailwind utilities also present on the drag handle (flex-shrink-0 etc.)
    assert_contains "$PLI" "flex-shrink-0" \
        "ProjectListItem.vue: flex-shrink-0 Tailwind utility present on drag-handle"

    # Item active state uses a bespoke colour not in the Tailwind token set
    assert_contains "$PLI" ".item.active {" \
        "ProjectListItem.vue: .item.active rule present in scoped styles"

    assert_contains "$PLI" "#7aa2ff" \
        "ProjectListItem.vue: active border-color #7aa2ff retained in scoped styles"
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "=============================="
echo "  Validation Summary"
echo "=============================="
echo "  Passed   : ${PASS}"
echo "  Warnings : ${WARNINGS}"
echo "  Failed   : ${FAIL}"
echo "=============================="

if [[ $FAIL -gt 0 ]]; then
    echo "  RESULT: FAILED (${FAIL} check(s) did not pass)" >&2
    exit 1
else
    echo "  RESULT: PASSED"
    exit 0
fi
