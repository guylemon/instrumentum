#!/bin/bash
set -e

rm -f chat.jsonl

while true; do
  read -p '> ' prompt
  echo $prompt | llm_msg >> chat.jsonl
  cat chat.jsonl | llm_generate --provider xai >> chat.jsonl
  echo ""
  tail -1 chat.jsonl | llm_display
  echo ""
done
