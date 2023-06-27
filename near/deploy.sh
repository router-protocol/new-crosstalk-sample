#!/bin/sh

sh build.sh

cd deployment
npx ts-node scripts/deployPingPong.ts
