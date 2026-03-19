#!/bin/bash
TOKEN=$1

git fetch --tags --force https://$TOKEN@github.com/rocketman510/Robo-Pope.git
latest=$(git describe --tags --abbrev=0)
git checkout "$latest"
git pull https://$TOKEN@github.com/rocketman510/Robo-Pope.git main
bun install

pm2 restart index.ts
