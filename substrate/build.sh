#!/bin/bash

# Initialize array to store build options
declare -a BUILDS

# Parse command-line arguments
while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        --build)
        # Collect all build options passed
        shift
        while [[ $# -gt 0 && $1 != -* ]]; do
            BUILDS+=("$1")
            shift
        done
        ;;
        *)
        echo "Unknown option: $key"
        exit 1
        ;;
    esac
    shift
done

# Check if any build options were specified
if [ ${#BUILDS[@]} -eq 0 ]; then
    echo "No build options specified. Please specify at least one build option."
    exit 1
fi

# Check if all build options are valid
for build in "${BUILDS[@]}"; do
    if [[ $build != "test-dapp" && $build != "rate-limit" ]]; then
        echo "Invalid value for --build argument: $build. Valid values are: test-dapp, rate-limit"
        exit 1
    fi
done

# Your logic for handling the build process here
for build in "${BUILDS[@]}"; do
    echo "Building $build"

    cd "contracts/$build"
    cargo contract build --release

    # Move the target directory to the root directory if it exists
    # if [ -d "target/ink" ]; then
    #     mkdir -p "../../target"
    #     mkdir -p "../../target/ink"
    #     if [ -d "../../target/ink/$build" ]; then
    #        rm -r  "../../target/ink/$build" 
    #     fi
    #     mv "target/ink" "../../target/ink/$build"
    # fi
    # rm -rf "target"
    cd ../..
done


