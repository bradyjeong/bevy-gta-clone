#!/bin/bash
# Oracle's Version Consistency Guard
# Ensures single source of truth for all versions
# Two-layer approach: declarative hygiene + actual resolution sanity

set -e

echo "🔍 Checking version consistency..."

# ====================================
# LAYER 1: Dependency-List Hygiene
# ====================================
echo "🧹 Layer 1: Checking dependency-list hygiene..."

ROOT_TOML="Cargo.toml"
fail=false

# Get workspace dependencies using more robust parsing
workspace_deps=$(awk '/^\[workspace\.dependencies\]/{flag=1; next} /^\[/{flag=0} flag && /^[a-zA-Z_][a-zA-Z0-9_-]*[[:space:]]*=/ {print $1}' "$ROOT_TOML" | sort -u)

if [ -z "$workspace_deps" ]; then
    echo "✅ No workspace dependencies to check"
else
    echo "🔍 Ensuring all workspace deps are inherited..."
    
    # Check each member crate for violations
    for crate_toml in $(find crates -name "Cargo.toml"); do
        crate_name=$(basename "$(dirname "$crate_toml")")
        
        while IFS= read -r dep; do
            if [ -n "$dep" ]; then
                # Check each dependency section separately
                for section in "dependencies" "dev-dependencies" "build-dependencies"; do
                    # Extract the section and check for our dependency
                    section_content=$(awk "/^\[$section\]/{flag=1; next} /^\[/{flag=0} flag" "$crate_toml")
                    
                    if echo "$section_content" | grep -q "^[[:space:]]*$dep[[:space:]]*="; then
                        line=$(echo "$section_content" | grep "^[[:space:]]*$dep[[:space:]]*=" | head -n 1)
                        
                        # Check if it has version field and NOT workspace = true
                        if echo "$line" | grep -q "version[[:space:]]*=" && ! echo "$line" | grep -q "workspace[[:space:]]*=[[:space:]]*true"; then
                            echo "❌ $crate_name pins $dep locally in [$section]; use { workspace = true }."
                            fail=true
                        fi
                        
                        # Check shorthand form: foo = "version"
                        if echo "$line" | grep -E "^[[:space:]]*$dep[[:space:]]*=[[:space:]]*\"[^\"]*\"[[:space:]]*$" | grep -q .; then
                            echo "❌ $crate_name uses shorthand version for $dep in [$section]; use { workspace = true }."
                            fail=true
                        fi
                        
                        # Check for conflicting workspace = true + version = "..."
                        if echo "$line" | grep -q "workspace[[:space:]]*=[[:space:]]*true" && echo "$line" | grep -q "version[[:space:]]*="; then
                            echo "❌ $crate_name has conflicting workspace = true AND version for $dep in [$section]; remove version field."
                            fail=true
                        fi
                    fi
                done
            fi
        done <<< "$workspace_deps"
    done
fi

# ====================================
# LAYER 1.5: Reverse Check - workspace=true without root declaration
# ====================================
echo "🔍 Ensuring every 'workspace = true' points to root..."

for crate_toml in $(find crates -name "Cargo.toml"); do
    crate_name=$(basename "$(dirname "$crate_toml")")
    
    # Extract all dependencies that use workspace = true
    workspace_deps_used=$(grep -E "^[[:space:]]*[a-zA-Z0-9_-]+[[:space:]]*=.*workspace[[:space:]]*=[[:space:]]*true" "$crate_toml" | \
                          sed -E 's/^[[:space:]]*([a-zA-Z0-9_-]+)[[:space:]]*=.*/\1/' | sort -u)
    
    while IFS= read -r dep; do
        if [ -n "$dep" ]; then
            # Check if this dependency exists in workspace.dependencies
            if ! echo "$workspace_deps" | grep -q "^$dep$"; then
                echo "❌ $crate_name uses 'workspace = true' for $dep but it's missing in [workspace.dependencies]"
                fail=true
            fi
        fi
    done <<< "$workspace_deps_used"
done

# ====================================
# LAYER 0: Warn about duplicate root+local version declarations (optional)
# ====================================
echo "🔍 Checking for duplicate version declarations..."

# Check if root package has hardcoded versions for workspace dependencies (simplified)
root_deps=$(sed -n '/^\[dependencies\]/,/^\[/p' "$ROOT_TOML" | \
           grep -E "^[[:space:]]*[a-zA-Z0-9_-]+[[:space:]]*=" | \
           grep -v "workspace[[:space:]]*=" | \
           sed -E 's/^[[:space:]]*([a-zA-Z0-9_-]+)[[:space:]]*=.*/\1/' | sort -u)

while IFS= read -r dep; do
    if [ -n "$dep" ]; then
        if echo "$workspace_deps" | grep -q "^$dep$"; then
            echo "⚠️  Root package declares '$dep' locally but it also exists in [workspace.dependencies] - consider using workspace inheritance"
        fi
    fi
done <<< "$root_deps"

# ====================================
# LAYER 2: Actual Resolution Sanity
# ====================================
echo "🔍 Layer 2: Checking actual resolution sanity..."

# Check for duplicate major versions of critical dependencies
check_critical_duplicates() {
    local crate_name="$1"
    local duplicates=$(cargo tree --duplicates | grep "^$crate_name v" | wc -l)
    if [ "$duplicates" -gt 1 ]; then
        echo "❌ Found multiple major versions of $crate_name:"
        cargo tree --duplicates | grep "^$crate_name v"
        return 1
    fi
    return 0
}

# Critical dependencies that must not have version conflicts
# Note: Only check deps where we control both sides - not transitive deps from Bevy
critical_deps=(
    "wgpu"
    "winit" 
    "glam"
    "bevy"
    "serde"
    "ron"
    "pollster"
    "anyhow"
)

echo "🔍 Checking for duplicate major versions..."
# Use -e no-dev to avoid failing CI on dev-only duplicates
for dep in "${critical_deps[@]}"; do
    duplicates=$(cargo tree -e no-dev --duplicates | grep "^$dep v" | wc -l)
    if [ "$duplicates" -gt 1 ]; then
        echo "❌ Found multiple major versions of $dep in production dependencies:"
        cargo tree -e no-dev --duplicates | grep "^$dep v"
        fail=true
    fi
done

# Enforce hard patch locks for engine nucleus
echo "🔍 Verifying engine nucleus version locks..."
if ! grep -q 'bevy = "=0\.16\.1"' "$ROOT_TOML"; then
    echo "❌ Bevy version not patch-locked to =0.16.1"
    fail=true
fi

if ! grep -q 'bevy_rapier3d = "=0\.26\.0"' "$ROOT_TOML"; then
    echo "❌ bevy_rapier3d version not patch-locked to =0.26.0"
    fail=true
fi

# ====================================
# FINAL RESULT
# ====================================
if [ "$fail" = true ]; then
    echo ""
    echo "❌ Version consistency violations found!"
    echo "To debug: cargo tree --duplicates"
    echo "See Oracle's version consistency rules in docs/ADR-version-policy.md"
    exit 1
fi

echo ""
echo "✅ Version consistency checks passed"
echo "Oracle's two-layer guard verified:"
echo "  - Dependency-list hygiene ✓"
echo "  - Actual resolution sanity ✓"
