#!/bin/bash

PORT=5432
CONTAINER_NAME="pg"
TARGET_DIR="/home/gninraw/Codebase/clic/"

# Stop container on exit or interruption
cleanup() {
    echo "
    Cleaning up: stopping container $CONTAINER_NAME"
    sudo docker stop "$CONTAINER_NAME" >/dev/null 2>&1
}
trap cleanup EXIT

# Make sure Docker is running
if ! pgrep -x dockerd >/dev/null; then
    echo "Docker daemon is not running. Starting dockerd..."
    sudo systemctl start docker

    sleep 2

    if ! pgrep -x dockerd >/dev/null; then
        echo "Failed to start Docker daemon. Exiting."
        exit 1
    fi
else
    echo "Docker daemon is already running."
fi

# Check if port is in use
if ss -ltn | grep -q ":$PORT "; then
    echo "Port $PORT is already in use. Assuming PostgreSQL is running."
else
    echo "Port $PORT is free. Starting PostgreSQL container..."
    # Remove --rm to make the container persistent
    sudo docker run --rm --name "$CONTAINER_NAME" -p 5432:5432 \
        -e POSTGRES_PASSWORD=welcome \
        -d postgres:latest

    # Wait a moment to allow the DB to initialize
    sleep 3
fi

if [ -n "$TMUX" ]; then
    tmux new-window -n db "sudo docker logs -f $CONTAINER_NAME; read"
else
    echo "Not in tmux. Skipping docker logs window."
fi

# Change directory and run cargo watch
cd "$TARGET_DIR" || {
    echo "Directory $TARGET_DIR not found"
    exit 1
}

echo "Starting cargo watch..."
cargo watch -q -c -w src/ -w .cargo/ -x run
