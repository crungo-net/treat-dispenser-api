#!/bin/bash
# This script builds a Docker image using BuildKit and loads it into Docker (local daemon).
# Usage: ./build-image-local.sh <image_name>:<tag>
# Example: ./build-image-local.sh nyaa-rss-archiver:dev

set -e
IMAGE_NAME=$1
if [ -z "$IMAGE_NAME" ]; then
  echo "Usage: $0 <image_name>"
  exit 1
fi
CACHE_IMAGE="${IMAGE_NAME}-localbuildcache"

buildctl build \
  --frontend=dockerfile.v0 \
  --import-cache type=registry,ref=${CACHE_IMAGE} \
  --export-cache type=registry,ref=${CACHE_IMAGE},mode=max \
  --local context=. \
  --local dockerfile=. \
  --opt target=runtime \
  --output type=docker,name=${IMAGE_NAME} | docker load

echo "Exporting binary to dist directory..."

buildctl build \
  --frontend=dockerfile.v0 \
  --import-cache type=registry,ref=${CACHE_IMAGE} \
  --export-cache type=registry,ref=${CACHE_IMAGE},mode=max \
  --local context=. \
  --local dockerfile=. \
  --opt target=binary-export \
  --output type=local,dest=./target/docker/