#!/bin/bash

set -e

./deploy-localnet.sh
cd interface
npm i
npm run build
npm run test || command_failed=1
if [ ${command_failed:-0} -eq 1 ]
then
 echo "Test run failed to exit gracefully"
fi
# npm run lint
cd ..
./cleanup.sh