#!/usr/bin/env bash

jq "select(.issue == ${1})" < db/ices.jsonl
