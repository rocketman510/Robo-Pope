#!/bin/bash
TOKEN=$1

pm2 stop index.ts

git pull https://$TOKEN@github.com/rocketman510/Robo-Pope.git main

pm2 start index.ts
