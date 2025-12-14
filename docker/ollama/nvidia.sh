#!/usr/bin/env sh

# NVidia Container Toolkit Setup
# To be run on the Docker host to enable GPU support for Ollama container
#
# This script installs the NVidia Container Toolkit and configures Docker
# to use the NVidia runtime for GPU-accelerated containers.
#
# Requirements: NVidia GPU driver must be installed on the host

set -e

echo "Installing NVidia Container Toolkit..."

# Configure the NVidia repository
echo "Configuring NVidia repository..."
curl -fsSL https://nvidia.github.io/libnvidia-container/gpgkey | \
	sudo gpg --dearmor -o /usr/share/keyrings/nvidia-container-toolkit-keyring.gpg

curl -fsSL https://nvidia.github.io/libnvidia-container/stable/deb/nvidia-container-toolkit.list | \
	sed 's#deb https://#deb [signed-by=/usr/share/keyrings/nvidia-container-toolkit-keyring.gpg] https://#g' | \
	sudo tee /etc/apt/sources.list.d/nvidia-container-toolkit.list > /dev/null

# Install the NVidia Container Toolkit
echo "Installing NVidia Container Toolkit packages..."
sudo apt-get update
sudo apt-get install -y nvidia-container-toolkit

# Configure Docker to use the NVidia runtime
echo "Configuring Docker runtime..."
sudo nvidia-ctk runtime configure --runtime=docker

# Restart Docker to apply changes
echo "Restarting Docker..."
sudo systemctl restart docker

echo "Done! NVidia GPU support is now enabled for Docker containers."
