#!/usr/bin/env bash

for l in $(cat db/duplicates.jsonl); do
  dup=$(echo "${l}" | jq '.duplicate')
  of=$(echo "${l}" | jq '.of')
  printf -- "---------------------------------------------------\n"
  jq "select(.issue == ${dup})" < db/ices.jsonl
  jq "select(.issue == ${of})" < db/ices.jsonl
done
