---
name: rhwp-bulk-pipeline
description: 폴더의 HWP/HWPX 문서 수백 건을 rhwp batch 로 한 번에 처리합니다. batch info(메타 스윕)/export-text(본문)/extract-data(날짜·금액·수량)/convert(형식 변환)/fill(메일머지) — stdin 목록 → NDJSON 스트림, 실패 행 격리·jq 재시도, 입력 N=성공+실패 게이트까지 닫습니다. 트리거 — 사용자가 "폴더 전체를 텍스트로/한꺼번에 변환", "여러 hwp 대량 처리/코퍼스 추출", "아카이브 전역 검색", "서식 하나에 여러 명 데이터 채워(메일머지)", "rhwp batch" 등을 요청할 때.
---

# rhwp-bulk-pipeline — 폴더 대량 처리 Skill

## 목적

문서 N건이 든 폴더에서 메타·본문·표·데이터를 한 번에 뽑고 형식을 일괄 변환하되,
**실패한 파일이 조용히 사라지지 않게** 오류를 행 단위로 받아 재시도와 수량 게이트까지
닫는다. 표면은 전부 `batch` 하나다.

권위 출처: [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md)
(§batch) + `rhwp capabilities` 의 batch 항목(stdin·NDJSON·종료 집계·출력 충돌·인증
규약의 단일 출처). 절차의 실측 원형은 레시피 9(PR #4182)·레시피 5.

## 바이너리 실행

```bash
cargo build --release        # 최초 1회 또는 소스 변경 후
./target/release/rhwp batch <축> --json [옵션] < 목록.txt
```
(공통 규약은 [rhwp-cli skill](../rhwp-cli/SKILL.md) 참조)

## batch 의 세 규약 — 이 스킬 전체를 지배한다

1. **입력은 stdin, 한 줄당 파일 경로 하나.** 인자로 늘어놓지 않는다 — 수백 건이면
   인자 길이 한계에 걸린다. (`batch fill` 만 예외 — 아래 매핑 표)
2. **stdout 은 순수 NDJSON — 한 줄이 문서 하나의 봉투다.** 사람용 요약(`batch: N건 중
   …`)과 진행·진단 메시지는 stderr 로 간다. 파이프에는 stdout 만 태운다.
3. **실패도 봉투다.** 한 파일이 깨져도 파이프는 죽지 않고 그 파일의 오류 레코드
   `{"schemaVersion":"1.0","source","error","exitClass":"runtime"}` 를 낸 뒤 다음
   파일로 간다. 출력 순서는 병렬(`--threads`, 기본 CPU 코어 수)에서도 입력 순서 보존.

## 요청 → 명령 매핑

| 사용자 요청 | 명령 |
|------------|------|
| "폴더 문서들 규모/형식 먼저 훑어줘" | `batch info --json < 목록.txt` |
| "본문 전부 뽑아줘" | `batch export-text --json [--threads N] < 목록.txt` |
| "개요/조문 구조 일괄" | `batch export-structure --json [--mode auto\|outline\|clause] < 목록.txt` |
| "표 전부 수확" | `batch export-tables --json < 목록.txt` |
| "서식 템플릿 일괄 조사" | `batch fields --json < 목록.txt` |
| "아카이브 전역 검색" | `batch search --query <검색어> --json < 목록.txt` (`--query` 필수) |
| "날짜·금액·수량 일괄 수확" | `batch extract-data --json [--kind …] [--limit N] < 목록.txt` |
| "편집 가능한 HWP5 로 일괄 변환" | `batch convert --out-dir <폴더> [--verify] [--verify-pages] --json < 목록.txt` |
| "서식 1개에 여러 행 채워(메일머지)" | `batch fill --form <서식> --data <행.jsonl\|행.csv> --out-dir <폴더> --json [--name-field <필드>] [--verify] [--dry-run]` |

## 절차 — 목록 → 선점검 → 본작업 → 재시도 → 게이트

```bash
# 1. 파일 목록 (한 줄당 경로 하나)
find 폴더/ -name '*.hwp' -o -name '*.hwpx' > 목록.txt

# 2. batch info 로 스윕 선점검 — 깨진 파일·암호 문서·형식 오인이 먼저 드러난다
cat 목록.txt | rhwp batch info --json > meta.ndjson

# 3. 본작업 — stdout 만 파일로 태운다
cat 목록.txt | rhwp batch export-text --json --threads 4 > 결과.ndjson

# 4. 성공/실패 분리는 jq 한 줄
jq -r 'select(.error) | .source' 결과.ndjson              # 실패 파일만
jq -r 'select(.error|not) | "\(.source)\t\(.pageCount)쪽"' 결과.ndjson

# 5. 실패 행만 골라 재시도 — 오류 부류를 가른 뒤에 돈다
jq -r 'select(.error) | .source' 결과.ndjson > 재시도.txt
cat 재시도.txt | rhwp batch export-text --json

# 6. 게이트: 숫자가 맞아야 끝난 것이다
입력=$(wc -l < 목록.txt)
성공=$(jq -s '[.[]|select(.error|not)]|length' 결과.ndjson)
실패=$(jq -s '[.[]|select(.error)]|length' 결과.ndjson)
echo "입력 $입력 = 성공 $성공 + 실패 $실패"     # 안 맞으면 행이 증발한 것
```

재시도 전에 오류 부류를 가른다 — `os error 2` 부류는 경로를 고쳐야 하고, 암호 문서는
단건 `--password` 경로로 뺀다(batch 는 credential 을 받지 않는다).

## 봉투·종료 코드 규약

- **exit 집계** (capabilities batch.exitAggregation): error 레코드가 하나라도 있으면
  **1**, 없고 `--verify-pages` 불일치가 있으면 **4**, `--verify` 차이만 있으면 **3**,
  전부 통과면 **0**. 성공 4건+실패 1건이면 exit 1 이 정상이다 — 종료 코드는 집계이고,
  행별 판정은 NDJSON 봉투가 한다.
- 성공 레코드는 단건 명령의 `--json` 봉투와 **같은 스키마**다(`batch info` = `info
  --json`, `batch search` = `search --json` 등) — 단건/배치를 같은 소비 코드로 읽는다.
  `batch fill` 은 여기에 `row`(0 기준 행 번호)가 붙는다.
- 실패 레코드 실측 예(레시피 9 — 일부러 섞은 없는 파일 1건):
  ```json
  {"error":"문서를 열 수 없습니다: 지정된 파일을 찾을 수 없습니다. (os error 2)","exitClass":"runtime","schemaVersion":"1.0","source":"samples/없는파일.hwp","untrustedContent":false,"untrustedFields":[]}
  ```
- `batch search` 는 파일당 매치 1,000건 상한(스트림 팽창 방지), 대소문자 구분.

### 축별 전용 플래그 (공용은 `--json`·`--threads`)

| 플래그 | 소속 축 | 비고 |
|--------|--------|------|
| `--mode auto\|outline\|clause` | export-structure | 기본 auto |
| `--query <검색어>` | search | **필수** — 없으면 exit 2 |
| `--kind` · `--limit` | extract-data | `--limit` 는 문서마다 적용 |
| `--out-dir <폴더>` | convert · fill | 둘 다 필수 |
| `--verify` | convert · fill | convert 는 `--verify-pages` 도 |
| `--form` · `--data` · `--name-field` · `--dry-run` | fill | `--name-field` 생략 시 1 기준 순번(최소 4자리), 파일명 금지 문자는 `_` 치환, 이름 겹치면 `_2` |

## 함정 (실측)

- **batch 는 전역 인증 옵션(`--password` 등)을 지원하지 않는다** — 함께 주면 입력을
  소비하지 않고 exit 2. 암호 문서는 단건 명령으로 뺀다.
- **`batch convert` 는 쓰기 전에 모든 산출 이름을 예약한다** — 같은 이름은 물론
  대소문자만 달라도 충돌로 exit 2, 산출 파일을 **하나도 쓰지 않는다**(절반만 써 놓고
  성공한 척하지 않는다). CLI 전용 쓰기 축이라 MCP `hwp_batch` 에는 없다.
- **`batch extract-data` 의 `--limit` 는 배치 전체가 아니라 문서마다** 적용된다 —
  단건 `extract-data --limit` 과 같은 의미다. `counts`·`totalItemCount` 는 절단 **전**
  총량이므로 "잘렸는가"는 limit 와 counts 비교로 안다.
- **`batch extract-data` 축은 cli_commands.md §batch 표면 목록에 아직 없다** —
  `rhwp capabilities` 의 batch.subcommands 와 레시피 9가 실측 근거다(존재는
  `src/main.rs` 디스패치로 확인됨).
- **`batch fill` 은 입력 축 자체가 다르다** — stdin 파일 목록이 아니라 서식 1개
  (`--form`)+데이터 1개(`--data`)를 받고, 산출은 데이터 행 수만큼이다. `--dry-run`
  에도 `--out-dir` 는 필수(선검증이 실행 명령줄에서 `--dry-run` 하나만 빼면 되도록).
- `--out-dir` 값은 다음 플래그가 될 수 없다 — `-` 로 시작하는 폴더는 `./-결과` 로 명시.
- **입력 N ≠ 성공+실패면 결과 파일이 아니라 파이프 중간을 의심한다** — head·grep 의
  버퍼링 등에서 행이 증발한다.
- `--data` 파일은 UTF-8 이어야 한다 — CP949 저장 시 `stream did not contain valid
  UTF-8` 로 exit 1 (`edit fill-fields` §명문).

## 상세 레퍼런스

- batch 전체 옵션: [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md) §batch
- 파이프라인 시나리오(선별→추출·RAG 청킹·실패 처리): [`mydocs/manual/cli_json_pipeline_guide.md`](../../../mydocs/manual/cli_json_pipeline_guide.md)
- 읽기·변환 방향 대량(레시피 9): `mydocs/manual/recipes/09_bulk_extract_convert.md` — PR #4182 머지 후 유효
- 쓰기 방향 대량(메일머지): [`recipes/05_mail_merge_batch_fill.md`](../../../mydocs/manual/recipes/05_mail_merge_batch_fill.md)
- 단건 표 작업의 원형: [`recipes/02_table_csv_roundtrip.md`](../../../mydocs/manual/recipes/02_table_csv_roundtrip.md)
