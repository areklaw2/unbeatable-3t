image := "unbeatable-3t:latest"
container := "u3t"
platform := "linux/arm64"
port := "8080"

# List available recipes
default:
    @just --list

# Build the arm64 Docker image
build:
    docker build --platform {{platform}} -t {{image}} .

# Run the container (Ctrl+C to stop)
run:
    docker run --rm --name {{container}} -p {{port}}:{{port}} {{image}}

# Tail container logs
logs:
    docker logs -f {{container}}

# Stop the running container
stop:
    docker stop {{container}}
