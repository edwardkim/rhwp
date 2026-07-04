# PR #1568 처리 보고서 — visible host 섹션 제목/표 세로 레이아웃 한컴 정렬

- PR: https://github.com/edwardkim/rhwp/pull/1568
- 제목: `Task #1549: visible host 섹션 제목/표 세로 레이아웃을 한컴 기준으로 정렬`
- 작성자: kkyu8925 (두 번째 기여, #1548 후속)
- 연결: Closes #1549, #1535(PR #1548) 위 stack
- base ← head: `devel` ← `kkyu8925:fix/1549-host-title-table-vertical-layout`
- 처리일: 2026-06-27

## 1. 처리 결정

**admin merge (시각 판정 통과 후).** visible host 문단(섹션 제목)에 양수 offset co-anchored
TopAndBottom float 표가 있을 때, 제목이 표 아래로 밀리거나 표와 겹치거나 간격 없이 붙던
문제(#1549)를 한컴 기준으로 정렬한다. fork PR 이라 CI 자동 미실행 → 로컬 전체 회귀로 대체.
시각 정합이 본질이라 작업지시자 시각 판정을 거쳐 통과 확인 후 merge.

## 2. 변경 범위

| 파일 | 내용 |
|---|---|
| `src/renderer/layout.rs` | visible host 제목/표 세로 레이아웃 4가지 정렬 (#1549 신규 델타) |
| `tests/issue_1549.rs` | 신규 회귀 (2 단언, red→green) |
| `samples/issue1549_multipositive_float_tables.hwpx`, `issue1549_empty_host_float_clamp.hwpx` | 합성 fixture (issue1510 Clone and Narrow) |
| `mydocs/plans·report/task_m100_1549*` | 계획/보고서 |

> #1535 부분(issue_1535.rs, fixture)은 devel(#1548 머지본)과 동일하여 실질 델타 0, 중복 없음.

## 3. 코드 검토 (layout.rs 4가지)

1. 제목이 자기 표 아래로 안 밀림 — `VisibleFloatExclusion.owner_para` 추가, consume 시 같은
   문단 텍스트는 자기 표 zone skip.
2. 후행 float 흡수 offset 복원 — #1535 클램프를 `is_para_topbottom_float` 로 확대, 빈-host 는
   zone 하단 아래로 자기 offset 만큼 복원(표-표 간격).
3. 제목/표 겹침·아래 간격 — visible-host 양수 float 표를 host 제목 줄 아래로 클램프. 큰 offset 은
   `max` 라 영향 없고 `v_off==0` 보존(#1510).
4. 표/다음 섹션 간격 — float exclusion 하단을 `outer_margin_bottom` 만큼 확장.

typeset PageItem 순서·`current_height` 누적은 무변경, layout 좌표 클램프만 조정.

## 4. 검증

### 자동 (로컬 — CI 대체)

| 항목 | 결과 |
|---|---|
| GitHub CI | no checks (fork PR) → 로컬 전체 회귀로 대체 |
| 충돌 시뮬레이션 | 0건 (#1548 머지본 위) |
| 신규 `issue_1549` (2) + 회귀 `issue_1535`(1)/`issue_1510`(4) | 통과 |
| **golden SVG 불변** (svg_snapshot) | 8/8 |
| `hwpx_roundtrip_baseline` | 4/4 |
| 전체 `cargo test --tests` | **FAILED 0건** (lib 1937 passed) |
| fmt / clippy | clean |

### 시각 판정 (게이트)

- 산출물: `output/poc/pr1568/` 일반 SVG 3종 + debug 오버레이 2종(문단/표 경계 + y좌표),
  원본 fixture `src_fixtures/`.
- 작업지시자 시각 판정 **통과** (제목 위치, 제목↔표·표↔표·표↔다음섹션 간격 한컴 정합).
- golden 불변 + `issue_1510` page_count==1 유지로 페이지네이션 영향 없음.

## 5. 첫 기여 후속

kkyu8925 의 두 번째 기여(#1548 후속). CONTRIBUTING 가이드 준수 + "메인테이너 시각 재검증
요청"을 직접 명시한 신중한 PR. 자기검증 ≠ 한컴 호환 원칙에 맞춰 시각 판정 게이트를 거쳐 merge.

## 6. 산출물

- 본 처리 보고서: `mydocs/pr/archives/pr_1568_review.md`
