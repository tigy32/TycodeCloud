#!/bin/bash
set -e

echo "Building Tycode Lambda functions..."

# Install cargo-lambda if not present
if ! command -v cargo-lambda &> /dev/null; then
    echo "Installing cargo-lambda..."
    pip3 install cargo-lambda
fi

# Create output directory
mkdir -p target/lambda/statistics
mkdir -p target/lambda/bugs

# Build statistics Lambda
echo "Building statistics Lambda..."
cargo lambda build --release --arm64 --package statistics

# Build bugs Lambda
echo "Building bugs Lambda..."
cargo lambda build --release --arm64 --package bugs

# Package Lambda functions
echo "Packaging Lambda functions..."
cd target/lambda/release
zip -j ../statistics/bootstrap.zip bootstrap-statistics
mv bootstrap-statistics ../statistics/bootstrap
zip -j ../bugs/bootstrap.zip bootstrap-bugs
mv bootstrap-bugs ../bugs/bootstrap
cd ../../..

echo "Build complete!"
echo "Lambda packages created:"
echo "  - target/lambda/statistics/bootstrap.zip"
echo "  - target/lambda/bugs/bootstrap.zip"
