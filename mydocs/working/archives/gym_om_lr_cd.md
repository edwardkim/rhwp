---
kind: investigation
status: active
canonical: mydocs/working/gym_om_lr_cd.md
last_verified: 2026-08-18
---

# OM·LR·CD pack 확장 작업 노트

이 문서는 PR #5222 (`feat/gym-om-lr-cd-expand`) 를 키운 작업 기록이다.
규범 문서는 각 pack README 다.

- [gym/packs/objects-media/README.md](../../../gym/packs/objects-media/README.md)
- [gym/packs/layout-rendering/README.md](../../../gym/packs/layout-rendering/README.md)
- [gym/packs/corpus-diagnostics/README.md](../../../gym/packs/corpus-diagnostics/README.md)

## 무엇을 했는가

[#5219](https://github.com/edwardkim/rhwp/issues/5219) 첫 확장은 OM08–OM09,
LR09–LR10, CD08–CD10 일곱 건이었다(약 365 insertions). 세 pack 모두
여정이 한 줄짜리라 에이전트가 힌트 한 줄을 외워 축 전체를 통과할 수
있었다. 같은 계약으로 표본·필드·깊이를 갈라 과제를 더하고, 한국어
README 세 장과 계약 테스트를 붙였다.

건드리지 않은 것:

- 새 CLI, 새 pack, T07 복제 (`홍길동` + `filled.hwp` + 첫 필드 단일 검사)
- 편집 과제에 `deep_contains` / `not_contains`
- `profiles/` · `gym/README.md` · `gym/PARK.md` · `gym/core/checks.py`
- 다른 pack 의 과제 ID
- `cargo fmt --all` (JSON·문서·테스트만 바꿨다)
- `pack.json` 의 `runner` 신원 (`rhwpVersion` / `rhwpCommit` /
  `capabilitiesSha256`). 요구 명령 목록도 기존 값을 유지했다.

## 왜 이 두께인가

세 pack 은 명령 수가 적다. 얇게 두면 한 표본 × 한 필드가 축 전부가
된다. 같은 `fields` 라도

- 본편 서식인가, 메모 판인가, HWPX 서식인가
- 읽는 키가 `fieldCount` 인가 `name` 인가 `fieldType` 인가
- 채우는 자리가 첫째인가 둘째인가 셋째인가, 한 자리인가 쌍인가

가 다른 계약이다. 같은 `verify` 라도 `--expect-format hwp5` 와 `hwpx`,
통과하는 최소 쪽과 일부러 깨는 100쪽·99표가 갈라진다. 같은 `scan` 라도
폴더와 `--max-depth` 1/2/재귀가 갈라진다.

과제를 합치면 에이전트가 "fields 를 한 번 돌리면 된다"고 학습한다.
갈라 두면 자리를 다시 지목해야 한다.

## 과제 계보

### objects-media

#### 기존 (devel)

| ID | 명령 | 요지 |
|---|---|---|
| OM01 | `fields` | `fieldCount` |
| OM02 | `fill-fields` | 둘째만 '지목채움' |
| OM03 | `thumbnail` | 다쪽 문서 `width` |
| OM04 | `export-markdown` | 다쪽 문서 `imageCount` |
| OM05 | `fields` | `fields[2].name` |
| OM06 | `fill-fields` | 첫째·셋째 |
| OM07 | `export-markdown` | 실문서 `imageCount` |

#### 첫 확장 (OM08–OM09)

| ID | 명령 | 요지 |
|---|---|---|
| OM08 | `fields` | `fields[0].fieldType` |
| OM09 | `thumbnail` | 다쪽 문서 `mime` |

#### 이번 확장 (OM10–OM45)

| ID | 명령 | 요지 |
|---|---|---|
| OM10 | `fields` | 메모 판 `fieldCount` |
| OM11 | `fields` | `fields[1].fieldType` |
| OM12 | `fields` | `fields[1].name` |
| OM13 | `fields` | `fields[0].name` |
| OM14–OM16 | `fields` | form-01 개수·이름·유형 |
| OM17–OM19 | `thumbnail` | table-001 가로·세로·MIME |
| OM20–OM22 | `thumbnail` | para / 실문서 / landscape |
| OM23–OM25 | `export-markdown` | table / para / 2쪽 시험지 |
| OM26 | `fields` | form-02 `fieldCount` |
| OM27–OM28 | `fields` | 메모 판 이름·유형 |
| OM29–OM30 | `thumbnail` / md | 1쪽 시험지 · landscape |
| OM31–OM35 | `fill-fields` | 자리·값 조합 (T07 아님) |
| OM36–OM39 | thumb / fields / md | form-01 세로 · 셋째 유형 · 2p 세로 · 1p 이미지 |
| OM40–OM45 | fields / thumb / fill | 서평·form-02·실문서 MIME·request·감사팀 |

채움 값 목록: `진단부서` · `한컴점검` · `기록사`/`기록자` · `점검메시지` ·
`운영자`/`운영팀` · `감사팀`. `홍길동` 없음. 산출 이름:
`dept.hwp` · `company.hwp` · `pair.hwp` · `formout.hwpx` · `ops.hwp` ·
`audit.hwp`. `filled.hwp` 없음.

### layout-rendering

#### 기존 (devel)

| ID | 명령 | 요지 |
|---|---|---|
| LR01 | `info` | 다쪽 `pageCount` |
| LR02 | `verify` | min-pages 17 |
| LR03 | `export-svg` | table-001 첫 쪽 |
| LR04 | `render-diff` | table-001 라운드트립 |
| LR05 | `thumbnail` | 다쪽 `height` |
| LR06 | `verify` | min-pages 100 (위반) |
| LR07 | `export-svg` | 실문서 첫 쪽 |
| LR08 | `verify` | 10쪽+표 1 |

#### 첫 확장 (LR09–LR10)

| ID | 명령 | 요지 |
|---|---|---|
| LR09 | `info` | table-001 `paraCount` |
| LR10 | `verify` | `--expect-format hwp5` |

#### 이번 확장 (LR11–LR48)

| 묶음 | ID | 요지 |
|---|---|---|
| 쪽수 | LR11–LR14, LR36, LR38, LR39, LR43 | 시험지 2/3/4/1p, para, landscape, 실문서, english |
| 문단 | LR15–LR17, LR44 | para, 1p, landscape, english |
| 형식 | LR18–LR21, LR37, LR45 | hwp5 / hwpx |
| 최소 기대 | LR22–LR27, LR40, LR46, LR48 | 통과·위반·이중 기대·passCount |
| 썸네일 | LR28, LR29, LR41 | width / mime / landscape width |
| 회귀 | LR30–LR32, LR47 | para / 2p / landscape / 실문서 |
| SVG | LR33–LR35, LR42 | para / 1p / landscape / 2p |

### corpus-diagnostics

#### 기존 (devel)

| ID | 명령 | 요지 |
|---|---|---|
| CD01 | `scan` | `samples/hml` 재귀 |
| CD02 | `scan --probe` | hml `extMismatch` |
| CD03 | `dump-pages` | 다쪽 `pageCount` |
| CD04 | `ir-diff` | 143E 쌍 `identical` |
| CD05 | `render-diff A A` | status + maxDisp 0 |
| CD06 | `extract-pages` | 다쪽 1–1 |
| CD07 | `convert` | table-001 `format` |

#### 첫 확장 (CD08–CD10)

| ID | 명령 | 요지 |
|---|---|---|
| CD08 | `scan --max-depth 1` | `samples/hwpx` |
| CD09 | `ir-diff A A` | table-001 |
| CD10 | `extract-pages` | 다쪽 2–2 |

#### 이번 확장 (CD11–CD48)

| 묶음 | ID | 요지 |
|---|---|---|
| 스윕 | CD11–CD15, CD35, CD39, CD43 | basic/chart/unicode/hwpx2/images/issues/hml1 |
| 탐침 | CD14, CD48 | hwpx · basic |
| 덤프 | CD16–CD19, CD37, CD44 | table/2p/3p/para/landscape/4p |
| IR | CD20–CD23, CD36, CD40, CD45, CD47 | 자기대조·짝·diffCount·표vs문단 |
| 렌더 | CD24, CD25 | para / 1p A=A |
| 추출 | CD26–CD30, CD41, CD46 | 2p/3p/4p 첫·둘째·마지막 |
| 변환 | CD31–CD34, CD38, CD42 | para/land/1p/hwpx표/실문서/aift |

## 제약과 지킨 것

1. **기존 표본만.** 모든 `input` 과 보조 경로는 `samples/` 아래 실재
   파일·폴더다. 새 픽스처를 만들지 않았다.
2. **기존 연산자만.** `answer_eq` · `len_answer_eq` · `value_eq` ·
   `file_exists` · `differs_from_input` · `xml_root_eq`. 스키마에 없는
   연산자는 없다.
3. **편집에 전역 훑기 없음.** 채움 과제는 `fields[n].value` 를
   `value_eq` 로 지목한다.
4. **T07 미복제.** 첫 필드 + `홍길동` + `filled.hwp` 조합이 없다.
5. **라이브 오라클.** 쪽수·필드 수·픽셀·MIME·파일 수를 과제 JSON 에
   숫자로 박제하지 않았다. 예외는 추출물의 `pageCount == 1` (CD06/CD10
   과 같은 단일 쪽 계약) 과 자기 렌더 `maxDisp == 0` 뿐이다.
6. **extract-pages 는 1 기준.** `--from`/`--to` 가 1 이상이다.
7. **export-svg 는 0 기준.** `-p 0` 이 첫 쪽이다.
8. **고유 ID.** OM·LR·CD 접두사는 다른 pack 과 충돌하지 않는다.

## 위험

- CD08·CD11–CD15·CD35·CD39·CD43 은 해당 `samples/` 폴더의 현재 파일
  수에 묶여 있다. 문서를 더하거나 빼면 라이브 오라클 정답이 바뀐다.
  그게 스윕 과제의 계약이다.
- 단일 쪽 추출(CD26–CD30, CD41, CD46)은 재조판 뒤 쪽수가 1 이 아니면
  기준 풀이가 깨진다. 입력은 이미 CD06/CD10 이 쓰는 다쪽 시험지 계열이다.
- form-02 · field-01-memo · BlogForm 의 필드 구성은 본편 field-01 과
  다를 수 있다. 채움은 알려진 이름(`회사명`·`작성자`·`부서명`·
  `myMsg01`)만 썼고, 나머지 서식은 조사(`fieldCount`/`name`/`fieldType`)
  만 한다.
- `convert` 산출이 이미 편집 가능본과 바이트가 같을 수 있어, 변환
  과제는 CD07 처럼 `differs_from_input` 을 걸지 않았다. 형식 표기와
  파일 실재만 본다.

## 검증

저장소 루트에서:

```
python gym/tools/audit.py
python -m unittest scripts.tests.test_gym_packs
python -m unittest scripts.tests.test_gym_om_lr_cd_packs
```

`cargo fmt --all` 는 돌리지 않는다. JSON·Markdown·Python 만 바꿨다.

## 의도적으로 하지 않은 것

- `pack.json` runner 신원 갱신. 점수 신원은 기존 커밋에 묶여 있다.
- `requires.commands` 확장. `export-svg` · `extract-pages` · `convert` ·
  `fill-fields` 는 이미 기존 과제가 쓰고 있었고, 이번에도 그 표면만
  늘렸다.
- 새 골든 바이트, 새 샘플, 새 연산자, 새 하위명령.
- gym 프로필·PARK 문서 수정. 과제 수 표기가 남아 있으면 후속 문서
  PR 에서 맞춘다.
