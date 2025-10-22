#!/usr/bin/env bash

# Deps:
# https://pypi.org/project/toml-cli/
# https://github.com/davidrjonas/semver-cli

set -e

# --- Configuration ---

# Define the array of target Rust package directories.
# NOTE: These names must match the directory names.
# IMPORTANT: Order matters for publishing - dependencies must come first!
TARGET_PKGS=(
    "rsx-dominator"  # No internal dependencies - publish first
    "rsx-parser"     # Depends on rsx-dominator
    "rsx-macros"     # Depends on rsx-parser  
    "rustsx"         # Depends on all others - publish last
)

# --- Utility Functions ---

function usage {
    echo "Usage: $0 [FORCE_VERSION]"
    echo ""
    echo "  FORCE_VERSION: If provided, skips interactive prompt and forces all packages to this version."
    echo "                 e.g. $0 1.2.3"
    echo ""
    echo "  If no FORCE_VERSION is provided, an interactive prompt will ask for"
    echo "  (M)ajor, (m)inor, or (p)atch increment."
    exit 1
}

function cleanup_and_exit {
    echo "Something went wrong. Reverting changes."
    # Revert to the last commit to clean up any changes made by the script
    # '|| true' prevents cleanup_and_exit from failing if 'git reset' fails
    git reset --hard HEAD || true
    exit 1
}

# Trap any exit signals (e.g., Ctrl+C) and run the cleanup function
trap cleanup_and_exit INT

# --- Argument Parsing ---
FORCE_VERSION=""
# Check if a specific version was passed as an argument
if [ "$#" -eq 1 ]; then
    # Check if the argument is a valid semver string before proceeding
    if ! semver-cli validate "$1"; then
        echo "Error: '$1' is not a valid semantic version."
        usage
    fi
    FORCE_VERSION="$1"
    echo "Force version '$FORCE_VERSION' detected. Skipping interactive prompt."
elif [ "$#" -gt 1 ]; then
    usage
fi

# --- Pre-flight Checks ---

echo "--- Running Pre-flight Checks ---"

# Check for required dependencies
for cmd in toml semver-cli; do
    if ! command -v "$cmd" &> /dev/null; then
        echo "Error: Required command '$cmd' not found. Please install it."
        exit 1
    fi
done

# Check for uncommitted changes
if [[ -n $(git status --porcelain) ]]; then
    echo "Error: You have uncommitted changes. Please commit or stash them first."
    exit 1
fi

# Check that all target directories exist
for pkg in "${TARGET_PKGS[@]}"; do
    if [ ! -d "$pkg" ]; then
        echo "Error: Target directory '$pkg' not found."
        exit 1
    fi
done

# --- Get and Validate Versions ---

echo "--- Validating Package Versions ---"

# Get the version of the first package in the array as the "source of truth"
FIRST_PKG="${TARGET_PKGS[0]}"
CURRENT_VERSION=$(toml get package.version --toml-path "$FIRST_PKG/Cargo.toml")

# Check all other packages against the first package's version
for pkg in "${TARGET_PKGS[@]}"; do
    PKG_VERSION=$(toml get package.version --toml-path "$pkg/Cargo.toml")
    if [ "$CURRENT_VERSION" != "$PKG_VERSION" ]; then
        echo "Error: Package '$pkg' version ($PKG_VERSION) does not match '$FIRST_PKG' version ($CURRENT_VERSION)."
        exit 1
    fi
    echo "Confirmed version for $pkg is $CURRENT_VERSION."
done

# --- Get Next Version ---

echo "--- Determining Next Version ---"

NEXT_VERSION=""

if [ -n "$FORCE_VERSION" ]; then
    # Use the version passed as an argument
    NEXT_VERSION="$FORCE_VERSION"
    if [ "$NEXT_VERSION" == "$CURRENT_VERSION" ]; then
        echo "Warning: Forced version $NEXT_VERSION is the same as the current version. Proceeding anyway."
    fi
