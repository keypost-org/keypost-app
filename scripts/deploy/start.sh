#!/bin/bash

set -e

echo "Running migrations..."
diesel migration run

echo "Starting keypost-app..."
./keypost-app