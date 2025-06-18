#!/bin/bash

SCRIPT_DIR=$(dirname "$0")

rm "manifest.yaml"
LIST=$(find "$SCRIPT_DIR" -type f \( -name "*.yaml" -o -name "*.yml" \) -exec echo "{}" \;)
for FILE in $LIST; do
    DESCRIPTION=$(yq '.description' "$FILE")
    #  echo $FILE - $DESCRIPTION
    {
        echo "- name: $(basename "$FILE" .yaml)"
        echo "  description: $DESCRIPTION"
        echo "  md5sum: $(md5sum "$FILE" | awk '{print $1}')"
    } >> "manifest.yaml"
done
# echo "" >> "manifest.yaml
