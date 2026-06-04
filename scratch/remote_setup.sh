#!/bin/bash
set -e

# Cleanup any Docker repository conflicts first
echo "Cleaning up any Docker repository conflicts..."
sudo rm -f /etc/apt/sources.list.d/docker.list
sudo rm -f /etc/apt/keyrings/docker.gpg

echo "Updating apt cache..."
sudo apt-get update -y

echo "Installing required packages..."
sudo apt-get install -y \
    apt-transport-https \
    ca-certificates \
    curl \
    gnupg \
    lsb-release \
    python3-pip \
    ufw \
    nginx \
    certbot \
    python3-certbot-nginx

echo "Installing Docker Engine..."
sudo apt-get install -y \
    docker-ce \
    docker-ce-cli \
    containerd.io \
    docker-compose-plugin

echo "Starting and enabling Docker..."
sudo systemctl start docker
sudo systemctl enable docker

echo "Configuring UFW..."
sudo ufw allow 22/tcp
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
echo "y" | sudo ufw enable

echo "Creating swap file (4GB)..."
if [ ! -f /swapfile ]; then
    sudo fallocate -l 4G /swapfile
    sudo chmod 600 /swapfile
    sudo mkswap /swapfile
    sudo swapon /swapfile
    echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
    echo "Swap created."
else
    echo "Swap file already exists."
fi

echo "Configuring Nginx for iviss-prod.vpn.kivoyo.com..."
cat <<EOF | sudo tee /etc/nginx/sites-available/iviss
server {
    listen 80;
    server_name iviss-prod.vpn.kivoyo.com;

    location / {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_cache_bypass \$http_upgrade;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }

    location /api {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_cache_bypass \$http_upgrade;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }

    location /adminer {
        proxy_pass http://localhost:8081; # Assuming Adminer runs on 8081
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_cache_bypass \$http_upgrade;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF

sudo ln -sf /etc/nginx/sites-available/iviss /etc/nginx/sites-enabled/iviss
sudo rm -f /etc/nginx/sites-enabled/default

echo "Reloading Nginx..."
sudo systemctl reload nginx

echo "Obtaining SSL certificate with Certbot..."
# Check if certificate already exists
if [ ! -d "/etc/letsencrypt/live/iviss-prod.vpn.kivoyo.com" ]; then
    sudo certbot --nginx -d iviss-prod.vpn.kivoyo.com --non-interactive --agree-tos --register-unsafely-without-email
else
    echo "Certificate already exists, attempting renewal if needed..."
    sudo certbot renew
fi

echo "Setup script completed successfully."
