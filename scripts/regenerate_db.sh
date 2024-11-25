#!/bin/bash

SCRIPT_DIR=$(dirname "$0")
SELECTIA_DIR=$(realpath "$SCRIPT_DIR/../selectia")

cd "$SELECTIA_DIR"
rm ../selectia.db
touch ../selectia.db
DATABASE_URL=sqlite:../selectia.db sqlx migrate run
cd -
