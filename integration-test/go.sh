#!/usr/bin/env bash
set -ex

if [ -f ~/.config/modality/license ]; then
    key=$(< ~/.config/modality/license)
    echo "MODALITY_LICENSE_KEY=${key}" > .env
fi

docker compose build
docker compose up --abort-on-container-exit test-collector
