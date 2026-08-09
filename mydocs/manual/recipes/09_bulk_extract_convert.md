---
kind: guide
status: active
canonical: mydocs/manual/recipes/09_bulk_extract_convert.md
last_verified: 2026-08-06
---

# 레시피 9 — 폴더의 문서를 한 번에: 대량 추출·변환 파이프라인

**목표 한 줄**: 문서 수백 건이 든 폴더에서 메타·본문·표 데이터를 한 번에 뽑고 형식을
일괄 변환하되, **실패한 파일이 조용히 사라지지 않게** 오류를 행 단위로 받아 재시도까지
닫는다.

[레시피 5](05_mail_merge_batch_fill.md)가 **쓰기 방향** 대량(서식 하나 → 산출물 N)이라면,
이 레시피는 **읽기·변환 방향** 대량(문서 N → 데이터·변환본)이다. 표면은 전부
`batch` 하나이고, 그중 info·export-text·extract-data·convert 네 축을 쓴다.

번호가 07·08 을 건너뛰는 이유: 07(인계)·08(협업)은 다중 에이전트 협업 계약(#3905,
로드맵 트랙 C)의 설계 승인이 선행이라 예약만 되어 있다. 빈 번호는 의도된 결번이다.

모든 출력은 저장소 `samples/` 표본으로 **실제 실행해서 얻었다**(rhwp v0.8.2 release,
2026-08-06). 지어낸 값은 없다.

`batch` 의 세 규약이 이 레시피 전체를 지배한다(`capabilities` 의 batch 항목이 단일
출처다):

- **입력은 stdin, 한 줄당 파일 경로 하나.** 인자로 늘어놓지 않는다 — 수백 건이면
  인자 길이 한계에 걸린다.
- **stdout 은 순수 NDJSON — 한 줄이 문서 하나의 봉투다.** 사람용 요약(`batch: 5건 중
  4 성공, 1 실패 …`)과 진행·진단 메시지는 stderr 로 간다. 파이프에는 stdout 만 태운다.
- **실패도 봉투다.** 한 파일이 깨져도 파이프는 죽지 않고 그 파일의 오류 레코드를 낸
  뒤 다음 파일로 간다. 전체 종료 코드가 집계를 말한다: *"error 레코드가 하나라도
  있으면 1, 없고 verifyPages 불일치가 있으면 4, verify 차이만 있으면 3, 전부 통과면
  0"* (`capabilities` batch.exitAggregation).

## 1단계 — 파일 목록을 만든다

```bash
find 폴더/ -name '*.hwp' -o -name '*.hwpx' > 목록.txt
```

이 레시피의 실측 목록은 표본 4건 + **일부러 섞은 없는 파일 1건**이다(실패 행이
어떻게 생기는지 보여주기 위해서다):

```
samples/2022년 국립국어원 업무계획.hwp
samples/156636617_240617 2024년 5월 월간 수출입 현황(확정치).hwp
samples/field-01.hwp
samples/hwp3-sample.hwp
samples/없는파일.hwp        ← 실패 시연용
```

## 2단계 — `batch info` 로 스윕 선점검

본문을 뽑기 전에 메타부터 한 바퀴 돈다 — 깨진 파일·암호 문서·형식 오인이 여기서 먼저
드러난다.

```bash
cat 목록.txt | rhwp batch info --json
```

실측 첫 행(지면상 필드 일부 `…`):

```json
{"format":"hwp5","pageCount":35,"paraCount":630,"schemaVersion":"1.0","source":"samples/2022년 국립국어원 업무계획.hwp","title":"2022년 국립국어원 업무계획", …}
```

## 3단계 — `batch export-text` 로 본문을 뽑는다

```bash
cat 목록.txt | rhwp batch export-text --json --threads 4 > 결과.ndjson
```

실측 — 성공 4행과 **실패 1행**(성공 행은 `text` 를 줄였다):

```json
{"pageCount":35,"schemaVersion":"1.0","source":"samples/2022년 국립국어원 업무계획.hwp","text":" \n \n\n\n\n2022년 국립국어원 업무계획\n…"}
{"pageCount":19, …, "source":"samples/156636617_240617 2024년 5월 월간 수출입 현황(확정치).hwp"}
{"pageCount":3,  …, "source":"samples/field-01.hwp"}
{"pageCount":16, …, "source":"samples/hwp3-sample.hwp"}
{"error":"문서를 열 수 없습니다: 지정된 파일을 찾을 수 없습니다. (os error 2)","exitClass":"runtime","schemaVersion":"1.0","source":"samples/없는파일.hwp","untrustedContent":false,"untrustedFields":[]}
```

