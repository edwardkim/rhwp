---
kind: investigation
status: active
canonical: gym/packs/serialization/README.md
last_verified: 2026-08-18
---

# serialization pack 확장 작업 노트

이 문서는 PR #5232 (`feat/gym-serialization-expand`) 를 키운 작업 기록이다.
규범 문서는 [gym/packs/serialization/README.md](../../../gym/packs/serialization/README.md)
다. 예외·가장자리는
[gym_serialization_exceptions.md](gym_serialization_exceptions.md) 에 모은다.

## 무엇을 했는가

serialization pack은 형식 왕복 축인데 과제가 얇았다. 기존 PR은 SR09–SR12
네 건만 더했다(약 259 insertions). 구조대 초안 SR13–SR24 가
`tmp-gym-rescue/serialization/` 에 남아 있었고, 그 과제·기준풀이를
`gym/packs/serialization/` 으로 옮긴 뒤 같은 계약으로 SR25–SR56 을 더했다.

건드리지 않은 것:

- 새 CLI, 새 pack, T07 복제
- `profiles/` · `gym/README.md` · `gym/PARK.md` · `gym/core/checks.py`
- 다른 pack 의 과제 ID
- `cargo fmt --all` (JSON·문서·테스트만 바꿨다)
- `pack.json` 의 `runner` 신원. 요구 명령만 `convert` · `export-hwpx` ·
  `export-pdf` · `extract-pages` 를 명시했다.

## 왜 이 두께인가

왕복 축은 "한 명령 × 한 표본" 으로 닫히지 않는다. 같은 `convert` 라도

- 입력이 HWPX 인가, 이미 편집 가능한 HWP5 인가
- 읽는 필드가 `format` 인가 `wasDistribution` 인가
- `--verify` 인가 `--verify-pages` 인가
- 산출물이 실제로 `info` 로 열리는가

가 다른 계약이다. `extract-pages` 는 쪽 수(1/2/3/4쪽 표본)와 필드
(`pagesBefore` / `pagesAfter` / `paragraphsKept` / `paragraphsRemoved`) 가
갈라지고, `export-pdf` 는 `format` / `backend` / `pageCount` /
`renderedCount` 가 갈라진다. 과제를 합치면 에이전트가 힌트 한 줄을
외워 모든 왕복을 통과한다.

## 과제 계보

### 기존 (devel)

| ID | 명령 | 요지 |
|---|---|---|
| SR01 | `export-hwpx` + `ir-diff` | HWPX 변환 자기검증 |
| SR02 | `export-hml` | HML 원본만 왕복 |
| SR03 | `export-doclang` | 손실 계수 |
| SR04 | `export-markdown` | 쪽수 |
| SR05 | `export-ir-schema` | dialect |
| SR06 | `export-hwpx` + `ir-diff` | 차이 건수 |
| SR07 | `info` | HWPX 원본 조사 |
| SR08 | `export-markdown` | HWPX 마크다운 |

### 첫 확장 (SR09–SR12)

| ID | 명령 | 요지 |
|---|---|---|
| SR09 | `convert` | HWPX → HWP5 `format` |
| SR10 | `extract-pages` | 첫 쪽 `pagesAfter` |
| SR11 | `export-pdf` | `pageCount` |
| SR12 | `ir-diff` | 기존 pic2 쌍 `identical` |

### 구조대 (SR13–SR24)

구조대 초안은 이미 라이브 오라클 형식이었다. 고친 것 없이 복사했다.

| ID | 명령 | 요지 |
|---|---|---|
| SR13 | `convert` | 편집 가능본 `wasDistribution` |
| SR14 | `extract-pages` | 1–2쪽 `pagesBefore` |
| SR15 | `export-pdf` | `backend` |
| SR16 | `ir-diff` | 자기대조 |
| SR17 | `convert --verify` | `verify.identical` |
| SR18 | `extract-pages` | 둘째 쪽 `paragraphsKept` |
| SR19 | `export-pdf` | 그림 문서 `pageCount` |
| SR20 | `ir-diff` | pic2 쌍 `diffCount` |
| SR21 | `convert --verify-pages` | `verifyPages.identical` |
| SR22 | `extract-pages` | 중첩 셀 `pagesBefore` |
| SR23 | `export-pdf` | HWPX `renderedCount` |
| SR24 | `ir-diff` | 다른 두 문서 `identical` |

### 후속 (SR25–SR56)

같은 연산자·같은 힌트 문체를 유지하고 표본과 필드만 갈랐다. 새 연산자를
만들지 않았다.

- convert: SR25, SR26, SR27, SR48, SR56
- extract-pages: SR28, SR29, SR30, SR31, SR54
- export-pdf: SR32, SR33, SR34, SR35, SR49, SR50
- ir-diff: SR36, SR37, SR38, SR39, SR53
- export-hwpx: SR40, SR41, SR42, SR51
- doclang/markdown/info: SR43, SR44, SR45, SR46, SR47, SR52, SR55

## 설계 규칙 (이 확장이 지킨 것)

1. **라이브 오라클.** 쪽수·차이 건수·백엔드·버전 문자열을 과제 JSON에
   숫자/문자열로 박제하지 않는다. `answer_eq` 가 재계산한다.
2. **지목 연산자.** `deep_contains` 를 쓰지 않는다. `value_eq` ·
   `answer_eq` · `file_exists` · `differs_from_input` · `value_ge` 만
   쓴다.
3. **판별력.** 산출물이 있는 과제는 `differs_from_input` 과 `minBytes` 를
   같이 둔다. 이름만 바꾼 복사를 거절한다.
