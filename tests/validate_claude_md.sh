#!/usr/bin/env bash
# validate_claude_md.sh
# Validates the accuracy of CLAUDE.md against the actual project files.
#
# Exit codes:
#   0 — all checks passed
#   1 — one or more checks failed
#
# Usage:
#   bash tests/validate_claude_md.sh
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

check_file_exists() {
    local rel_path="$1"
    local full_path="${PROJECT_ROOT}/${rel_path}"
    if [[ -e "$full_path" ]]; then
        pass "File/dir exists: ${rel_path}"
    else
        fail "File/dir missing: ${rel_path}"
    fi
}

# ---------------------------------------------------------------------------
# Section 1: Referenced file paths exist
# ---------------------------------------------------------------------------
section "1. Referenced file/directory paths from CLAUDE.md"

# src/ root files
check_file_exists "src/main.ts"
check_file_exists "src/App.vue"
check_file_exists "src/assets/base.css"

# components/
check_file_exists "src/components/AppToast.vue"
check_file_exists "src/components/ContextMenu.vue"
check_file_exists "src/components/FolderDropZone.vue"
check_file_exists "src/components/GitLogPanel.vue"

# components/git/
check_file_exists "src/components/git/ArchiveDialog.vue"
check_file_exists "src/components/git/GitLogEntry.vue"
check_file_exists "src/components/git/GitignoreEditor.vue"
check_file_exists "src/components/git/RemoteManagerDialog.vue"
check_file_exists "src/components/git/TemplatePicker.vue"

# components/project/
check_file_exists "src/components/project/CreateProjectModal.vue"
check_file_exists "src/components/project/DeleteConfirmDialog.vue"
check_file_exists "src/components/project/ProjectDeleteDialog.vue"
check_file_exists "src/components/project/ProjectListEmpty.vue"
check_file_exists "src/components/project/ProjectListItem.vue"
check_file_exists "src/components/project/RenameProjectDialog.vue"

# components/settings/
check_file_exists "src/components/settings/SettingsDataRootSection.vue"
check_file_exists "src/components/settings/SettingsDialog.vue"
check_file_exists "src/components/settings/SettingsExternalToolsTable.vue"
check_file_exists "src/components/settings/SettingsGitSection.vue"
check_file_exists "src/components/settings/SettingsLocale.vue"

# components/tree/
check_file_exists "src/components/tree/MoveRootToProjectModal.vue"

# components/ui/
check_file_exists "src/components/ui/UiButton.vue"
check_file_exists "src/components/ui/UiInput.vue"
check_file_exists "src/components/ui/UiSelect.vue"
check_file_exists "src/components/ui/UiTextarea.vue"

# components/workspace/
check_file_exists "src/components/workspace/FolderSelectionDetail.vue"
check_file_exists "src/components/workspace/GitStatusPathList.vue"
check_file_exists "src/components/workspace/GitStatusTreeNodes.vue"
check_file_exists "src/components/workspace/ProjectDetailPanel.vue"
check_file_exists "src/components/workspace/ProjectFolderPanel.vue"
check_file_exists "src/components/workspace/ProjectSidebar.vue"
check_file_exists "src/components/workspace/RootPathsList.vue"
check_file_exists "src/components/workspace/WorkingTreeStatusLabel.vue"

# composables/
check_file_exists "src/composables/bootstrap.ts"
check_file_exists "src/composables/pickFolder.ts"
check_file_exists "src/composables/rootFolderProjectPointerMove.ts"
check_file_exists "src/composables/useDialogShortcuts.ts"
check_file_exists "src/composables/useFolderRootRows.ts"

# i18n/
check_file_exists "src/i18n/index.ts"
check_file_exists "src/i18n/locales/ko.json"
check_file_exists "src/i18n/locales/en.json"

# pages/
check_file_exists "src/pages/BootGatePage.vue"
check_file_exists "src/pages/OnboardingPage.vue"
check_file_exists "src/pages/ProjectsWorkspacePage.vue"
check_file_exists "src/pages/NotFoundPage.vue"

# router/
check_file_exists "src/router/index.ts"
check_file_exists "src/router/routes.ts"
check_file_exists "src/router/guards.ts"

# stores/
check_file_exists "src/stores/projects.ts"
check_file_exists "src/stores/toast.ts"

# top-level config files
check_file_exists "vite.config.ts"
check_file_exists "package.json"

