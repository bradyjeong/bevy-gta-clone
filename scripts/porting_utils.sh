#!/bin/bash
#
# Porting Utilities for GTA Game Codebase
# Collection of helper functions for code porting tasks
#
# Usage: source scripts/porting_utils.sh
# Then call individual functions as needed
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
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.." && pwd)"

# Logging functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_step() {
    echo -e "${PURPLE}🔄 $1${NC}"
}

# Create module structure based on successful patterns
create_module_structure() {
    local target_crate="$1"
    local module_name="$2"
    
    if [[ -z "$target_crate" || -z "$module_name" ]]; then
        log_error "Usage: create_module_structure <target_crate> <module_name>"
        return 1
    fi
    
    local target_dir="$WORKSPACE_ROOT/crates/$target_crate/src/$module_name"
    
    log_step "Creating module structure for $target_crate/$module_name"
    
    # Create directory structure
    mkdir -p "$target_dir"
    
    # Create mod.rs with standard exports
    cat > "$target_dir/mod.rs" << 'EOF'
pub mod components;
pub mod plugin;
pub mod systems;

#[cfg(test)]
mod tests;

pub use components::*;
pub use plugin::*;
pub use systems::*;
EOF
    
    # Create components.rs template
    cat > "$target_dir/components.rs" << 'EOF'
use bevy::prelude::*;

// Add your components here following this pattern:
// #[derive(Component, Default, Debug, Clone)]
// pub struct YourComponent {
//     // fields here
// }
EOF
    
    # Create systems.rs template
    cat > "$target_dir/systems.rs" << 'EOF'
use bevy::prelude::*;
use super::components::*;

// Add your systems here following this pattern:
// pub fn your_system(
//     query: Query<&YourComponent>,
// ) {
//     // system logic here
// }
EOF
    
    # Create plugin.rs template
    cat > "$target_dir/plugin.rs" << EOF
use bevy::prelude::*;
use super::systems::*;

/// Plugin for the $module_name module
#[derive(Default)]
pub struct ${module_name^}Plugin;

impl Plugin for ${module_name^}Plugin {
    fn build(&self, app: &mut App) {
        app
            // Add your systems here:
            // .add_systems(Startup, setup_system)
            // .add_systems(Update, update_system)
            ;
        
        info!("📦 ${module_name^} Plugin initialized");
    }
}
EOF
    
    # Create tests.rs template
    cat > "$target_dir/tests.rs" << 'EOF'
use super::*;
use bevy::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_components() {
        // Add component tests here
    }

    #[test]
    fn test_module_systems() {
        // Add system tests here
    }
}
EOF
    
    log_success "Module structure created at $target_dir"
    log_info "Next steps:"
    log_info "1. Fill in the component definitions in components.rs"
    log_info "2. Implement your systems in systems.rs"
    log_info "3. Configure the plugin in plugin.rs"
    log_info "4. Add tests in tests.rs"
    log_info "5. Export the module in the parent crate's lib.rs"
}

# Validate module structure against successful patterns
validate_module_structure() {
    local target_dir="$1"
    
    if [[ -z "$target_dir" ]]; then
        log_error "Usage: validate_module_structure <module_directory>"
        return 1
    fi
    
    if [[ ! -d "$target_dir" ]]; then
        log_error "Directory does not exist: $target_dir"
        return 1
    fi
    
    log_step "Validating module structure: $target_dir"
    
    local errors=0
    
    # Check required files
    local required_files=("mod.rs" "components.rs" "systems.rs" "plugin.rs")
    for file in "${required_files[@]}"; do
        if [[ ! -f "$target_dir/$file" ]]; then
            log_error "Missing required file: $file"
            ((errors++))
        else
            log_success "Found: $file"
        fi
    done
    
    # Check mod.rs exports
    if [[ -f "$target_dir/mod.rs" ]]; then
        if grep -q "pub use components::\*;" "$target_dir/mod.rs" && \
           grep -q "pub use systems::\*;" "$target_dir/mod.rs" && \
           grep -q "pub use plugin::\*;" "$target_dir/mod.rs"; then
            log_success "mod.rs has proper exports"
        else
            log_warning "mod.rs might be missing proper exports"
        fi
    fi
    
    # Check for plugin implementation
    if [[ -f "$target_dir/plugin.rs" ]]; then
        if grep -q "impl Plugin for" "$target_dir/plugin.rs"; then
            log_success "Plugin implementation found"
        else
            log_warning "Plugin implementation not found in plugin.rs"
        fi
    fi
    
    # Check for Bevy components
    if [[ -f "$target_dir/components.rs" ]]; then
        if grep -q "#\[derive.*Component" "$target_dir/components.rs"; then
            log_success "Bevy components found"
        else
            log_warning "No Bevy components found in components.rs"
        fi
    fi
    
    # Check for tests
    if [[ -f "$target_dir/tests.rs" ]]; then
        log_success "Test file found"
    else
        log_warning "Consider adding tests.rs for comprehensive testing"
    fi
    
    if [[ $errors -eq 0 ]]; then
        log_success "Module structure validation passed"
        return 0
    else
        log_error "Module structure validation failed with $errors errors"
        return 1
    fi
}

