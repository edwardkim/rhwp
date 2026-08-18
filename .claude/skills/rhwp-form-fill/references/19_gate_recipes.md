# 19 — jq 게이트 (기계 완료)

사람이 로그를 읽지 않고 파이프라인이 닫히게 한다. 레시피 01·05 실측
패턴이다. 새 도구가 아니다.

## 단건

```bash
rhwp edit fill-fields 신청서.hwp --data @row.json -o out.hwp --verify --json \
  | jq -e '.verify.identical and (.notFound|length==0) and (.ambiguous|length==0)' \
  > /dev/null || { echo "채움 실패 — --json 없이 재실행해 상세 확인"; exit 1; }
```

통과 조건 세 개:

- `verify.identical == true`
- `notFound` 길이 0
- `ambiguous` 길이 0

`filledCount` 를 여기에 넣으려면 의도한 키 개수를 변수로 둔다.

```bash
want=$(jq 'length' row.json)
jq -e --argjson want "$want" \
  '.filledCount == $want and .verify.identical and (.notFound|length==0) and (.ambiguous|length==0)'
```

## batch

```bash
rhwp batch fill --form 신청서.hwp --data 명단.csv \
  --out-dir output/filled --name-field 성명 --json > filled.ndjson

jq -es 'map(select(((.notFound - ["성명"])|length>0) or (.ambiguous|length>0)
        or (.error != null)
        or (.verify != null and .verify.identical==false)))
        | if length==0 then "OK" else error("실패 행 \(length)건") end' filled.ndjson
```

`--name-field` 가 `접수번호` 면 배열에서 그 문자열을 뺀다.

행 수 대조:

```bash
data_rows=$(grep -cve '^$' 명단.jsonl)
nd_rows=$(grep -cve '^$' filled.ndjson)
test "$data_rows" -eq "$nd_rows"
```

실패한 행이 사라지면 이 등식이 깨진다.

## dry-run 게이트

실행과 같은 조건에서 `output` 키가 없어야 한다.

```bash
jq -e '.dryRun == true and (.output == null) and (.notFound|length==0) and (.ambiguous|length==0)'
```

## sanitize 게이트

```bash
c1=$(rhwp edit sanitize in.hwp -o out.hwp --json | jq '.removedCount')
c2=$(rhwp edit sanitize out.hwp -o /tmp/san2.hwp --json | jq '.removedCount')
test "$c2" -eq 0
```

`c1` 이 0 이어도 실패가 아니다(이미 깨끗한 파일).

## 본문 불변 (sanitize)

```bash
jq -n --slurpfile a before.json --slurpfile b after.json \
  '$a[0].text == $b[0].text'
```

## 실패 행만 재시도

```bash
jq -c 'select((.notFound - ["성명"]|length>0) or .error)' filled.ndjson > retry.ndjson
# retry.ndjson 의 row 로 원본 CSV 를 걸러 새 파일을 만든다.
# 새 fill 명령을 발명하지 않는다. 같은 batch fill 을 부분 데이터로.
```

## PowerShell

```powershell
$j = rhwp edit fill-fields 신청서.hwp --data '@row.json' -o out.hwp --verify --json | ConvertFrom-Json
if (-not ($j.verify.identical -and $j.notFound.Count -eq 0 -and $j.ambiguous.Count -eq 0)) {
  throw "채움 실패"
}
```

here-string 을 `gh --body-file -` 에 파이프하지 않는 것과 같이, JSON 도
파일로 다루는 편이 안전하다.