# src-tauri/
check_file_exists "src-tauri/Cargo.toml"
check_file_exists "src-tauri/tauri.conf.json"
check_file_exists "src-tauri/build.rs"

# Review finding #2: layouts/ listed as empty placeholder but actually contains AppShell.vue.
# Verify the directory exists AND that AppShell.vue is present (documenting the discrepancy).
if [[ -d "${PROJECT_ROOT}/src/layouts" ]]; then
    pass "Directory exists: src/layouts/"
    if [[ -f "${PROJECT_ROOT}/src/layouts/AppShell.vue" ]]; then
        warn "CLAUDE.md (line 106) marks layouts/ as an empty placeholder, but AppShell.vue exists inside it. The comment should be updated."
    else
        pass "src/layouts/ is empty (consistent with CLAUDE.md placeholder comment)"
    fi
else
    fail "Directory missing: src/layouts/ (referenced in CLAUDE.md)"
fi

# ---------------------------------------------------------------------------
# Section 2: npm scripts match package.json
# ---------------------------------------------------------------------------
section "2. npm scripts match package.json"

PACKAGE_JSON="${PROJECT_ROOT}/package.json"

check_npm_script() {
    local script_name="$1"
    if grep -q "\"${script_name}\"" "$PACKAGE_JSON"; then
        pass "npm script present: ${script_name}"
    else
        fail "npm script missing from package.json: ${script_name}"
    fi
}

# CLAUDE.md §4 documents these four scripts
check_npm_script "dev"
check_npm_script "build"
check_npm_script "preview"
check_npm_script "tauri"

# Review finding #1: @tauri-apps/api is listed as a runtime dependency in CLAUDE.md's
# tech-stack table but it is actually a devDependency in package.json.
# Verify that it appears under devDependencies (not dependencies).
if python3 -c "
import json, sys
data = json.load(open('${PACKAGE_JSON}'))
dev_deps = data.get('devDependencies', {})
sys.exit(0 if '@tauri-apps/api' in dev_deps else 1)
" 2>/dev/null; then
    pass "@tauri-apps/api correctly present in devDependencies"
else
    fail "@tauri-apps/api not found under devDependencies in package.json"
fi
if python3 -c "
import json, sys
data = json.load(open('${PACKAGE_JSON}'))
deps = data.get('dependencies', {})
sys.exit(0 if '@tauri-apps/api' in deps else 1)
" 2>/dev/null; then
    fail "CLAUDE.md issue #1: @tauri-apps/api appears in runtime dependencies — it should be devDependencies only"
else
    pass "@tauri-apps/api is NOT listed under runtime dependencies (correct)"
fi

# Verify the runtime dependencies documented in the tech-stack table
check_runtime_dep() {
    local pkg="$1"
    # Check it is present in the dependencies block (not devDependencies)
    if python3 -c "
import json, sys
data = json.load(open('${PACKAGE_JSON}'))
deps = data.get('dependencies', {})
sys.exit(0 if '${pkg}' in deps else 1)
" 2>/dev/null; then
        pass "Runtime dependency present: ${pkg}"
    else
        fail "Runtime dependency missing from package.json: ${pkg}"
    fi
}

check_runtime_dep "vue"
check_runtime_dep "vue-i18n"
check_runtime_dep "vue-router"
check_runtime_dep "@tauri-apps/plugin-dialog"

# ---------------------------------------------------------------------------
# Section 3: Rust module files exist in src-tauri/src/
# ---------------------------------------------------------------------------
section "3. Rust module files exist in src-tauri/src/"

RUST_SRC="${PROJECT_ROOT}/src-tauri/src"

check_rust_module() {
    local module="$1"
    local file="${RUST_SRC}/${module}.rs"
    if [[ -f "$file" ]]; then
        pass "Rust module file exists: src-tauri/src/${module}.rs"
    else
        fail "Rust module file missing: src-tauri/src/${module}.rs"
    fi
}

# CLAUDE.md §2 and §3 document these modules
check_rust_module "lib"
check_rust_module "main"
check_rust_module "app_lock"
check_rust_module "fs_tree"
check_rust_module "git"
check_rust_module "git_log"
check_rust_module "git_ops"
check_rust_module "gitignore"
check_rust_module "gitignore_templates"
check_rust_module "paths"
check_rust_module "persistence"
check_rust_module "settings"
check_rust_module "shell"

# Verify module declarations in lib.rs match CLAUDE.md visibility claims
LIB_RS="${RUST_SRC}/lib.rs"

