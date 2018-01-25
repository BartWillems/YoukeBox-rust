#!/bin/bash

set -e

readonly WORKDIR='/opt/youkebox'
readonly RED='\033[0;31m'
readonly BLUE='\033[0;34m'
readonly YELLOW='\033[1;33m'

# Diesel database migrations
which diesel > /dev/null 2>&1 || {
    echo -e "${RED}diesel not found. Not running database migrations."
    echo -e "${BLUE}Install diesel using '${YELLOW}cargo install diesel_cli --no-default-features --features postgres --root /usr${BLUE}'"
    exit 1
}

cd ${WORKDIR}

diesel migration run > /dev/null 2>&1 || {
    echo -e "${RED}Database migrations failed."
    echo -e "${RED}Please ensure postgresql is running and your .env file is configured."
    echo -e "${RED}You can run the migrations manually with the following command:"
    echo -e "${BLUE}cd ${WORKDIR} && diesel migration run"
    exit 1
}