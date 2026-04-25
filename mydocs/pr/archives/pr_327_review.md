# PR #327 검토 — Task #324: form-002 인너 표 페이지 분할 결함 수정

## PR 정보

| 항목 | 내용 |
|------|------|
| PR 번호 | [#327](https://github.com/edwardkim/rhwp/pull/327) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#324](https://github.com/edwardkim/rhwp/issues/324) (CLOSED — 작성자가 PR 제출 전 자체 close) |
| base/head | `devel` ← `task324` |
| 변경 | +697/-278, 11 파일 (코드 2 + 문서 8 + 골든 1) |
| Mergeable | CONFLICTING (orders/20260425.md 만 — 코드 충돌 0) |
| maintainerCanModify | ✅ true |
| CI | ✅ 모두 SUCCESS (Build & Test, CodeQL × 3) |
| 검토일 | 2026-04-25 |

## 사전 절차

1. **트러블슈팅 사전 검색** (memory 규칙)

| 문서 | 관련성 |
|------|--------|
| `table_reflow_and_cell_rendering.md` | 셀 렌더링 (다른 영역) — 직접 관련 없음 |
| `paragraph_indent_and_table_x_position.md` | 들여쓰기 + x position — 본 영역과 다름 |
| `repeat_header_image_duplication.md` | 분할 표 header — 다른 케이스 |
| `multi_tac_table_pagination.md` | 다중 TAC 표 — 본 PR 과 다른 경로 |

→ 신규 영역. 본 PR 이 새 트러블슈팅 후보 (분할 표 atomic nested table 가시성).

2. **이슈 #324 상태 점검**

이슈 #324 가 본 PR 제출 (06:11) 보다 1시간 앞서 (05:17) 작성자에 의해 CLOSED 상태. 작업 완료 후 코멘트 + close → PR 제출 흐름. **작업지시자 승인 없는 자체 close** — 메모리 규칙 위반 가능성 (`feedback_no_close_without_approval.md`). PR review 단계에서 이슈 reopen 검토 필요.

## 변경 요약

### 핵심 수정 (3-단계, 작성자 자체 보강)

#### v1 (`4452894`) — `compute_cell_line_ranges` 누적위치 기반 재작성

기존: `offset_remaining`/`limit_remaining` 잔량 추적 → 잔량 0 도달 시 cumulative position 정보 손실
변경: 셀 시작부터 누적 위치 (`cum`) 명시적 추적 → atomic nested table 의 `para_end_pos` 로 정확한 페이지 분할 결정

#### v2 (`3d304cd`) — `layout_partial_table` 의 `content_y_accum` 갱신 누락 수정

기존: offset 으로 완전 소비된 일반 문단 (`line_ranges=(n,n)`) 스킵 시 `content_y_accum` 미갱신 → 후속 nested table 위치 판정 부정확
변경: `!has_nested_table` 분기에 `is_in_split_row` 일 때 `content_y_accum` 갱신 추가

#### v3 (`e086ab4`) — `split-start row` cell visible height 통일된 계산

기존: `has_nested_table` 셀 분기가 `calc_nested_split_rows` 에 raw `split_start_content_offset` (cell 전체 기준) 그대로 전달 → inner table 의 cell 내 위치 무시
변경: `has_nested_table` 분기 제거, 모든 셀에 `compute_cell_line_ranges` + `calc_visible_content_height_from_ranges` 통일 경로

### 변경 파일

| 파일 | 변경 | 비고 |
|------|------|------|
| `src/renderer/layout/table_layout.rs` | +63/-78 | `compute_cell_line_ranges` 재작성 |
| `src/renderer/layout/table_partial.rs` | +44/-64 | split-end atomic 스킵 + content_y_accum 갱신 + 통일 경로 |
| `tests/golden_svg/form-002/page-0.svg` | +2/-136 | 의도된 갱신 (인너 표 page 1 제거) |
| 문서 (수행/구현/stage1/2/2_v2/report) | +557 | 작성자 작성 |
| `samples/hwpx/form-002.pdf` | (binary) | PDF 참조 자료 |

## 검토 시 확인할 점

### A. 코드 정확성

| 항목 | 평가 |
|------|------|
| **누적위치 기반 재작성의 의미** | 잔량 추적의 한계 (잔량 0 도달 시 cumulative 정보 손실) 정확히 식별. `cum`/`para_end_pos`/`para_start_pos` 명시적 추적은 정공법 |
| **atomic 단위 가시성 결정** | `was_on_prev` (이전 페이지 전체 포함) + `exceeds_limit` (다음 페이지로 미룸) 두 가지 명시적 가드. 명료 |
| **content_y_accum 갱신 누락** | v2 의 핵심 수정 — split-end 페이지에서 일반 문단 스킵 후 nested table 위치가 부정확했던 점, 시각 검증으로 식별. 정확 |
| **통일 경로 (v3)** | has_nested_table 분기 제거로 코드 단순화 + 정합성 향상. 정공법 |

### B. 회귀 리스크

| 리스크 | 검증 |
|--------|------|
| 기존 테스트 영향 | `cargo test --release` 992 lib + 71 통합 통과 (작성자 보고) |
| svg_snapshot 골든 영향 | `form-002/page-0.svg` 1건 (의도) — 다른 골든 무영향 작성자 보고 |
| 페이지 수 변화 | dump-pages 결과 동일 (page 1 rows=0..20, page 2 rows=19..26) |

### C. 절차 준수 점검 (외부 기여자 PR)

| 규칙 | 준수 | 비고 |
|------|------|------|
| 이슈 → 브랜치 → 계획서 → 구현 순서 | ✅ | 수행/구현/stage1/2/2_v2/report |
| 작업지시자 승인 없는 이슈 close | ⚠️ | **이슈 #324 가 PR 제출 전 자체 close** — 메모리 규칙 위반 가능성 |
| 브랜치 `task{번호}` | ✅ | task324 |
| 커밋 메시지 `Task #N:` | ✅ | 일관 |
| Test plan | ✅ | 992 lib + 71 통합 + clippy 보고 |
| 자체 보강 (v1 → v2 → v3) | ✅ | 시각 검증으로 자체 발견 후 단계별 수정 — 책임감 있는 처리 |

## 처리 방향

### Mergeable 충돌

`mydocs/orders/20260425.md` 만 충돌 (PR #282 머지로 인한 Task #279 섹션 추가 vs PR #327 의 Task #324 섹션 추가). 코드 충돌 0. 메인테이너가 force-push 로 직접 정리 (PR #282 사례와 동일 방식) 가능.

### 검증 흐름

1. 본 review 작성지시자 승인
2. 이슈 #324 reopen 검토 (자체 close 위반 사항 — 머지 후 close 권한 회복)
3. 작성자 fork merge devel + 충돌 해소 후 force-push
4. 빌드/lib test/svg_snapshot/clippy/wasm32 검증
5. WASM Docker 빌드 + 작업지시자 시각 검증 (form-002 page 1/2)
6. CI 통과 + admin merge

## 판정 (예정)

✅ **Merge 권장** (검증 통과 시)

**사유:**
1. 명확한 원인 분석 (잔량 추적 → 누적위치 추적의 한계 식별)
2. 시각 검증으로 자체 보강 (v1 → v2 → v3) — 책임감 있는 처리
3. CI 모두 SUCCESS
4. 통일 경로 (v3) 로 코드 단순화 + 정합성 향상

**처리 시 주의:**
- 이슈 #324 자체 close 사항을 작성자에게 인지시킬 필요 (메모리 규칙 안내)
- 다음 외부 기여자 PR 처리에서 동일 패턴 반복 방지

**머지 후 후속 (선택):**
- 트러블슈팅 등록 (분할 표 atomic nested table 가시성 — `has_offset/has_limit + cum + para_end_pos` 패턴)
- Task #325 후속 이슈 (cell.h vs 실제 콘텐츠 누적 높이 불일치, diff=-21.2px)

## 참고 자료

- 이슈: [#324](https://github.com/edwardkim/rhwp/issues/324)
- 관련 작업: Task #309 Epic (Paginator 페이지 분할 작업) — Task #311/#312/#313/#314/#317/#318
- 후속 이슈 후보: #325 (cell.h vs 콘텐츠 높이 불일치)