echo ""
echo "  -- Module visibility checks (lib.rs) --"

check_mod_declaration() {
    local visibility="$1"   # "pub mod" or "^mod"
    local module="$2"
    local description="$3"

    if grep -qE "^${visibility} ${module};" "$LIB_RS"; then
        pass "${description}"
    else
        fail "${description}"
    fi
}

# CLAUDE.md: app_lock is 'pub mod'; all others are private 'mod'
check_mod_declaration "pub mod" "app_lock" "app_lock declared as pub mod (public) in lib.rs"
check_mod_declaration "mod" "fs_tree"   "fs_tree declared as private mod in lib.rs"
check_mod_declaration "mod" "git"       "git declared as private mod in lib.rs"
check_mod_declaration "mod" "git_log"   "git_log declared as private mod in lib.rs"
check_mod_declaration "mod" "git_ops"   "git_ops declared as private mod in lib.rs"
check_mod_declaration "mod" "gitignore" "gitignore declared as private mod in lib.rs"
check_mod_declaration "mod" "gitignore_templates" "gitignore_templates declared as private mod in lib.rs"
check_mod_declaration "mod" "paths"     "paths declared as private mod in lib.rs"
check_mod_declaration "mod" "persistence" "persistence declared as private mod in lib.rs"
check_mod_declaration "mod" "settings"  "settings declared as private mod in lib.rs"
check_mod_declaration "mod" "shell"     "shell declared as private mod in lib.rs"

# Confirm app_lock is NOT declared as private-only mod (i.e., no bare 'mod app_lock;' line)
if grep -qE "^mod app_lock;" "$LIB_RS"; then
    fail "app_lock has a private 'mod app_lock;' declaration — CLAUDE.md requires it to be 'pub mod'"
else
    pass "No conflicting private 'mod app_lock;' declaration found"
fi

# Review finding #3: CLAUDE.md shows AppState WITHOUT #[derive(Default)].
# Verify that in the actual lib.rs the struct does have #[derive(Default)].
if grep -B2 "struct AppState" "$LIB_RS" | grep -q "Default"; then
    warn "CLAUDE.md issue #3: AppState code block (line 113-114) omits '#[derive(Default)]'. Actual lib.rs has it."
else
    pass "AppState in lib.rs does not derive Default (consistent with CLAUDE.md code block)"
fi

# ---------------------------------------------------------------------------
# Section 4: IPC commands documented in CLAUDE.md appear in lib.rs
# ---------------------------------------------------------------------------
section "4. Documented IPC commands present in lib.rs"

check_ipc_command() {
    local cmd="$1"
    # A command must appear as a #[tauri::command] fn definition AND inside
    # generate_handler![]. The last entry in generate_handler! has no trailing
    # comma, so we accept both "  cmd," and "  cmd" (end of list or no comma).
    local fn_defined handler_registered
    fn_defined=$(grep -cE "^fn ${cmd}\b" "$LIB_RS" || true)
    handler_registered=$(grep -cE "^\s+${cmd}[,]?$" "$LIB_RS" || true)
    if [[ "$fn_defined" -gt 0 ]] && [[ "$handler_registered" -gt 0 ]]; then
        pass "IPC command registered: ${cmd}"
    else
        fail "IPC command missing or not registered: ${cmd}"
    fi
}

# projects_* commands (CLAUDE.md §7)
check_ipc_command "projects_list"
check_ipc_command "projects_create"
check_ipc_command "projects_update"
check_ipc_command "projects_delete"
check_ipc_command "projects_reorder"
check_ipc_command "projects_add_root"
check_ipc_command "projects_import_folder_drop"
check_ipc_command "projects_remove_root"
check_ipc_command "move_root_to_project"
check_ipc_command "check_root_path_available"

# git_* commands
check_ipc_command "git_remote_origin"
check_ipc_command "git_remote_summary"
check_ipc_command "git_remote_list"
check_ipc_command "git_remote_add"
check_ipc_command "git_remote_remove"
check_ipc_command "git_remote_rename"
check_ipc_command "git_remote_set_url"
check_ipc_command "git_status"
check_ipc_command "git_status_files"
check_ipc_command "git_probe_version"
check_ipc_command "git_init"
check_ipc_command "git_remove_dot_git"
check_ipc_command "git_archive_export"
check_ipc_command "folder_root_row"

# tree_* commands
check_ipc_command "tree_readdir"

