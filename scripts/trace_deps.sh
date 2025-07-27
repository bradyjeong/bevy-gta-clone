#!/bin/bash
#
# Dependency Tracing Script for GTA Game Codebase
# Analyzes dependencies for modules to assist with porting
#
# Usage: ./scripts/trace_deps.sh <target_directory>
# Example: ./scripts/trace_deps.sh crates/amp_gameplay/src/water
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${1:-}"

# Usage function
usage() {
    echo "Usage: $0 <target_directory>"
    echo ""
    echo "Analyzes dependencies for a given module to assist with porting."
    echo ""
    echo "Examples:"
    echo "  $0 crates/amp_gameplay/src/water"
    echo "  $0 crates/amp_render/src/culling"
    echo "  $0 src/legacy_module"
    echo ""
    echo "Output includes:"
    echo "  - File structure analysis"
    echo "  - External crate dependencies"
    echo "  - Internal module dependencies"
    echo "  - Bevy system usage patterns"
    echo "  - Component definitions"
    echo "  - Integration points"
}

# Validate arguments
if [[ -z "$TARGET_DIR" ]]; then
    echo -e "${RED}Error: Target directory is required${NC}"
    usage
    exit 1
fi

# Convert to absolute path
if [[ "$TARGET_DIR" == /* ]]; then
    FULL_TARGET_DIR="$TARGET_DIR"
else
    FULL_TARGET_DIR="$WORKSPACE_ROOT/$TARGET_DIR"
fi

# Validate target directory exists
if [[ ! -d "$FULL_TARGET_DIR" ]]; then
    echo -e "${RED}Error: Directory does not exist: $FULL_TARGET_DIR${NC}"
    exit 1
fi

echo -e "${BLUE}🔍 Analyzing dependencies for: $TARGET_DIR${NC}"
echo -e "${BLUE}Full path: $FULL_TARGET_DIR${NC}"
echo ""

# File structure analysis
echo -e "${GREEN}📁 File Structure:${NC}"
find "$FULL_TARGET_DIR" -name "*.rs" -type f | sort | while read -r file; do
    rel_path="${file#$FULL_TARGET_DIR/}"
    line_count=$(wc -l < "$file" 2>/dev/null || echo "0")
    echo "  ${rel_path} (${line_count} lines)"
done
echo ""

# External crate dependencies
echo -e "${GREEN}📦 External Crate Dependencies:${NC}"
external_deps=$(find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -h "^use [a-zA-Z0-9_]\+::" {} \; 2>/dev/null | \
    grep -v "^use crate::" | \
    grep -v "^use super::" | \
    grep -v "^use std::" | \
    sed 's/^use \([a-zA-Z0-9_]\+\)::.*/\1/' | \
    sort | uniq -c | sort -nr)

if [[ -n "$external_deps" ]]; then
    echo "$external_deps" | while read -r count crate; do
        echo "  ${crate} (${count} usages)"
    done
else
    echo "  No external crate dependencies found"
fi
echo ""

# Internal module dependencies
echo -e "${GREEN}🔗 Internal Module Dependencies:${NC}"
internal_deps=$(find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -h "^use crate::" {} \; 2>/dev/null | \
    sed 's/^use crate::\([^;]*\);.*/\1/' | \
    sort | uniq -c | sort -nr)

if [[ -n "$internal_deps" ]]; then
    echo "$internal_deps" | while read -r count module; do
        echo "  crate::${module} (${count} usages)"
    done
else
    echo "  No internal module dependencies found"
fi
echo ""

# Bevy-specific patterns
echo -e "${GREEN}🚀 Bevy System Patterns:${NC}"

# Find component definitions
components=$(find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -l "#\[derive.*Component" {} \;)
if [[ -n "$components" ]]; then
    echo "  Components found in:"
    echo "$components" | while read -r file; do
        rel_path="${file#$FULL_TARGET_DIR/}"
        component_names=$(grep "#\[derive.*Component" "$file" -A1 | grep "^pub struct\|^struct" | sed 's/.*struct \([a-zA-Z0-9_]\+\).*/\1/')
        echo "    ${rel_path}: $(echo $component_names | tr '\n' ', ' | sed 's/,$//')"
    done
else
    echo "  No Bevy components found"
fi
echo ""

# Find system functions
systems=$(find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -l "Query<\|Res<\|ResMut<\|Commands\|EventWriter<\|EventReader<" {} \;)
if [[ -n "$systems" ]]; then
    echo "  System functions found in:"
    echo "$systems" | while read -r file; do
        rel_path="${file#$FULL_TARGET_DIR/}"
        system_count=$(grep -c "fn.*Query<\|fn.*Res<\|fn.*ResMut<\|fn.*Commands\|fn.*EventWriter<\|fn.*EventReader<" "$file" 2>/dev/null || echo "0")
        echo "    ${rel_path} (${system_count} potential systems)"
    done
else
    echo "  No Bevy systems found"
fi
echo ""

# Find plugin definitions
plugins=$(find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -l "impl Plugin for\|#\[derive.*Plugin" {} \;)
if [[ -n "$plugins" ]]; then
    echo "  Plugin definitions found in:"
    echo "$plugins" | while read -r file; do
        rel_path="${file#$FULL_TARGET_DIR/}"
        plugin_names=$(grep "impl Plugin for\|pub struct.*Plugin" "$file" | sed 's/.*impl Plugin for \([a-zA-Z0-9_]\+\).*/\1/; s/.*struct \([a-zA-Z0-9_]*Plugin[a-zA-Z0-9_]*\).*/\1/')
        echo "    ${rel_path}: $(echo $plugin_names | tr '\n' ', ' | sed 's/,$//')"
    done
else
    echo "  No Bevy plugins found"
fi
echo ""

# Resource usage analysis
echo -e "${GREEN}💾 Resource Usage:${NC}"
resources=$(find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -ho "Res<[^>]*>" {} \; 2>/dev/null | sort | uniq -c | sort -nr)
if [[ -n "$resources" ]]; then
    echo "$resources" | while read -r count resource; do
        echo "  ${resource} (${count} usages)"
    done
else
    echo "  No resource usage found"
fi
echo ""

# Event usage analysis
echo -e "${GREEN}📡 Event Usage:${NC}"
events=$(find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -ho "EventWriter<[^>]*>\|EventReader<[^>]*>" {} \; 2>/dev/null | sort | uniq -c | sort -nr)
if [[ -n "$events" ]]; then
    echo "$events" | while read -r count event; do
        echo "  ${event} (${count} usages)"
    done
else
    echo "  No event usage found"
fi
echo ""

# Test file analysis
echo -e "${GREEN}🧪 Test Coverage:${NC}"
test_files=$(find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -l "#\[test\]\|#\[cfg(test)\]" {} \;)
if [[ -n "$test_files" ]]; then
    echo "  Test files found:"
    echo "$test_files" | while read -r file; do
        rel_path="${file#$FULL_TARGET_DIR/}"
        test_count=$(grep -c "#\[test\]" "$file" 2>/dev/null || echo "0")
        echo "    ${rel_path} (${test_count} test functions)"
    done
else
    echo "  No test files found"
fi
echo ""

# Potential porting concerns
echo -e "${YELLOW}⚠️  Potential Porting Concerns:${NC}"

# Check for deprecated patterns
deprecated_patterns=()

# Check for old Bevy API usage
if find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -q "app\.world[^()]" {} \; 2>/dev/null; then
    deprecated_patterns+=("Old Bevy API: app.world (should be app.world())")
fi

# Check for complex circular dependencies
complex_deps=$(find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -h "^use crate::" {} \; 2>/dev/null | grep -c "amp_" 2>/dev/null || echo "0")
if [[ "${complex_deps}" -gt 5 ]]; then
    deprecated_patterns+=("High cross-crate coupling (${complex_deps} amp_* imports)")
fi

# Check for hard-coded paths or values
if find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -q "\"assets/\|\"config/\|\"saves/\"" {} \; 2>/dev/null; then
    deprecated_patterns+=("Hard-coded paths detected (should use configuration)")
fi

if [[ ${#deprecated_patterns[@]} -gt 0 ]]; then
    for pattern in "${deprecated_patterns[@]}"; do
        echo "  - $pattern"
    done
else
    echo "  No obvious concerns detected"
fi
echo ""

# Suggested target crate analysis
echo -e "${PURPLE}🎯 Suggested Target Crate Analysis:${NC}"

# Analyze content to suggest appropriate crate
if find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -q "Transform\|Material\|Mesh\|Camera" {} \; 2>/dev/null; then
    echo "  Rendering components detected → Consider amp_render"
fi

if find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -q "RigidBody\|Collider\|Velocity\|rapier" {} \; 2>/dev/null; then
    echo "  Physics components detected → Consider amp_physics"
fi

if find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -q "Player\|NPC\|Vehicle\|Character" {} \; 2>/dev/null; then
    echo "  Gameplay components detected → Consider amp_gameplay"
fi

if find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -q "Config\|Settings\|\.ron\|serde" {} \; 2>/dev/null; then
    echo "  Configuration components detected → Consider config_core"
fi

if find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -q "spawn\|factory\|Entity.*bundle" {} \; 2>/dev/null; then
    echo "  Entity spawning detected → Consider gameplay_factory"
fi

if find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -q "AABB\|Morton\|Vec3\|Quat" {} \; 2>/dev/null; then
    echo "  Math utilities detected → Consider amp_math"
fi

if ! find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec grep -q "bevy::" {} \; 2>/dev/null; then
    echo "  No Bevy dependencies detected → Consider amp_core"
fi

echo ""

# Summary
echo -e "${CYAN}📋 Summary:${NC}"
file_count=$(find "$FULL_TARGET_DIR" -name "*.rs" -type f | wc -l)
total_lines=$(find "$FULL_TARGET_DIR" -name "*.rs" -type f -exec wc -l {} \; 2>/dev/null | awk '{sum += $1} END {print sum}')
echo "  Total Rust files: $file_count"
echo "  Total lines of code: ${total_lines:-0}"
echo "  Complexity estimate: $(if [[ ${total_lines:-0} -lt 200 ]]; then echo "Low"; elif [[ ${total_lines:-0} -lt 800 ]]; then echo "Medium"; else echo "High"; fi)"
echo ""

echo -e "${GREEN}✅ Analysis Complete!${NC}"
echo -e "${YELLOW}💡 Next steps: Use this analysis to fill out docs/PORTING_TEMPLATE.md${NC}"
