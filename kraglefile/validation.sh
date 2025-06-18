#!/bin/bash

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")

# pip install pre-commit
# pre-commit autoupdate --repo https://github.com/pre-commit/pre-commit-hooks

LIST=$(find "$SCRIPT_DIR" -type f \( -name "*.json" -o -name "*.yaml" -o -name "*.yml" \) -exec echo "{}" \;)
for FILE in $LIST; do
    DIR_TMP=$(mktemp -d)

    cargo run -r -- import "" "$DIR_TMP"
$FILE
    # if [ -f "$DIR_TMP/.pre-commit-config.yaml" ]; then
    #     cd $DIR_TMP
    #     git init
    #     git add .
    #     git config user.email "you@example.com"
    #     git config user.name "Your Name"
    #     git commit -m "Initial commit"
    #     pre-commit run --all-files -v
    #     cd -
    # fi

    cargo run -r -- validate "$FILE" "$DIR_TMP"

    rm -rf "$DIR_TMP"
done