else
    # Interactive prompt for major/minor/patch increment
    read -n 1 -p "What type of publish is this?: (M)ajor, (m)inor, (p)atch: " PUBLISH_TYPE
    echo

    case "$PUBLISH_TYPE" in
        M) NEXT_VERSION=$(semver-cli inc major "$CURRENT_VERSION") ;;
        m) NEXT_VERSION=$(semver-cli inc minor "$CURRENT_VERSION") ;;
        p) NEXT_VERSION=$(semver-cli inc patch "$CURRENT_VERSION") ;;
        *) echo "Error: Unknown upgrade type '$PUBLISH_TYPE'."; exit 1 ;;
    esac
fi

echo "Will perform update from $CURRENT_VERSION to $NEXT_VERSION."

read -p "Ok to proceed? (y/N): " CONFIRM
if [ "$CONFIRM" != "y" ]; then
    echo "Cancelled by user. Exiting."
    exit 1
fi

# --- Update TOML files ---

echo "--- Updating Cargo.toml files ---"

# 1. Loop through all packages and update their 'package.version'
for pkg_dir in "${TARGET_PKGS[@]}"; do
    TOML_FILE="$pkg_dir/Cargo.toml"

    echo "Setting package.version to $NEXT_VERSION in $TOML_FILE"
    toml set package.version "$NEXT_VERSION" --toml-path "$TOML_FILE"
done

# 2. Check for and update internal dependencies
# Outer loop: The package whose Cargo.toml we are modifying (the dependent)
for dependent_pkg_dir in "${TARGET_PKGS[@]}"; do
    DEPENDENT_TOML_FILE="$dependent_pkg_dir/Cargo.toml"

    # Inner loop: The packages that might be dependencies (the dependency)
    for dependency_pkg_dir in "${TARGET_PKGS[@]}"; do
        # Do not check a package against itself
        if [ "$dependent_pkg_dir" == "$dependency_pkg_dir" ]; then
            continue
        fi

        # Get the actual package name of the potential dependency
        # This is the name used in the [dependencies] section
        DEPENDENCY_NAME=$(toml get package.name --toml-path "$dependency_pkg_dir/Cargo.toml")

        # Check if the potential dependency exists in the dependent's dependencies table
        # toml get ... returns 0 (success) if the key exists, and 1 (failure) if not.
        if toml get dependencies."$DEPENDENCY_NAME" --toml-path "$DEPENDENT_TOML_FILE" &> /dev/null; then
            echo "-> Found dependency '$DEPENDENCY_NAME' in '$dependent_pkg_dir'. Updating version..."

            # Use toml set to update the version key for this dependency
            # The dependency version is set to the new package version
            toml set dependencies."$DEPENDENCY_NAME".version "$NEXT_VERSION" --toml-path "$DEPENDENT_TOML_FILE"
        fi
    done
done

echo "--- Committing and Pushing Changes ---"

COMMIT_MESSAGE="Bump version $CURRENT_VERSION -> $NEXT_VERSION"
if [ -n "$FORCE_VERSION" ]; then
    COMMIT_MESSAGE="FORCE Bump version to $NEXT_VERSION (was $CURRENT_VERSION)"
fi

echo "Committing version bump..."
git add .
git commit -am "$COMMIT_MESSAGE"

echo "Pushing changes..."
if ! git push; then
    echo "Git push failed. Please resolve manually or run git reset --hard HEAD."
    exit 1
fi

# --- Final Publish ---

echo "--- Publishing Crates ---"

# Publish each package individually in dependency order
for pkg_dir in "${TARGET_PKGS[@]}"; do
    PKG_NAME=$(toml get package.name --toml-path "$pkg_dir/Cargo.toml")
    echo "Publishing $PKG_NAME..."
    
    if ! cargo +nightly publish -p "$PKG_NAME"; then
        echo "Publishing failed for $PKG_NAME. Please check and resolve manually."
        exit 1
    fi
    
    # Wait a moment between publishes to allow crates.io to propagate
    echo "Waiting for crates.io to propagate..."
    sleep 10
done

echo "✅ Publishing process complete. Version $NEXT_VERSION is now live for all packages!"