---
name: rhwp-form-fill
description: rhwp CLI 로 HWP/HWPX 서식의 누름틀에 값을 채우고 메일머지 산출물을 만듭니다. fields 조사 → fill-fields 단건 채움(반복 필드 `이름[순번]` 지목) → batch fill(서식 1 + 데이터 N행) → --dry-run/--verify 판정 → sanitize 제출 정리까지 수행합니다. 트리거 — 사용자가 "이 서식/신청서/양식 채워줘", "누름틀에 값 넣어줘", "명단으로 N명분 만들어줘", "메일머지", "서식에 뭘 채워야 하는지 알려줘", "제출용으로 만들어줘" 등을 요청할 때. 판정 규칙 실측은 mydocs/manual/recipes/01·05.
---

# rhwp-form-fill — 서식 채우기·메일머지 Skill

## 목적

누름틀이 있는 서식(`.hwp`/`.hwpx`)에 값을 채워 제출 가능한 산출물을 만든다.
"값이 들어갔다"와 "제출할 수 있다"는 다른 명제다 — 단계마다 기계 판정
(`notFound`/`ambiguous`/`verify`)으로 확인하고, 사람 눈 확인 없이 넘어가지 않는다.

## 바이너리 실행

```bash
cargo build --release        # 최초 1회 또는 소스 변경 후
./target/release/rhwp <명령> [옵션]
```
- 네이티브 실행은 항상 로컬 cargo (Docker 는 WASM 전용).
- 산출물은 `output/` 아래 분리 권장(gitignore 됨). 원본은 어떤 실패에서도 불변이다.

## 요청 → 명령 매핑

