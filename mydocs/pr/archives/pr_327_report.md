# PR #327 처리 결과 보고서

## PR 정보

| 항목 | 내용 |
|------|------|
| PR 번호 | [#327](https://github.com/edwardkim/rhwp/pull/327) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeook Ryu) |
| 이슈 | [#324](https://github.com/edwardkim/rhwp/issues/324) |
| 처리 | **Merge (admin)** |
| 처리일 | 2026-04-25 |
| Merge commit | `4fd4565` |

## 변경 요약

`samples/hwpx/form-002.hwpx` 의 1×1 inner table ("연구개발계획서 제출시…") 페이지 분할 결함 3건을 단계적으로 수정.

### 3-단계 자체 보강

**v1** (`compute_cell_line_ranges` 누적위치 기반 재작성):
- 잔량(remaining) 추적 → 잔량 0 도달 시 cumulative position 정보 손실
- 셀 시작부터 누적 위치 (`cum`) 명시적 추적 + atomic nested table `was_on_prev` / `exceeds_limit` 두 가드

**v2** (`layout_partial_table` content_y_accum 갱신):
- offset 으로 완전 소비된 일반 문단 스킵 시 `content_y_accum` 미갱신
- `!has_nested_table` 분기에 갱신 추가

**v3** (split-start row 통일된 계산):
- `has_nested_table` 셀 분기 제거 → 모든 셀에 `compute_cell_line_ranges` 통일 경로

## 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ 27.19s |
| `cargo test --lib` | ✅ 992 passed |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed (form_002_page_0 갱신 의도) |
| `cargo test --test issue_301` | ✅ z-table 가드 |
| `cargo clippy --lib -D warnings` | ✅ clean |
| `cargo check --target wasm32` | ✅ clean |
| 7 핵심 샘플 페이지 수 회귀 | ✅ 무변화 |
| WASM 시각 검증 (form-002 page 1/2) | ✅ 통과 |

## 처리 흐름

1. PR review 문서 작성 + 작업지시자 승인 (1)
2. 이슈 #324 reopen (작성자 자체 close 사항 정정)
3. 작성자 fork merge devel (orders/20260425.md 충돌만, 코드 충돌 0)
4. 자동 검증 + WASM 시각 검증
5. force-push 후 CI 통과 → admin merge
6. 이슈 #324 close
7. 트러블슈팅 등록 (`cell_split_nested_table_visibility.md`)

## 안내 사항

이슈 #324 가 PR 제출 (06:11) 전 자체 close (05:17) 되어 있어, 메모리 규칙 (`feedback_no_close_without_approval.md`) 위반 안내. PR 머지 코멘트에 정중하게 전달 — 다음 PR 부터 PR 제출 시 이슈를 OPEN 상태로 두도록 요청.

## 후속 이슈 후보

- **#325**: cell.h 과 실제 콘텐츠 누적 높이 불일치 (form-002 page 2 의 `diff=-21.2px`). Epic #309 의 후속 task 후보.

## 참고 링크

- [PR #327](https://github.com/edwardkim/rhwp/pull/327)
- [merge announcement comment](https://github.com/edwardkim/rhwp/pull/327#issuecomment-4319849606)
- 트러블슈팅: `mydocs/troubleshootings/cell_split_nested_table_visibility.md`
- 작성자 산출물 (PR 에 포함):
  - `mydocs/plans/task_m100_324{,_impl}.md`
  - `mydocs/working/task_m100_324_stage{1,2,2_v2}.md`
  - `mydocs/report/task_m100_324_report.md`