전체 종료 코드는 **1**(실측) — error 레코드 1건의 집계다. 성공/실패 분리는 `jq` 한
줄이다:

```bash
jq -r 'select(.error) | .source' 결과.ndjson            # 실패 파일만
jq -r 'select(.error|not) | "\(.source)\t\(.pageCount)쪽"' 결과.ndjson
```

## 4단계 — `batch extract-data` 로 숫자·날짜·금액을 수확한다

```bash
cat 목록.txt | rhwp batch extract-data --json --limit 3
```

실측(행별 `counts` 만 발췌):

```json
{"source":"samples/2022년 국립국어원 업무계획.hwp","counts":{"amount":65,"date":29,"number":203},"totalItemCount":297}
{"source":"samples/156636617_240617 2024년 5월 월간 수출입 현황(확정치).hwp","counts":{"amount":0,"date":22,"number":124},"totalItemCount":146}
{"source":"samples/field-01.hwp","counts":{"amount":0,"date":0,"number":0},"totalItemCount":0}
{"source":"samples/hwp3-sample.hwp","counts":{"amount":0,"date":0,"number":11},"totalItemCount":11}
```

> `--limit` 는 **배치 전체가 아니라 문서마다** 적용된다 — 단건 `extract-data --limit`
> 과 같은 의미다(`capabilities` batch.limit 명문). `counts`·`totalItemCount` 는 절단
> **전** 그 문서의 총량이므로, "잘렸는가"는 limit 와 counts 를 비교하면 안다 — 첫
> 행은 297건 중 3건만 실었다.

## 5단계 — `batch convert` 로 형식을 일괄 변환한다

```bash
printf '%s\n' "samples/2025 행정업무운영 편람(최종).hwpx" | rhwp batch convert --out-dir out/bulk --json
```

실측(핵심 필드 발췌 — 387쪽 문서가 428ms):

```json
{"source":"samples/2025 행정업무운영 편람(최종).hwpx","format":"hwp5","output":"out/bulk\\2025 행정업무운영 편람(최종).hwp","bytes":9083392}
```

> 목적지는 `--out-dir` 하나, 이름은 `<입력이름>.hwp` 규칙이다. **이름이 겹치면
> (대소문자만 달라도) 한 건도 쓰지 않고 exit 2** — 절반만 써 놓고 성공한 척하지
> 않는다(`capabilities` batch.output 명문). convert 는 파일을 쓰는 축이라 MCP
> `hwp_batch` 도구에는 노출되지 않고 CLI 전용이다(batch.mcp.excluded).

## 6단계 — 실패 행만 골라 재시도한다

오류 레코드가 행 단위라서 재시도 목록은 후처리 한 줄로 나온다:

```bash
jq -r 'select(.error) | .source' 결과.ndjson > 재시도.txt
cat 재시도.txt | rhwp batch export-text --json
```

재시도 루프는 **오류 부류를 가른 뒤에** 돈다 — 이번 실측의 실패(`os error 2`,
`exitClass: runtime`)는 경로 오타 부류라 목록을 고쳐야 하고, 암호 문서는 단건
`--password` 경로로 뺀다: *"batch 는 credential 을 받지 않는다 — --password 는
usage error"* (`capabilities` batch.authentication).

## 7단계 — 게이트: 숫자가 맞아야 끝난 것이다

```bash
입력=$(wc -l < 목록.txt); 성공=$(jq -s '[.[]|select(.error|not)]|length' 결과.ndjson); 실패=$(jq -s '[.[]|select(.error)]|length' 결과.ndjson)
echo "입력 $입력 = 성공 $성공 + 실패 $실패"
```

실측: **입력 5 = 성공 4 + 실패 1.** 성공+실패=입력이 안 맞으면 어딘가에서 행이
증발한 것이다 — 그때는 결과 파일이 아니라 파이프 중간(head·grep 의 버퍼링 등)을
의심한다.

## 관련

- [레시피 5 — 메일머지](05_mail_merge_batch_fill.md) — 쓰기 방향 대량.
- [레시피 2 — 표 CSV 왕복](02_table_csv_roundtrip.md) — 단건 표 작업의 원형.
- `rhwp capabilities` 의 batch 항목 — stdin·NDJSON·종료 집계·출력 충돌·인증 규약의
  단일 출처.
