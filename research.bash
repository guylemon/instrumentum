#!/bin/bash
set -e
set -o pipefail

BIBLIOGRAPHY_FILE=bibliography.txt
BULLET_POINTS_FILE=bullet_points.txt
DRAFT_REPORT_CONTEXT_FILE=draft_report_context.jsonl
DRAFT_REPORT_FILE=draft_report.txt
FINAL_REPORT_CONTEXT_FILE=final_report_context.jsonl
FINAL_REPORT_FILE=final_report.txt
QUERY_CONTEXT=query_chat.jsonl
SEARCH_RESULTS_FILE=results_prepped.jsonl
SEARCH_RESULTS_FILE_RAW=results.jsonl
SOURCE_CONTENT_FILE=source_content_file.txt
SOURCE_EVAL_CONTEXT=source_eval_chat.jsonl
SOURCE_RATINGS_FILE=source_ratings.jsonl
SUMMARIZER_CONTEXT_FILE=summarizer_chat.jsonl
TOP_RESULTS=3

rm -f $BIBLIOGRAPHY_FILE
rm -f $BULLET_POINTS_FILE
rm -f $DRAFT_REPORT_CONTEXT_FILE
rm -f $DRAFT_REPORT_FILE
rm -f $FINAL_REPORT_CONTEXT_FILE
rm -f $FINAL_REPORT_FILE
rm -f $QUERY_CONTEXT
rm -f $SEARCH_RESULTS_FILE
rm -f $SEARCH_RESULTS_FILE_RAW
rm -f $SOURCE_CONTENT_FILE
rm -f $SOURCE_EVAL_CONTEXT
rm -f $SOURCE_RATINGS_FILE
rm -f $SUMMARIZER_CONTEXT_FILE

read -p '<Enter your research query> ' topic

# QUERY GENERATION PHASE
llm_prompt \
  --template ./templates/sys_query_agent.txt \
  --var "DATE=$(date)" \
| llm_msg --role system > $QUERY_CONTEXT

echo "Generate 5 targeted, high-value Google search queries to gather comprehensive information on the following research question: $topic" | llm_msg >> $QUERY_CONTEXT

llm_generate --provider xai --context $QUERY_CONTEXT

output=$(tail -1 $QUERY_CONTEXT | llm_display)
if ! echo $output | jq -e 'type == "array" and length == 5' > /dev/null 2>&1; then
  echo "Error: search queries must be valid JSON and have length 5" &>2
  exit 1
fi

# WEB SEARCH 
# Clear results
echo '' > $SEARCH_RESULTS_FILE_RAW
mapfile -t queries < <(echo $output | jq -c -r '.[]')
for q in "${queries[@]}"; do
  echo "Searching for query: $q"
  echo "$q" | websearch >> $SEARCH_RESULTS_FILE_RAW
  sleep 1 
done
jq -s -c 'add | .[]' $SEARCH_RESULTS_FILE_RAW > $SEARCH_RESULTS_FILE

# EVALUATION
while IFS= read -r line || [[ -n "$line" ]]; do
  # Extract fields
  url=$(echo "$line" | jq -r '.url // empty')
  title=$(echo "$line" | jq -r '.title // empty')
  content=$(echo "$line" | jq -r '.content // empty')

  # URL must be non-empty
  [[ -z "$url" ]] && continue


  # Build context
  cat ./templates/sys_source_eval_agent.txt | llm_msg --role system > $SOURCE_EVAL_CONTEXT
  llm_prompt \
    --template ./templates/user_source_eval_agent.txt \
    --var "TOPIC=$topic" \
    --var "URL=$url" \
    --var "TITLE=$title" \
    --var "SNIPPET=$content" \
    --var "MARKDOWN=$content" \
  | llm_msg >> $SOURCE_EVAL_CONTEXT

  # Generate
  llm_rating="$(cat $SOURCE_EVAL_CONTEXT | llm_generate --provider xai | llm_display)"
  echo "$llm_rating"

  # validate result
  if ! echo $llm_rating | jq -e 'type == "object"' > /dev/null 2>&1; then
    echo "Error: relevance scores must be valid JSON" &>2
    exit 1
  fi

  # Add result to line in ratings file
  jq -n --arg url "$url" --arg title "$title" --argjson rating "$llm_rating" '{url: $url, title: $title, rating: $rating}' | jq -c >> $SOURCE_RATINGS_FILE
