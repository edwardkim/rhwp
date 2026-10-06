# Task M100 #1378 최종 보고서 — HWPX serializer 문단 run 분할 보존

- 이슈: #1378 (M100, #1315 하위 4건 중 1번째)
- 브랜치: `local/task1378`
- 계획서: `mydocs/plans/task_m100_1378.md` / `task_m100_1378_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_1378_stage{1,2,3}.md`

## 1. 목표와 결과 요약

다중 `<hp:run charPrIDRef>` 문단이 단일 run 으로 평탄화되던 serializer 한계를 해소
— `char_shapes` 경계를 cut point 로 삼아 본문·셀·글상자 문단을 **다중 run 으로
분할 출력**하고, `diff_documents` 게이트가 문단별 char_shapes 시퀀스를
본문 + 셀·글상자(Group 재귀)·각주/미주까지 재귀 비교하도록 강화했다.

| 측정 | 착수 전 (1단계 게이트 강화 직후) | 완료 후 |
|------|--------------------------------|--------|
| 본문 경로 char_shapes diff | 37 샘플 / 1,483건 (평탄화 1,438 + 첫 문단 구조 45) | **0건** |
| 셀·글상자·각주 재귀 diff | (게이트 미비교) | 13 샘플 / 66건 — 전건 범위 밖 분류 (아래 4절) |
| baseline 게이트 | 4 테스트 | 5 테스트 (still-fail 가드 추가) 전부 그린 |

## 2. 단계별 진행

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | `ParagraphCharShapes` 게이트 + 본문 비교 + 전수 측정(37건/1,483) + 임시 xfail | `fd8b0fc5` |
| 2 | 본문 경로 `render_runs()` 전환(RunSplitter) + write_section 치환 단순화 + secPr run id 정비 — 37건 전건 해소 | `272740f9` |
| 3 | `render_text_runs()` 공유 헬퍼로 셀·글상자 분할 + 게이트 재귀 확장 + 범위 밖 13건 임시 xfail (분류 승인) | `3ad95202` |
| 4 | 전수 검증 + SVG 비교 + 매뉴얼·보고서 + CI급(wasm 포함) | 본 보고서 커밋 |

## 3. 구현 핵심

- **RunSplitter** (`src/serializer/hwpx/section.rs`): char_shapes 경계 기반 다중
  run 조립 상태기계. 경계 규칙 5종(동일 위치 경계 우선/연속 동일 id skip/빈
  char_shapes 단일 run 0/비정상 IR 관용/빈 run `<hp:t></hp:t>` 방출)을 단위
  테스트로 고정.
- **render_runs()**: UTF-16 위치 축에서 텍스트·컨트롤 슬롯·필드·경계를 함께
  처리하는 본문 경로. 호출부 수동 run 감쌈 제거, 템플릿 치환 1회로 단순화,
  섹션 첫 문단 secPr run id 를 첫 char_shape id 로 정비(양상 ② 해소).