4. **exit 계약.** `ir-diff` / `--verify` 는 `expect_exits: [0, 3]`,
   `--verify-pages` 는 `[0, 4]`. 기준풀이의 `allowExits` 와 맞춘다.
5. **1 기준 명시.** `extract-pages` 과제는 instructions 에 `--from/--to`
   가 1 기준이라고 적는다.
6. **기존 표본만.** 새 fixture를 만들지 않는다. `samples/` 에 이미 있는
   작은 파일만 고른다.
7. **ID 전역 고유.** `SR*` 접두사는 이 pack 만 쓴다. 다른 pack 과
   충돌하지 않는지 `audit.py` 가 확인한다.
8. **기준풀이 짝.** 과제 파일 이름과 reference 파일 이름이 같다. id
   필드도 같다.

## 표본 선택 이유

| 표본 | 이유 |
|---|---|
| `para-001.hwp` | 작은 문단 문서. convert/pdf/hwpx 기본축 |
| `table-001.hwp` | 표가 있어 DocLang 손실·IR 차이가 의미 있다 |
| `exam-kor-1p/2p/3p/4p.hwp` | 쪽 수가 이름에 드러나 extract-pages 축을 가른다 |
| `pic2.hwp` + `pic2.hwpx` | 저장소에 이미 있는 짝 파일 |
| `pic2-2018.hwp` | 같은 그림의 다른 연도 저장본. 차이 규모용 |
| `hwpx_sample2.hwpx` + `.hwp` | 다른 짝 파일. pic2 한 쌍만 보면 과적합 |
| `hwpx/143E433F503322BD33.hwpx` | 기존 SR07/SR08 이 쓰던 HWPX 원본 |
| `basic/issue2007_…42065.hwp` | 중첩 셀·페이지네이션. 재조판 위험이 드러난다 |
| `hml/formatting_table.hml` | `export-hml` 계약 (HML 전용) |
| `footnote-01.hwp` | 각주가 렌더 쪽에 영향을 줄 수 있다 |
| `form-01.hwp` | 서식 컨트롤이 있는 HWPX 변환 |
| `landscape-001.hwp` | 가로 용지. 쪽수 검증이 세로 문서와 다르다 |
| `multi-table-001.hwp` | 표가 여러 개라 DocLang 손실 축이 살아 있다 |

고의로 빼 둔 표본:

- `2025 행정업무운영 편람` 같은 대형 문서. gym 채점이 무거워진다.
- 암호 걸린 `HWP5-password-*.hwp`. 보안 pack 과 겹친다.
- HWP3 원본. convert 경로가 다르고 이 확장의 범위가 아니다.

## pack.json requires

devel 의 requires 는 `export-doclang` · `export-markdown` · `info` ·
`ir-diff` 뿐이었다. SR01/SR02/SR05 가 이미 `export-hwpx` · `export-hml` ·
`export-ir-schema` 를 쓰는데도 빠져 있었다. 이번 확장은 **새로 두드리는
명령** 을 명시했다.

```text
convert
export-doclang
export-hwpx
export-markdown
export-pdf
extract-pages
info
ir-diff
```

`export-hml` · `export-ir-schema` 는 기존 과제 전용이라 목록을 부풀리지
않았다. 요구 명령이 없는 바이너리는 pack 전체가 `unavailable` 이지 0점이
아니다. 명령을 더 넣으면 오래된 바이너리가 더 자주 unavailable 이 되므로
신규 과제에 실제로 필요한 것만 넣었다.

`runner` 는 기존 해시·커밋을 유지한다. 이 확장은 바이너리를 다시 측정하지
않았다.

## 검증

로컬에서 바이너리 없이 돌리는 것:

```text
python gym/tools/audit.py
python -m unittest scripts.tests.test_gym_packs -v
python -m unittest scripts.tests.test_gym_serialization_pack -v
```

`audit.py` 는 전 pack 스키마·기준풀이 짝·ID 충돌을 본다.
`test_gym_packs.py` 는 같은 계약을 CI 가 항상 돌린다.
`test_gym_serialization_pack.py` 는 이 pack 만의 여정·필드·exit·표본
실재·골든 파일 금지를 본다.

바이너리 왕복(`build_baseline.py` / `score.py`)은 이 작업 환경에서
돌리지 못했다. 기준풀이 형식은 기존 SR01–SR12 와 같다. 라이브 오라클이라
값 박제가 없으므로 바이너리만 있으면 재계산된다.

## 위험과 잔여

- `extract-pages` 의 `pagesAfter` 는 재조판으로 요청 범위와 다를 수 있다.
  값을 박제하지 않았으므로 채점은 흔들리지 않는다. 다만 에이전트가
  "1쪽만 남겼으니 1" 이라고 추측하면 틀린다.
- `export-pdf` 바이트는 폰트에 흔들린다. 채점은 `format` · `pageCount` ·
  `renderedCount` · `backend` 만 본다.
- `convert` / `export-hwpx` 의 `--verify` 는 차이가 있는 표본에서 exit 3
  이다. 과제가 0만 허용하면 기준풀이도 실패한다. 0과 3을 같이 넣었다.
- `ir-diff` 자기대조(SR16/SR36/SR53)가 false 이면 파서 비결정성이다.
  그런 실패는 pack 이 아니라 코어 버그로 올려야 한다.
- gym/README 의 과제 수(serialization 8, 만점 19)는 이 PR 이 고치지
  않는다. 집계 문서는 별도 커밋이 맞다.

## 관련

- 이슈 #5223 (serialization pack 과제 확장)
- PR #5232
- 구조대 초안: 작업 트리 밖 `tmp-gym-rescue/serialization/`
- 예외 노트: [gym_serialization_exceptions.md](gym_serialization_exceptions.md)
