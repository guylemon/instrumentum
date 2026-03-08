#!/bin/bash
set -e

rm chat.jsonl

while true; do
  read -p '> ' prompt
  echo $prompt | llm_msg >> chat.jsonl
  cat chat.jsonl | llm_generate >> chat.jsonl
  echo ""
  tail -1 chat.jsonl
  echo ""
done