done < "$SEARCH_RESULTS_FILE"

# SCORING
jq -s 'map(.rating) | map(.relevance.score * 0.4 + (.credibility.score + .timeliness.score + .depth.score) * 0.2) | 
  [range(length) as $i | {idx: $i, score: .[$i]}] | sort_by(.score) | reverse | .[:'"$TOP_RESULTS"'] | map(.idx)' \
  $SOURCE_RATINGS_FILE > top_indices.json

echo '' > $SOURCE_CONTENT_FILE
top_indices=$(jq -r '.[]' top_indices.json)
for idx in $top_indices; do
  url="$(jq -s -r ".[$idx].url" $SOURCE_RATINGS_FILE)"
  title="$(jq -s -r ".[$idx].title" $SOURCE_RATINGS_FILE)"
  markdown="$(echo "$url" | webfetch --provider tavily)"

  # Markdown must not be empty
  [[ -z "$markdown" ]] && continue

  # Use xml tags to clarify the summarization agent prompt
  cat << EOF >> "$SOURCE_CONTENT_FILE"
<source id="$idx">
<title>${title}</title>
<url>${url}</url>
<content>
${markdown}
</content>
</source>

EOF

echo "- [$idx]: [$title]($url)" >> $BIBLIOGRAPHY_FILE
done

# GENERATE BULLET POINTS FROM SOURCES
sources="$(< "$SOURCE_CONTENT_FILE")"
cat ./templates/sys_summarizer_agent.txt | llm_msg --role system > $SUMMARIZER_CONTEXT_FILE
llm_prompt --template ./templates/user_summarizer_agent.txt \
  --var TOPIC="$topic" \
  --var SOURCES="$sources" \
| llm_msg --role user >> $SUMMARIZER_CONTEXT_FILE

cat summarizer_chat.jsonl | llm_generate --provider xai | llm_display >> $BULLET_POINTS_FILE

# WRITE DRAFT REPORT
cat ./templates/sys_report_writing_agent.txt | llm_msg --role system > $DRAFT_REPORT_CONTEXT_FILE
llm_prompt --template ./templates/user_report_write_agent.txt \
  --var TOPIC="$topic" \
  --var BULLETED_LIST="$(cat $BULLET_POINTS_FILE)" \
| llm_msg --role user >> $DRAFT_REPORT_CONTEXT_FILE

cat draft_report_context.jsonl | llm_generate --provider xai | llm_display > $DRAFT_REPORT_FILE

# Review the REPORT
cat ./templates/sys_report_editor_agent.txt | llm_msg --role system > $FINAL_REPORT_CONTEXT_FILE
llm_prompt --template ./templates/user_report_editor_agent.txt \
  --var TOPIC="$topic" \
  --var BULLETED_LIST="$(cat $BULLET_POINTS_FILE)" \
  --var DRAFT_REPORT="$(cat $DRAFT_REPORT_FILE)" \
| llm_msg --role user >> $FINAL_REPORT_CONTEXT_FILE

cat final_report_context.jsonl | llm_generate --provider xai | llm_display > $FINAL_REPORT_FILE

# Append bibliography from sources (created by script)
echo -e "\n\nBIBLIOGRAPHY" >> $FINAL_REPORT_FILE
echo -e "------------\n" >> $FINAL_REPORT_FILE
cat $BIBLIOGRAPHY_FILE >> $FINAL_REPORT_FILE