# template_* commands
check_ipc_command "template_list"
check_ipc_command "template_sync"

# bootstrap / settings / system commands
check_ipc_command "get_bootstrap"
check_ipc_command "confirm_data_root"
check_ipc_command "get_settings"
check_ipc_command "update_settings"
check_ipc_command "logs_list"
check_ipc_command "reveal_path"
check_ipc_command "open_remote_in_browser"
check_ipc_command "open_external"
check_ipc_command "path_dot_git_exists"
check_ipc_command "path_is_git_repo_root"
check_ipc_command "read_gitignore"
check_ipc_command "write_gitignore"

# ---------------------------------------------------------------------------
# Section 5: Review-finding-specific checks
# ---------------------------------------------------------------------------
section "5. Review-finding-specific checks"

# Finding #4: bootstrap.json write strategy — CLAUDE.md claims "Direct fs::write".
# Verify that save_bootstrap in lib.rs uses fs::write (not atomic rename).
PERSISTENCE_RS="${RUST_SRC}/persistence.rs"

if grep -q "fn save_bootstrap" "$LIB_RS"; then
    if grep -A 15 "fn save_bootstrap" "$LIB_RS" | grep -q "fs::write"; then
        pass "Finding #4: bootstrap.json uses fs::write (direct, non-atomic) — matches CLAUDE.md claim"
    else
        fail "Finding #4: save_bootstrap in lib.rs does NOT use fs::write — CLAUDE.md claim may be wrong"
    fi
    if grep -A 15 "fn save_bootstrap" "$LIB_RS" | grep -q "fs::rename"; then
        fail "Finding #4: save_bootstrap uses fs::rename — CLAUDE.md should document it as atomic"
    else
        pass "Finding #4: save_bootstrap does NOT use fs::rename (non-atomic confirmed)"
    fi
else
    fail "Finding #4: save_bootstrap function not found in lib.rs"
fi

# Verify state.json and app-settings.json ARE atomic (CLAUDE.md table, line 331-332)
if grep -q "save_state_atomic" "$PERSISTENCE_RS" && \
   grep -A 10 "fn save_state_atomic" "$PERSISTENCE_RS" | grep -q "fs::rename"; then
    pass "Finding #4: state.json write is atomic (fs::rename present in save_state_atomic)"
else
    fail "Finding #4: save_state_atomic in persistence.rs does not appear to use fs::rename"
fi

if grep -q "fn save_app_settings" "$PERSISTENCE_RS" && \
   grep -A 10 "fn save_app_settings" "$PERSISTENCE_RS" | grep -q "fs::rename"; then
    pass "Finding #4: app-settings.json write is atomic (fs::rename present in save_app_settings)"
else
    fail "Finding #4: save_app_settings in persistence.rs does not appear to use fs::rename"
fi

# Finding #5: Project ID format p-{nanoseconds} — the value is a u128 (as_nanos() returns u128).
# Check the ID generation in lib.rs for as_nanos() usage.
if grep -q "as_nanos()" "$LIB_RS"; then
    pass "Finding #5: Project ID uses as_nanos() (u128) — CLAUDE.md should document p-{u128}"
    warn "Finding #5: CLAUDE.md documents Project ID as 'p-{nanoseconds}' but omits u128 type caveat (line 455)"
else
    fail "Finding #5: as_nanos() not found in lib.rs — Project ID generation differs from CLAUDE.md"
fi

# ---------------------------------------------------------------------------
# Section 6: Vite config cross-checks
# ---------------------------------------------------------------------------
section "6. Vite config cross-checks (CLAUDE.md §4, §5)"

VITE_CONFIG="${PROJECT_ROOT}/vite.config.ts"

if grep -q "port: 1420" "$VITE_CONFIG"; then
    pass "Dev server port is 1420 (matches CLAUDE.md)"
else
    fail "Dev server port is NOT 1420 in vite.config.ts"
fi

if grep -q "strictPort: true" "$VITE_CONFIG"; then
    pass "strictPort: true present in vite.config.ts (matches CLAUDE.md 'will fail if occupied')"
else
    warn "strictPort not set to true in vite.config.ts"
fi

if grep -q '"@"' "$VITE_CONFIG" && grep -q '"./src"' "$VITE_CONFIG"; then
    pass "@ alias mapped to ./src in vite.config.ts (matches CLAUDE.md §5)"
else
    fail "@ alias not configured as documented in CLAUDE.md §5"
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