- **render_text_runs()**: 텍스트만 경계 분할하는 셀·글상자 공유 헬퍼
  (컨트롤 출력은 #1379 범위). char_offsets 매핑으로 컨트롤 갭에도 경계 정확.
  글상자 탭/lineBreak 직렬화가 부수 정비됨.
- **게이트 재귀**: `diff_paragraph_char_shapes()` 가 컨트롤 쌍(zip 인덱스 대응)을
  따라 셀(`tbl.cell[k].p[m]`)·글상자(`shape.tb.p[m]`, Group `child[k]` 재귀)·
  각주/미주(`fn/en.p[m]`)로 내려가며 path 를 표기.
- 신규 단위 테스트 27건 (1단계 5 + 2단계 11 + 3단계 11).

## 4. 잔존 한계 (범위 밖 분류 — 3단계 승인)

`XFAIL_1378_RECURSIVE` 13건 (66 diff), 해소 시 still-fail 가드가 승격을 강제:

| 분류 | 건수 | 원인 | 해소 경로 |
|------|------|------|----------|
| 셀·글상자 경계 8 배수 시프트 | 65 (tbl 64 + shape.tb 1) | subList 컨트롤 미출력 — id 시퀀스는 보존 | **#1379** |
| 각주 경계 1 유닛 시프트 | 1 (fn) | 파서 autoNum 폭 비일관 — char_shapes 축(calc) 1 유닛 vs offsets 축 8 유닛 (`parser/hwpx/section.rs:4909` vs `642-648`) | 별도 이슈 후보 |

## 5. 전수 검증 + 시각 비교 (4단계)

### 배치 roundtrip (`output/poc/task1378/`, inventory.tsv + 48 rt 파일)

54건: **PASS 39 / IR_DIFF 13**(= 임시 xfail 13건과 정확 일치) /
SERIALIZE_FAIL 1(exam_social, 기존 XFAIL #1381) / PARSE_FAIL 1(hwpx-01, EXCLUDED).
2-round 드리프트 전건 0.

### SVG byte 비교 (원본 vs rt, `output/poc/task1378/svg/`)

대표 11건 — #1315 대표 8건 교집합(4) + run 분할 빈도 상위(4) + #1315
byte-identical 회귀 가드(3):

| 샘플 | byte-identical | #1315 대비 |
|------|---------------|-----------|
| blank_hwpx / para-001 / basic-table-01 | 1/1, 3/3, 1/1 | 유지 (회귀 없음) |
| **math-001** | **1/1** | **개선** — 0/1 (1~2px 시프트) → 전 일치 |
| footnote-01 | 0/6 | 동일 (autoNum 비일관 + #1380) |
| 보도자료f(2025-1Q) / form-002 | 0/9 (9→13쪽) / 0/10 (10→17쪽) | 동일 — #1379·#1380 기인 |
| exam_kor / aift / mel-001 / 온새미로 | 0 (페이지 폭증) | #1379·#1380 기인 |

run 분할 자체로 인한 시각 회귀는 없으며, math-001 은 글자모양 적용 위치가
정확해져 일치로 개선됐다. 잔존 차이는 전부 #1379(컨트롤 소실)·#1380(lineseg)
기인으로 #1315 측정과 동일 양상.

### 한컴에디터 판정 요청 (작업지시자)

`output/poc/task1378/` rt 파일 중 대표 8건
(blank_hwpx, para-001, basic-table-01, math-001, footnote-01, form-002,
2025년 1분기 해외직접투자 보도자료f, exam_kor)의 한컴에디터 열기 + **글자모양
(굵게/색/크기 변화 구간) 표시 정상 여부** 판정을 요청한다 — run 분할 출력의
한컴 호환 확정 (#1315 의 rt 8건 판정과 별개, run 분할 반영본).

## 6. CI급 검증

- `cargo test --lib` — 1685 passed / 0 failed
- `cargo test --tests` — 123개 바이너리 전부 ok
- `cargo test --test hwpx_roundtrip_baseline` — 5 passed (still-fail 가드 포함)
- `cargo fmt --check` — 통과
- `cargo clippy --lib --tests -- -D warnings` — 통과
- `cargo check --target wasm32-unknown-unknown --lib` — 통과

## 7. 문서

- `mydocs/manual/hwpx_roundtrip_baseline.md` — 게이트 비교 항목(char_shapes 재귀)·
  임시 xfail 체계·known limitations 이슈 번호 반영
- 오늘할일 `mydocs/orders/20260611.md` — #1378 상태 갱신

## 8. 후속

1. rt 대표 8건 한컴에디터 판정 (작업지시자)
2. #1379 subList 컨트롤 보존 → `XFAIL_1378_RECURSIVE` 12건 해소 예상
3. autoNum 파서 폭 비일관 별도 이슈 등록 (승인 후)
4. #1380 lineseg → #1381 exam_social → #1315 close (전건 해소 + 승인 후)
