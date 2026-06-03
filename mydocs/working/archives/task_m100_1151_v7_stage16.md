# Task #1151 v7 Stage 16 완료 보고서 — Audit 기반 코드 품질 정리 (요약)

수행계획서: [task_m100_1151_v4.md](../plans/task_m100_1151_v4.md) (v5~v7 모두 v4 PR 머지 전 same-PR 정리) · Stage 9 (v5) / Stage 10 (v6) / Stage 12~15 (v7)

## 1. Context

Task #1151 (v1~v6, 22 commits) 의 코드 audit (2026-05-30, Explore agent 2 회) 결과 9 항목 평가. 사용자 결정으로 4 개 fix 진행 (Stage 12~15) + 나머지는 분석 결과 "의도된 분리" 로 확정하여 별도 이슈 발급 없이 정리.

## 2. Audit 항목별 처리

| # | 항목 | 처리 | Stage |
|---|------|------|-------|
| **1** | cell_ctx → ImageNode 3 필드 매핑 4 곳 반복 (DRY 위반) | **Fix** — `CellContext::last_image_indices()` helper 추가 + 4 곳 적용 | 12 |
| **4** | by_path setter/getter 4 함수의 cell_path 파싱 + traversal 95 줄 중복 | **Fix** — `parse_cell_path_json` + `resolve_cell_paragraph_mut` helper 추출 + 4 함수 적용 (v4 cell picture 2 + Task #1138 cell shape 2) | 13 |
| **7** | paragraph_layout 3 곳의 ImageNode 생성 boilerplate (~28 줄 × 3) | **Fix** — `make_picture_image_node` private helper 추출 + 3 곳 적용. 같은 코드베이스의 `picture_footnote::layout_picture_full` 가 helper 패턴 표준 | 14 |
| **2** | `Table::update_ctrl_dimensions` 의 raw_ctrl_data ↔ self.common dual maintenance | **유지 + 주석 보강** — 진단 결과 의도된 구조 확정 (아래 §3 참조) | 15 |
| **3** | `invalidate_page_tree_cache` 호출이 모든 setter 에 분산 | **유지** — 진단 결과 의도된 책임 분리 (아래 §4 참조) | — |
| 5 | dialog setter 4 종 분기 | Skip (UI 리팩토링, 별도) | — |
| 6 | `layout_picture` 매개변수 11 개 폭증 | Skip (`#[allow(clippy::too_many_arguments)]` 패턴, 표준) | — |
| 8 | `migrate_picture_floating_to_inline` 위치 | OK (commands 영역 적절) | — |
| 9 | `calc_sibling_topandbottom_table_reserved_hu` 가시성 | OK (`pub(crate)` 적절) | — |

## 3. 항목 2 (Table dual maintenance) — 의도된 분리 확정

진단 결과:
- `serializer/control.rs:461` (Table): `table.raw_ctrl_data` 그대로 기록 → **raw_ctrl_data 가 source-of-truth**
- `serializer/control.rs:895` (Picture/Shape): `&serialize_common_obj_attr(&pic.common)` 매번 재생성 → **self.common 이 source-of-truth**

→ Table 만 dual 인 것은 serializer 의 source-of-truth 정책 차이로 의도된 구조. v6 fix (self.common 동기화 추가) 는 정합 fix — paragraph_layout cache (self.common.height) 와 serialize source (raw_ctrl_data) 의 양쪽 갱신.

추후 모델 통일 (Picture/Shape 정합으로 Table 의 raw_ctrl_data 폐기) 가능성 있으나 본 PR 범위 외.

## 4. 항목 3 (invalidate_page_tree_cache 분산) — 의도된 분리 확정

진단 결과:
- 각 setter 가 자신의 mutation 책임 명확화 — page tree 영향 유무를 setter 가 가장 잘 앎
- 모든 mutation 이 invalidate 필요한 건 아님 (예: log_only 변경) — 자동화 시 불필요한 invalidate 발생 가능
- v5 의 누락 결함은 일관성 부족이 root cause 였으나, 이는 코드 리뷰 / regression test 로 보완 가능

→ 분산은 anti-pattern 이 아닌 **의도된 책임 명확화**. v5 fix 는 누락된 일관성 보강 = 정합 fix. 별도 리팩토링 불필요.

## 5. 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` 전수 | **1445 passed, 0 failed, 6 ignored** (회귀 0) |
| `cargo clippy --lib -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| v1~v6 의 모든 기존 테스트 | PASS 유지 |
| 진단 테스트 (v5_, v6_) | PASS 유지 |

## 6. v7 정량 효과

| Stage | 파일 | 정량 |
|-------|------|------|
| 12 (항목 1) | layout.rs + paragraph_layout.rs + picture_footnote.rs | +47 / -41 (helper 추가 + 4 곳 boilerplate 축소) |
| 13 (항목 4) | object_ops.rs | +65 / -118 (순 -53 줄, ~45% 보일러플레이트 제거) |
| 14 (항목 7) | paragraph_layout.rs | +81 / -91 (순 -10 줄 + helper 분리로 재사용성 향상) |
| 15 (항목 2) | table.rs | +14 / -5 (주석 보강만) |
| **합계** | 5 파일 | **+207 / -255 = 순 -48 줄** + 4 helper 추출 |

## 7. Stage 17 진입 조건

- 모든 audit 항목 처리 완료 (4 fix + 2 의도된 유지 + 4 skip/OK) ✓
- 회귀 0 (cargo test/clippy/fmt 모두 통과) ✓
- 4 helper 추출로 향후 유지보수성 향상

→ Stage 17 (v7 통합 최종 보고서 + push + upstream devel PR) 진행.
