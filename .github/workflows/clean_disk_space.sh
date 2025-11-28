#!/usr/bin/env bash
set -euo pipefail

# copyed from:
# https://github.com/apache/flink/blob/master/tools/azure-pipelines/free_disk_space.sh

echo "=============================================================================="
echo "Freeing up disk space on CI system (with parallel optimization)"
echo "=============================================================================="

# Function to log with timestamp
log() {
    echo "[$(date '+%H:%M:%S')] $*"
}

# Function to remove packages sequentially (APT lock prevents parallel execution)
remove_packages_sequential() {
    local patterns=("$@")
    
    # Show largest packages for reference
    log "Listing 20 largest packages for reference:"
    dpkg-query -Wf '${Installed-Size}\t${Package}\n' 2>/dev/null | sort -n | tail -n 20 || true

    log "Starting sequential package removal..."
    
    # Remove packages matching patterns sequentially due to APT lock
    for pattern in "${patterns[@]}"; do
        log "Removing packages matching: $pattern"
        sudo apt-get remove -y "$pattern" 2>/dev/null || true
    done
    
    sudo apt-get autoremove -y
    sudo apt-get clean

    log "All package removals completed"
}

# Function to remove directories in parallel
remove_directories_parallel() {
    local dirs=("$@")
    local pids=()
    
    log "Starting parallel directory removal..."
    
    for dir in "${dirs[@]}"; do
        if [ -d "$dir" ]; then
            {
                log "Removing directory: $dir"
                sudo rm -rf "$dir"
                log "Completed removal of: $dir"
            } &
            pids+=($!)
        else
            log "Directory not found (skipping): $dir"
        fi
    done
    
    # Wait for all directory removals to complete
    for pid in "${pids[@]}"; do
        wait "$pid"
    done
    
    log "All directory removals completed"
}

# Define package patterns to remove
PACKAGE_PATTERNS=(
    '^dotnet-.*'
    '^llvm-.*'
    'php.*'
    '^mongodb-.*'
    '^mysql-.*'
)

# Define individual packages to remove
INDIVIDUAL_PACKAGES=(
    "azure-cli"
    "google-cloud-sdk"
    "hhvm"
    "google-chrome-stable"
    "firefox"
    "powershell"
    "mono-devel"
    "libgl1-mesa-dri"
)

# TODO: skip removing packages by pattern for now to avoid slowing down the pipeline by too much
# # Remove packages sequentially (APT lock prevents parallel execution)
# remove_packages_sequential "${PACKAGE_PATTERNS[@]}"

# Define directories to remove in parallel
DIRECTORIES_TO_REMOVE=(
    "/usr/share/dotnet"
    "/usr/local/graalvm"
    "/usr/local/.ghcup"

    # NOTE: uncomment the followings if more disk space is needed
    # they are commented out to reduce CI time

    # "/usr/local/lib/node_modules"
    # "/usr/local/share/powershell"
    # "/usr/local/share/chromium"
    # "/usr/local/lib/android"
    # "/opt/hostedtoolcache"
    # "/usr/share/swift"
    # "/usr/local/julia*"
)

# Remove large directories in parallel using xargs
log "Starting parallel directory removal using xargs..."
printf '%s\n' "${DIRECTORIES_TO_REMOVE[@]}" | xargs -P 64 -I {} sudo rm -rf "{}"

log "Cleanining /opt directory..."
cd /opt
find . -maxdepth 1 -mindepth 1 '!' -path ./containerd '!' -path ./actionarchivecache '!' -path ./runner '!' -path ./runner-cache -print0 | xargs -0 -r -P 64 -n 1 rm -rf

log "All directory removals completed"

log "Final disk usage:"
df -h

echo "=============================================================================="
log "Disk cleanup completed successfully!"
echo "=============================================================================="