# Update imports in a file to match new module structure
update_imports() {
    local file_path="$1"
    local old_import="$2"
    local new_import="$3"
    
    if [[ -z "$file_path" || -z "$old_import" || -z "$new_import" ]]; then
        log_error "Usage: update_imports <file_path> <old_import> <new_import>"
        return 1
    fi
    
    if [[ ! -f "$file_path" ]]; then
        log_error "File does not exist: $file_path"
        return 1
    fi
    
    log_step "Updating imports in $file_path"
    log_info "  $old_import → $new_import"
    
    # Create backup
    cp "$file_path" "$file_path.backup"
    
    # Update imports
    sed -i.tmp "s|$old_import|$new_import|g" "$file_path"
    rm "$file_path.tmp" 2>/dev/null || true
    
    # Check if any changes were made
    if ! diff -q "$file_path" "$file_path.backup" > /dev/null 2>&1; then
        log_success "Imports updated successfully"
        rm "$file_path.backup"
    else
        log_warning "No imports were changed"
        rm "$file_path.backup"
    fi
}

# Run comprehensive port validation
validate_port() {
    local target_crate="$1"
    
    if [[ -z "$target_crate" ]]; then
        log_error "Usage: validate_port <target_crate>"
        return 1
    fi
    
    log_step "Running comprehensive port validation for $target_crate"
    
    local crate_dir="$WORKSPACE_ROOT/crates/$target_crate"
    
    if [[ ! -d "$crate_dir" ]]; then
        log_error "Crate directory does not exist: $crate_dir"
        return 1
    fi
    
    # Build check
    log_info "Checking compilation..."
    if cargo check -p "$target_crate" > /dev/null 2>&1; then
        log_success "Compilation check passed"
    else
        log_error "Compilation check failed"
        cargo check -p "$target_crate"
        return 1
    fi
    
    # Test check
    log_info "Running tests..."
    if cargo test -p "$target_crate" > /dev/null 2>&1; then
        log_success "Tests passed"
    else
        log_warning "Tests failed or no tests found"
        cargo test -p "$target_crate"
    fi
    
    # Clippy check
    log_info "Running clippy..."
    if cargo clippy -p "$target_crate" --all-targets --all-features -- -D warnings > /dev/null 2>&1; then
        log_success "Clippy check passed"
    else
        log_warning "Clippy warnings found"
        cargo clippy -p "$target_crate" --all-targets --all-features -- -D warnings
    fi
    
    # Format check
    log_info "Checking formatting..."
    if cargo fmt --package "$target_crate" --check > /dev/null 2>&1; then
        log_success "Formatting check passed"
    else
        log_warning "Formatting issues found"
        cargo fmt --package "$target_crate"
        log_info "Formatting applied"
    fi
    
    log_success "Port validation completed for $target_crate"
}

# Generate integration template for a new module
generate_integration_template() {
    local crate_name="$1"
    local module_name="$2"
    
    if [[ -z "$crate_name" || -z "$module_name" ]]; then
        log_error "Usage: generate_integration_template <crate_name> <module_name>"
        return 1
    fi
    
    log_step "Generating integration template for $crate_name::$module_name"
    
    # Create integration example
    cat << EOF

// Integration template for $crate_name::$module_name
// Add this to your main App configuration:

use $crate_name::$module_name::${module_name^}Plugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add your plugin here:
        .add_plugins(${module_name^}Plugin)
        .run();
}

