#!/usr/bin/env bash
chmod +x /home/orch/ws
sudo cp ~/orche.service /etc/systemd/system/orche.service
sudo systemctl enable orche
sudo systemctl restart orche