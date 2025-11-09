#!/bin/bash
set -e

echo "Building Tycode Lambda functions..."

# Install cargo-lambda if not present
if ! command -v cargo-lambda &> /dev/null; then
    echo "Installing cargo-lambda..."
    pip3 install cargo-lambda
fi

# Build statistics Lambda
echo "Building statistics Lambda..."
cargo lambda build --release --arm64 --package statistics

# Build bugs Lambda
echo "Building bugs Lambda..."
cargo lambda build --release --arm64 --package bugs

# Package Lambda functions
echo "Packaging Lambda functions..."

# Package statistics
cd target/lambda/statistics
zip bootstrap.zip bootstrap
echo "Created statistics package: $(pwd)/bootstrap.zip"
cd ../../..

# Package bugs
cd target/lambda/bugs
zip bootstrap.zip bootstrap
echo "Created bugs package: $(pwd)/bootstrap.zip"
cd ../../..

echo ""
echo "Build complete!"
echo "Lambda packages ready for deployment:"
echo "  - target/lambda/statistics/bootstrap.zip"
echo "  - target/lambda/bugs/bootstrap.zip"