// If the module has configuration options:
// use $crate_name::$module_name::{${module_name^}Plugin, ${module_name^}Config};
// 
// App::new()
//     .add_plugins(DefaultPlugins)
//     .add_plugins(${module_name^}Plugin::with_config(${module_name^}Config {
//         // your config options here
//     }))
//     .run();

EOF
    
    log_success "Integration template generated"
}

# Check for common porting issues
check_porting_issues() {
    local target_dir="$1"
    
    if [[ -z "$target_dir" ]]; then
        log_error "Usage: check_porting_issues <target_directory>"
        return 1
    fi
    
    if [[ ! -d "$target_dir" ]]; then
        log_error "Directory does not exist: $target_dir"
        return 1
    fi
    
    log_step "Checking for common porting issues in $target_dir"
    
    local issues_found=0
    
    # Check for old Bevy API usage
    if find "$target_dir" -name "*.rs" -exec grep -l "app\.world[^()]" {} \; 2>/dev/null | head -1 > /dev/null; then
        log_warning "Old Bevy API usage found: app.world (should be app.world())"
        find "$target_dir" -name "*.rs" -exec grep -l "app\.world[^()]" {} \; 2>/dev/null
        ((issues_found++))
    fi
    
    # Check for missing Component derives
    local components_without_derive=$(find "$target_dir" -name "*.rs" -exec grep -L "#\[derive.*Component" {} \; 2>/dev/null | \
        xargs grep -l "pub struct.*{" 2>/dev/null || true)
    if [[ -n "$components_without_derive" ]]; then
        log_warning "Potential components without #[derive(Component)] found:"
        echo "$components_without_derive"
        ((issues_found++))
    fi
    
    # Check for circular dependencies
    local amp_imports=$(find "$target_dir" -name "*.rs" -exec grep -h "^use crate::amp_" {} \; 2>/dev/null | wc -l || echo "0")
    if [[ "$amp_imports" -gt 3 ]]; then
        log_warning "High number of cross-crate imports detected ($amp_imports). Check for circular dependencies."
        ((issues_found++))
    fi
    
    # Check for hard-coded paths
    if find "$target_dir" -name "*.rs" -exec grep -l "\"assets/\|\"config/\|\"saves/\"" {} \; 2>/dev/null | head -1 > /dev/null; then
        log_warning "Hard-coded paths found. Consider using configuration resources."
        find "$target_dir" -name "*.rs" -exec grep -l "\"assets/\|\"config/\|\"saves/\"" {} \; 2>/dev/null
        ((issues_found++))
    fi
    
    if [[ $issues_found -eq 0 ]]; then
        log_success "No common porting issues detected"
    else
        log_warning "$issues_found potential issues found. Review and address as needed."
    fi
}

# Display usage information
show_porting_utils_help() {
    echo -e "${BLUE}🛠️  Porting Utilities Help${NC}"
    echo ""
    echo "Available functions:"
    echo ""
    echo -e "${GREEN}create_module_structure <target_crate> <module_name>${NC}"
    echo "  Creates standard module structure based on successful patterns"
    echo ""
    echo -e "${GREEN}validate_module_structure <module_directory>${NC}"
    echo "  Validates module structure against established patterns"
    echo ""
    echo -e "${GREEN}update_imports <file_path> <old_import> <new_import>${NC}"
    echo "  Updates import statements in a file (with backup)"
    echo ""
    echo -e "${GREEN}validate_port <target_crate>${NC}"
    echo "  Runs comprehensive validation (build, test, clippy, fmt)"
    echo ""
    echo -e "${GREEN}generate_integration_template <crate_name> <module_name>${NC}"
    echo "  Generates example integration code"
    echo ""
    echo -e "${GREEN}check_porting_issues <target_directory>${NC}"
    echo "  Checks for common porting problems"
    echo ""
    echo "Usage: source scripts/porting_utils.sh && <function_name> <args>"
}

# If script is executed directly, show help
if [[ "${BASH_SOURCE[0]:-$0}" == "${0}" ]]; then
    show_porting_utils_help
fi
