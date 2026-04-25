# PR #315 최종 보고서 — Epic #309 1차: Task #311 + #312

## 결정

✅ **Merge 승인 (admin merge)** — Investigation/Spike PR 모범 사례

## PR 정보

- **PR**: [#315](https://github.com/edwardkim/rhwp/pull/315)
- **이슈**: #311, #312 (둘 다 PR 본문 `closes` 로 CLOSED)
- **작성자**: @planet6897
- **처리일**: 2026-04-25
- **Merge commit**: `bf67ec8`

## 처리 절차

1. ✅ PR 브랜치 체크아웃 (`task312`)
2. ✅ devel 머지 → `mydocs/orders/20260425.md` 1구간 충돌 해결
3. ✅ 검증: 992 passed / 0 failed, dump-pages 신규 컬럼 정상 동작
4. ✅ `planet6897/task312` 에 push (`57dd81a..152ae40`)
5. ✅ 재승인 + admin merge → 이슈 #311, #312 CLOSED 확인

## 승인 사유

1. **default 동작 변경 0** — 옵트인 플래그 (`--respect-vpos-reset`) + 진단 출력 (`used/hwp_used/diff` 컬럼) 만 추가
2. **두 가설 모두 데이터 기반 부정** — Task #310 권장 가설 부정 + column 단일 origin 가설 부정
3. **의외의 발견** — TypesetEngine 이 이미 verification 모드로 작동 중이며 PDF에 더 가까운 결과 산출 → Epic #309 다음 단계 진로 (#313) 명확화
4. **신규 진단 도구 즉시 활용 가능** — 페이지네이션 디버깅 가속화
5. **CLAUDE.md 절차 완전 준수** — 두 task 모두 계획서/보고서 완비

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ 992 passed / 0 failed / 1 ignored |
| `cargo test --test svg_snapshot` | ✅ 6 passed (golden 유지) |
| `cargo test --test issue_301` | ✅ 1 passed (#301 회귀 없음) |
| `cargo test --test tab_cross_run` | ✅ 1 passed (#290 회귀 없음) |
| `cargo clippy --lib -- -D warnings` | ✅ clean |
| `cargo check --target wasm32` | ✅ clean |
| CI (원본) | ✅ 전부 SUCCESS |
| dump-pages 신규 컬럼 동작 | ✅ `used / hwp_used / diff` 정상 |

## 변경 내역

**코드 (10파일):**
- `src/renderer/pagination.rs` — `PaginationOpts` 구조체 + `ColumnContent.used_height` 필드
- `src/renderer/pagination/engine.rs` — `paginate_with_forced_breaks` 메서드 추가
- `src/renderer/pagination/state.rs` `typeset.rs` — `flush_column` 에서 `used_height` 저장
- `src/document_core/queries/rendering.rs` — `dump_page_items` used/hwp_used/diff 출력 + `compute_hwp_used_height` 헬퍼
- `src/document_core/mod.rs` `commands/document.rs` — `respect_vpos_reset` 필드
- `src/wasm_api.rs` — `set_respect_vpos_reset` 셋터
- `src/main.rs` — `--respect-vpos-reset` CLI 플래그
- `src/renderer/layout/tests.rs` — fixture 5건 업데이트

**문서 (14개):**
- `mydocs/plans/task_m100_311{,_impl}.md` `task_m100_312{,_impl}.md`
- `mydocs/working/task_m100_311_stage{1,2,3}.md` `task_m100_312_stage{1,2,3}.md`
- `mydocs/report/task_m100_311_report.md` `task_m100_312_report.md`
- `mydocs/tech/line_seg_vpos_analysis.md` 부록 A 추가

## Investigation/Spike PR 패턴 정착

본 PR 은 어제 정착시킨 **Investigation/Spike PR 모범 사례**의 두 번째 적용:

1. PR #308 (Task #306 + #310): vpos 진단 도구 1차
2. **PR #315 (Task #311 + #312)**: 가설 부정 + 의외의 발견
3. 차후 PR #316 통합 → #313/#318/#317 분리 → 단계별 머지 흐름

## 후속 PR (대기 중)

- **#320** (Task #318): #316 의 z-table 회귀 수정 (다음 검토)
- **#319** (Task #317): HWPX 어댑터 +1쪽 잔존 사안
- **#316** (Epic 통합): 위 분리 머지 후 자연스럽게 닫힘
