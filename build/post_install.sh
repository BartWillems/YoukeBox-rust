#!/bin/bash

set -e

readonly WORKDIR='/opt/youkebox'

which diesel > /dev/null 2>&1 || {
    echo 'diesel not found. Not running database migrations.'
    echo 'Install diesel using "cargo install diesel".'
    exit 1
}

cd ${WORKDIR}

diesel migration run > /dev/null 2>&1 || {
    echo 'Database migrations failed.'
    echo 'Please ensure postgresql is running and your .env file is configured.'
    echo 'You can run the migrations manually with the following command:'
    echo "cd ${WORKDIR} && diesel migration run"
    exit 1
}