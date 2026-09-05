# hwp5-* 가족 — HWPX→HWP 저장 계약

권위: `cli_commands.md` §4. oracle = 한컴 저장본, generated = rhwp 저장본.
#178 어댑터로 HWPX 를 열어 HWP 로 썼을 때 한컴이 다르게 여는 경우의 record 축 분석.
문서를 고치지 않는 **진단 전용**이다. 새 hwp5 명령을 발명하지 않는다.

## 왜 자기 라운드트립이 부족한가

`hwp5-roundtrip` 은 HWP5 → IR → HWP5 자기 직렬화 보존이다.
통과해도 한컴이 같은 파일을 연다는 뜻이 아니다.
한컴이 쓰는 record 순·CTRL_DATA·CHAR_SHAPE sentinel 은 oracle 과 비교해야 보인다.

## 명령 표

| 명령 | 용도 |
|---|---|
| `hwp5-inventory` | DocInfo/BodyText record inventory 생성 |
| `hwp5-inventory-diff` | oracle vs generated inventory + contract 힌트 |
| `hwp5-contract-analyze` | record-control contract graph 보고서 |
| `hwp5-ctrl-data-trace` | CTRL_DATA ParameterSet 구조 추적 |
| `hwp5-contract-probe` | MEMO_SHAPE/ID_MAPPINGS + 누락 CTRL_DATA probe |
| `hwp5-table-probe` | TABLE/CTRL_HEADER(Table) field 축 판정 |
| `hwp5-cell-header-probe` | 표 셀 LIST_HEADER/PARA_HEADER 계약 |
| `hwp5-mel-personnel-probe` | mel-001 인원현황 표 축 판정 |
| `hwp5-borderfill-diagonal-probe` | BORDER_FILL 대각선 attr/payload |
| `hwp5-first-para-control-probe` | 첫 문단 control/PARA_TEXT/PARA_CHAR_SHAPE |
| `hwp5-anchor-trace` | 특정 텍스트 주변 raw HWP5 record 추적 |
| `hwp5-char-shape-audit` | CHAR_SHAPE sentinel 차이와 PARA_CHAR_SHAPE 사용 위치 |
| `hwp5-roundtrip` | HWP5 → IR → HWP5 자기 라운드트립 (한컴 호환이 아님) |

## 기본 레시피

```bash
# 1. 한컴이 저장한 정본과 rhwp 가 저장한 산출을 나란히 둔다
rhwp hwp5-inventory-diff oracle.hwp generated.hwp --report hints --focus table

# 2. 표 축이 의심되면 probe
rhwp hwp5-table-probe oracle.hwp generated.hwp --out-dir output/poc/probe/
rhwp hwp5-cell-header-probe oracle.hwp generated.hwp --out-dir output/poc/probe/

# 3. 특정 글자 주변 raw record
rhwp hwp5-anchor-trace generated.hwp --needle "특정텍스트" --section 0

# 4. CHAR_SHAPE sentinel
rhwp hwp5-char-shape-audit oracle.hwp generated.hwp --out output/char-shape-audit.md
```

## 순서 계약

첫 positional 은 항상 oracle(한컴). 둘째는 generated(rhwp).
뒤집으면 힌트가 "한컴이 과다" / "rhwp 가 과다" 를 반대로 말한다.
픽스처와 예제는 파일명에 `oracle` / `generated` 를 박아 순서를 고정한다.

## 성공·실패

hwp5-char-shape-audit: 성공 0, 읽기/쓰기 실패 1, 인자 누락 2.
성공 시 stdout 은 `written: <보고서 경로>` 한 줄.
`--out` 은 이 명령에서 필수다.

inventory-diff 의 차이 보고는 **진단 데이터**다. 차이 자체를 크래시로 승격하지 않는다.

## 레이아웃 사다리에서의 위치

6단. IR(4단) 과 bbox(5단) 다음에 온다.
화면은 같은데 한컴이 안 열리면 1–5 를 건너뛰고 여기로 와도 된다.
화면이 다른데 inventory 만 보면 좌표를 놓친다 — overlay 가 먼저다.

## 하지 않는 것

- Hancom record 를 runtime serializer 에 주입하는 기능을 여기서 설계하지 않는다.
- equivalent 논리 payload 만 보고 canonicalization 을 적용하지 않는다.
- PARA_LINE_SEG bit 0 누적 쪽수를 한컴 PDF 쪽번호와 같다고 가정하지 않는다.
- gym 저장 계약 팩을 이 장의 정본으로 쓰지 않는다.

## 표본 경로

- `samples/basic/KTX.hwp` — 기본 표·본문
- `samples/basic/treatise sample.hwp` — info 표 1개 vs export-tables 3개
- `공문.hwp` — 사용자가 준 경로. 상대 경로 함정
- `편람.hwp` — 대형. --max-chars 없이 export-text 금지 기본
- `oracle.hwp` — 한컴 저장본
- `generated.hwp` — rhwp 저장본
- `source.hwpx` — HWPX 원본

## 대화 예

- 사용자: 두 파일 같아?
  - 명령: `ir-diff --json`
  - 메모: exit 3 = 데이터
- 사용자: 한컴이 안 연다
  - 명령: `hwp5-inventory-diff`
  - 메모: oracle 먼저

## 재시도

같은 실패 봉투가 나오면 플래그를 발명하지 말고 입력을 고친다.
