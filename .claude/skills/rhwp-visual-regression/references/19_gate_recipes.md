# 19 — 게이트 레시피

## 결정성

```bash
rhwp render-diff "$OUT" "$OUT"
test $? -eq 0
```

## 배치 TSV (사람 모드)

```bash
rhwp render-diff --batch "$DIR" -o "$OUT"
awk -F'\t' 'NR>1 && $2!="PASS" && $2!="WARN_TEXTRUN" {print; n++}
             END{exit n?1:0}' "$OUT/geom_inventory.tsv"
```

STRUCT 를 즉시 실패에서 빼려면 `$2=="STRUCT_MISMATCH"` 를 별도 큐로
보낸다. 그 행의 `struct_delta`($11)와 단건 `--json` 의 path 를 읽는다.

## ir-diff 변환

```bash
rhwp ir-diff "$A" "$B" --json
case $? in
  0) echo identical ;;
  3) echo diff; jq .categories ;;
  1) echo load-fail ;;
  2) echo usage ;;
esac
```

## render-diff --json 단건

```bash
rhwp render-diff "$A" "$B" --json > env.json
# exit 3 = regression true. path 를 읽는다
jq -r '.status, .maxDisp, .pages[].topDeltas[].path' env.json
```

stdout 은 순수 JSON/NDJSON. 진행 메시지는 stderr.
