# Task #546 Stage 4 완료 보고서 — 회귀 검증 종합

## 결정적 검증 (모두 통과)

| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | ✅ **1113 passed** (PR #538 시점과 동일, 회귀 0) |
| `cargo test --test issue_546` (신규) | ✅ 1 passed |
| `cargo test --test issue_505` | ✅ 9/9 passed (PR #507 회귀 0) |
| `cargo test --test issue_530` | ✅ 1 passed (PR #531 회귀 0) |
| `cargo test --test issue_418` | ✅ 1 passed |
| `cargo test --test issue_501` | ✅ 1 passed |
| `cargo test --test svg_snapshot` | ✅ **6/6 passed** (table_text_page_0 + issue_267_ktx_toc_page 등) |
| `cargo clippy --lib -- -D warnings` | ✅ 0 건 |
| `cargo build --release` | ✅ Finished |

## 광범위 fixture sweep — byte-level 회귀 검증

PR #538 머지 후 (`e585b58`) ↔ revert 후 (현재) 의 SVG byte 비교:

| fixture | before / after byte-identical | 결과 |
|---------|-----------------------------|------|
| 2010-01-06.hwp | 6 / 6 | ✅ 회귀 0 |
| 21_언어_기출_편집가능본.hwp | 15 / 15 | ✅ 회귀 0 (PR #538 fixture) |
| exam_eng.hwp | 8 / 8 | ✅ 회귀 0 |
| exam_kor.hwp | 20 / 20 | ✅ 회귀 0 |
| exam_math.hwp | 20 / 20 | ✅ 회귀 0 |
| **exam_science.hwp** | **0 / 4** (4 페이지 모두 변경) | ✅ **의도된 정정** (회귀 6→4 페이지 + p2 본문 복원) |
| synam-001.hwp | 35 / 35 | ✅ 회귀 0 |
| 복학원서.hwp | 1 / 1 | ✅ 회귀 0 |

**총 105 페이지 / 8 fixture 검증 — exam_science 의 의도된 4 페이지만 변경 + 다른 101 페이지 byte-identical.**

옵션 A revert 가 본 task 영역 외에 영향 0. v0.7.9 정합 + 본 사이클 후속 정정 (PR #506/#507/#509/#510/#514/#516/#531/#538) 의 다른 영역 정정 모두 보존됨.

## v0.7.9 (main) 와의 페이지 수 정합 (Stage 3 후속 검증)

| fixture | v0.7.9 | revert 후 | 정합 |
|---------|--------|-----------|------|
| 2010-01-06.hwp | 6 | 6 | ✅ |
| 21_언어_기출_편집가능본.hwp | 15 | 15 | ✅ (PR #538 의 fixture) |
| exam_eng.hwp | 8 | 8 | ✅ |
| exam_kor.hwp | 20 | 20 | ✅ |
| exam_math.hwp | 20 | 20 | ✅ |
| exam_science.hwp | **4** | **4** | ✅ (회귀 정정) |
| synam-001.hwp | 35 | 35 | ✅ |
| 복학원서.hwp | 1 | 1 | ✅ |

→ 모든 fixture 가 v0.7.9 와 동일 페이지 수.

## exam_science.hwp 변경 영역 (의도된 정정)

byte 변경 4 페이지 모두 본 task 의 의도된 정정:

| 페이지 | 변경 본질 |
|--------|----------|
| `_001.svg` | p1 의 본문 paragraph 들이 더 정합한 위치로 (회귀 시점 후속 paragraph 의 일부가 p2 로 분산되었던 것 복원) |
| `_002.svg` | **p2 본문 복원** (회귀 2 items → 정상 37 items) |
| `_003.svg` | p3 의 paragraph 위치 정합 |
| `_004.svg` | p4 의 paragraph 위치 정합 |

## 다음 단계

Stage 4 보고서 승인 후 **Stage 5** 진행:

1. WASM 빌드 + studio 동기화
2. 작업지시자 시각 판정 (rhwp-studio + 한컴 2010/2020):
   - exam_science.hwp p2 본문 정상 출력 확인
   - 다른 fixture 회귀 0 확인 (위 byte-identical 결과로 결정적 검증 완료, 시각 판정은 보강)
3. 최종 보고서 (`mydocs/report/task_m100_546_report.md`)
4. orders 갱신
5. local/task546 → local/devel merge → devel push → 이슈 close

## 산출물

- 본 보고서 (`mydocs/working/task_m100_546_stage4.md`)
- 변경분 (commit 대기): `src/renderer/layout.rs` (-58) + `src/renderer/typeset.rs` (-36)
- 신규 테스트: `tests/issue_546.rs`
- 산출물 documents: stage1/3/4 + plans/impl

## 메모리 정합

- ✅ `feedback_v076_regression_origin` — 옵션 A 의 revert 결과가 v0.7.9 정합 + 본 사이클 후속 정정 보존
- ✅ `feedback_visual_regression_grows` — byte-level 광범위 sweep 으로 회귀 0 결정적 검증 (시각 판정 게이트는 Stage 5)
- ✅ `feedback_pdf_not_authoritative` / `reference_authoritative_hancom` — Stage 5 시각 판정 단계로 한컴 2010/2020 직접 비교 진행
- ✅ `feedback_image_renderer_paths_separate` — 본 정정은 typeset 단계만, renderer 분기 영향 없음