| 사용자 요청 | 명령 |
|------------|------|
| "이 서식에 뭘 채워야 해?" | `fields <서식> --json` |
| "값 채워줘" (1건) | `edit fill-fields <서식> --data '{"필드":"값"}' -o <출력> --json` |
| "채우기 전에 미리 확인만" | `edit fill-fields … --dry-run --json` |
| "같은 이름 필드 중 N번째만" | `--data '{"이름[N]":"값"}'` (0 기준 순번, #3476) |
| "명단으로 N명분 만들어줘" (메일머지) | `batch fill --form <서식> --data <행.jsonl\|.csv> --out-dir <폴더> --json` |
| "산출 파일명을 이름으로" | `batch fill … --name-field <필드>` |
| "채워졌는지 재파싱 검증까지" | `--verify` (단건·batch 공통, 차이 시 exit 3) |
| "도장/서명 붙여줘" | `edit insert-image <파일> --image <그림> --page N --x --y … --json` |
| "제출 전 작성자 흔적 지워줘" | `edit sanitize <파일> -o <출력> --json` |
| 누름틀이 없는 표 칸 서식 | `edit set-cell` 축 — rhwp-table-exchange skill 로 전환 |

## 절차 (판단 분기 포함)

### 0단계 — 축 판정: `fields --json` (읽기 전용)

```bash
rhwp fields 신청서.hwp --json | jq '{fieldCount, names:[.fields[].name]}'
```

- `fieldCount: 0` → 누름틀 서식이 아니다. 표 칸 서식이면 `edit set-cell` 축
  (레시피 01 의 축 선택 표 참조)으로 전환하고 이 skill 을 계속 쓰지 않는다.
- `fields[].name` 이 `--data` JSON 의 키다 — **그대로 복사**해 쓴다(오타가 `notFound` 1순위 원인).
- `fields[].memo`/`guide` 는 "어떻게 쓰라"는 사람용 지시문 — 값 생성 시 참고한다.
- `textSecurity.status` 가 `"clean"` 이 아니면 채우기 전에 레시피 04(미검증 문서 점검)를 먼저 밟는다.
- 같은 이름이 여러 번 나오면(총수는 `fields --json` 목록에서 확인) 순번 지목을 준비한다.

### 1단계 — 선검증: `--dry-run`

파일을 쓰지 않고 채움 가능 여부만 판정한다. 실행 명령줄에서 `--dry-run` 하나만
빼면 실제 실행이 되도록 **같은 인자로** 돌린다.

```bash
rhwp edit fill-fields 신청서.hwp --data @row.json --dry-run --json
```

`notFound` 에 이름이 남으면 오타·없는 필드다. 여기서 잡고 진행한다.

### 2단계 — 실행 + 재파싱 검증

```bash
rhwp edit fill-fields 신청서.hwp --data @row.json -o output/작성본.hwp --verify --json
```

통과 판정(셋 다 만족해야 완료):
- `notFound: []` **그리고** `ambiguous: []`
- `verify.identical: true` (저장 직후 재파싱 대조, #3702 — 차이 시 exit 3)
- `filledCount` 가 의도한 개수와 일치

`ambiguous` 가 비어 있지 않으면 같은 이름 필드가 여러 곳이라는 뜻 —
`이름[0]`, `이름[1]` … 순번(0 기준, `fields --json` 목록 순서)으로 재지목한다.
순번 없는 키는 첫 매치만 채우므로 "14개 중 1개만 채운 문서"를 완성본으로 오판하기 쉽다.

### 3단계 — 메일머지: `batch fill` (서식 1 + 데이터 N행)

```bash
rhwp fields 신청서.hwp --json | jq -r '.fields[].name'      # 필드명 먼저
rhwp batch fill --form 신청서.hwp --data 명단.csv \
  --out-dir output/filled --name-field 성명 --json > filled.ndjson
jq -c 'select((.notFound - ["성명"] | length>0) or (.ambiguous|length>0))' filled.ndjson  # 실패 행만
```

- `--data` 는 확장자로 판별: `.jsonl`(한 줄 = JSON 객체 1개) / `.csv`(첫 줄 헤더 = 누름틀 이름).
- 산출은 행마다 1파일. `--name-field` 생략 시 `0001.hwp` 식 순번(최소 4자리).
- 행별 NDJSON 레코드는 단건 `fill-fields` 봉투 + `row`(0 기준) — 판정 규칙 동일.
- `--dry-run`/`--verify`/`--threads <N>` 을 단건과 같은 의미로 받는다.
  `--dry-run` 이어도 `--out-dir` 는 필수다(실행과 같은 명령줄에서 하나만 빼는 규약).

### 4단계(선택) — 제출 마무리

```bash
rhwp edit insert-image output/작성본.hwp --image 직인.png --page 0 --x 50000 --y 70000 -o output/날인본.hwp --json
rhwp edit sanitize output/날인본.hwp -o output/제출본.hwp --json | jq '.removedCount'
```

- `insert-image` 좌표·크기는 **HWPUNIT**(1/7200 inch, A4 세로 = 59528×84188)이다.
  `overflow` 가 비어 있지 않으면 그림이 쪽 밖으로 나갔다는 신호(삽입은 막지 않음 — 판단은 호출자 몫).
- `sanitize` 는 작성자·수정이력·미리보기를 지운다. 두 번째 실행이 `removedCount: 0`
  이면 첫 실행이 실제로 지웠다는 증거다(멱등).

### 파이프라인 게이트로 묶기

사람이 로그를 읽지 않고 기계로 완료를 판정할 때(레시피 01·05 실측 패턴):

```bash
# 단건: verify + notFound + ambiguous 를 한 번에 판정
rhwp edit fill-fields 신청서.hwp --data @row.json -o out.hwp --verify --json \
  | jq -e '.verify.identical and (.notFound|length==0) and (.ambiguous|length==0)' \
  > /dev/null || { echo "채움 실패 — --json 없이 재실행해 상세 확인"; exit 1; }

# batch: 실패 행만 걸러내기 (--name-field 컬럼 "성명" 은 notFound 오탐에서 제외)
jq -es 'map(select(((.notFound - ["성명"])|length>0) or (.ambiguous|length>0)
        or (.verify != null and .verify.identical==false)))
        | if length==0 then "OK" else error("실패 행 \(length)건") end' filled.ndjson
```

요약 줄("N행 중 M 성공")만 보고 넘어가면 어떤 행이 문제였는지 알 수 없다 —
게이트는 반드시 행별 레코드로 판정한다.

## 봉투 읽는 법 (--json · 종료 코드)

- `fill-fields`/`batch fill` 봉투:
  `{"schemaVersion":"1.0","source","dryRun","filledCount","filled":[{name,occurrence,value}],"notFound":[…],"ambiguous":[…],"output"?,"outputFormat"?}`
  (+ batch 는 `row`, `--verify` 시 `verify:{identical,diffCount}`)
  - `notFound` — 문서에 없는 필드 이름(또는 범위 밖 순번). 조용히 무시하지 않는다.
  - `ambiguous` — 순번 없는 이름이 여러 곳에 해당. `{name, matched, total}`.
  - `output`/`outputFormat` 은 실제 저장했을 때만 실린다(`--dry-run` 이면 없음).
  - `outputFormat` 은 입력 형식 보존 규약(#3383): HWPX 입력 → `"hwpx"`, HWP5/HWP3 → `"hwp5"`.
- 종료 코드(#2707): 0 성공 · 1 런타임 실패(파일 없음·쓰기 실패 — **원본 불변, 출력 미생성**) ·
  2 사용법 오류(인자/JSON 오류, 빈 데이터 파일) · 3 `--verify` IR 차이 검출.
- batch 요약(`batch fill: N행 중 …`)은 stderr — stdout 은 NDJSON 뿐이다. 건별 실패는
  레코드로 격리되고 하나라도 실패하면 최종 exit 1.

## 함정 (레시피 01·05 실측)

- **`--data @파일` 은 UTF-8 이어야 한다** — CP949 로 저장하면
  `stream did not contain valid UTF-8` 로 exit 1 (한국어 Windows 기본 인코딩 주의).
- **`--name-field` 컬럼은 매 행 `notFound` 에 뜬다** — 파일명 용도로만 쓴 컬럼도 채울
  필드 후보로 검사되기 때문. **실패가 아니다.** 자동화 게이트에서 그 컬럼명은
  `notFound` 비교 대상에서 미리 빼야 오탐이 없다(레시피 05 실측).
- **`batch fill` 은 stdin 을 읽지 않는다** — 다른 `batch` 하위 명령(파일 목록 stdin)과
  달리 `--form`+`--data` 인자 축이다. 파일 목록을 파이프하면 아무 일도 안 일어난다.
- 데이터 파일에 헤더만 있고 행이 0개면 exit 2 즉시 거부 — 명단 조회가 0건을 돌려준
  상류 문제부터 의심한다.
- `--name-field` 값이 중복이면 나중 행이 먼저 행 산출물을 **덮어쓴다** — 중복 검사는
  이 명령이 해주지 않는다. 파일명 금지 문자는 `_` 치환, 동명은 `_2` 접미.
- `fields` 의 재귀는 표 셀·글상자까지다 — **머리말/꼬리말·각주/미주 안의 필드는 못
  잡는다**(실재 사각지대, cli_commands.md `fields` 절).
- `--verify` 의 `identical: false` 는 문서 구조 특이 케이스 — `export-svg` 육안 확인
  또는 레시피 06 의 `render-diff` 로 정량화한다.
- 페이지 번호는 0부터(`--page 0` = 첫 쪽). 한컴 표기(1부터)와 혼동 주의.

## 실패 신호 → 처방 (요약표)

| 신호 | 원인 | 처방 |
|---|---|---|
| `fieldCount: 0` | 누름틀 서식이 아님(표 칸 양식) | `edit set-cell` 축으로 전환 — rhwp-table-exchange |
| `textSecurity.status` ≠ `"clean"` | 필드 안내문·현재값에 은닉/주입 신호 | 채우기 전에 레시피 04 절차 |
| `notFound` 에 필드명 잔류 | `--data` 키가 실제 이름과 다름(오타·공백) | `fields --json` 의 `name` 을 그대로 복사 |
| `--name-field` 컬럼만 매 행 `notFound` | 정상 동작(파일명 용도 컬럼) | 게이트의 `notFound` 비교에서 그 컬럼 제외 |
| `ambiguous` 비어 있지 않음 | 같은 이름 필드가 여러 곳 | `이름[0]`·`이름[1]` 순번 재지목 |
| `verify.identical: false` (exit 3) | 저장 후 재파싱 값이 요청과 다름 | `export-svg` 육안 확인 → 레시피 06 `render-diff` |
| `오류: --data 에 데이터 행이 없습니다` (exit 2) | 헤더만 있는 빈 CSV/JSONL | 상류 데이터 생성(명단 조회 0건?)부터 확인 |
| `insert-image` 의 `overflow` 비어 있지 않음 | 그림이 쪽 여백을 벗어남 | `--width/--height` 축소 또는 `--x/--y` 조정 |
| `sanitize` 의 `removedCount: 0` | 이미 정리됐거나 메타데이터 없음 | 정상(멱등) — 첫 실행 여부만 확인 |

## 권위 출처

- 명령·옵션·봉투 계약: [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md)
  (`fields` · `edit fill-fields` · `batch fill` · `edit insert-image` · `edit sanitize` · §종료 코드)
- 단건 채움 실측 절차: [`recipes/01_fill_form_and_submit.md`](../../../mydocs/manual/recipes/01_fill_form_and_submit.md)
- 메일머지 실측 절차: [`recipes/05_mail_merge_batch_fill.md`](../../../mydocs/manual/recipes/05_mail_merge_batch_fill.md)
- 반복 필드·`ambiguous` 심화: [`form_filling_guide.md`](../../../mydocs/manual/form_filling_guide.md)
