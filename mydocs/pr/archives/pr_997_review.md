# PR #997 검토 — fix: HWP5 line_segs 누락 paragraph 겹침 해소 (composer word wrap 합성)

- 작성일: 2026-05-19
- 컨트리뷰터: [@jangster77](https://github.com/jangster77) (Taesup Jang)
- PR: https://github.com/edwardkim/rhwp/pull/997
- base/head: `devel` ← `jangster77:local/task994` (cross-repo fork)
- 연결 이슈: closes #994 (HWP5 long text paragraph line_segs 누락 시 layout 겹침, sample16 page 22)
- 규모: +572 / -18, 7 files (소스 1: `src/renderer/composer.rs`, 문서 6)
- mergeable: **MERGEABLE**

## 1. PR 정보 확인

| 항목 | 값 |
|------|----|
| 본질 변경 | `src/renderer/composer.rs` `compose_lines` fallback (line_segs.is_empty 경로) |
| 이슈 #994 | OPEN, **assignee 없음** (PR 처리 절차상 확인 필요) |
| CI | 확인 필요 (Build & Test / CodeQL) |
| base 동기화 | 현 devel `3923b693`, PR base 동일 계열 |

## 2. 컨트리뷰터 사이클 점검 (`feedback_contributor_cycle_check`)

@jangster77 = **24+ 사이클 핵심 컨트리뷰터**. 직전 #995 (closes #991, CLOSED — composer marker synthesis).

**⚠️ PR 시리즈 연속 (상호 의존 주의):**

| PR | closes | 본질 | 상태 |
|----|--------|------|------|
| **#997** | #994 | sample16 line_segs 누락 겹침 (시각) | 본 PR |
| #999 | #998 | HWP5 sample16 페이지 수 HWP3 정합 (64) | OPEN |
| #1005 | #1001 | HWP5/HWP3 한컴 정합 종합 A/B/C/D | OPEN |
| #1009 | #1007 | HWP5 페이지 강제 나눔 정합 | OPEN |
| #1011 | #1006 | 쪽 테두리 포맷별 분리 | OPEN |

#997(겹침 시각) → #999(페이지 수)는 sample16 동일 대상의 연쇄. **순차 처리** 필요. 본 PR이 페이지 수를 62→67로 바꾸므로 #999가 그 위에서 64로 정합하는 의존 구조로 보임.

## 3. 변경 내용 분석

`compose_lines(para)` 의 `para.line_segs.is_empty()` fallback 경로:

- **기존**: 전체 텍스트 단일 ComposedLine → layout 이 wrap 없이 한 y 에 모든 char → 시각 겹침
- **변경**: word-boundary 기반 `CHARS_PER_LINE=35` 분할, 각 줄 ComposedLine. non-last line `has_line_break=true` (Justify 비활성화 → mid-word spacing 부풀림 회피)

3차 반복 (PR 본문): G1(효과 없음, 폐기) → G4-wide(겹침 해소 but Justify 부풀림) → **G4-final(word boundary + has_line_break)**.

## 4. 검토 의견

### 강점

1. **실제 결함 해소** — sample16 page 19~24 PUA bullet (`󰏅`) 59개 paragraph 줄겹침. 작업지시자 시각 판정 통과 (PR 본문).
2. **HWP3 전용 분기 아님** — `compose_lines`는 공통 렌더러 모듈이나, 변경은 `line_segs.is_empty()` 일반 조건. CLAUDE.md HWP3 규칙(공통 모듈에 HWP3 전용 분기 금지) 위반 아님.
3. **Justify 부풀림 회피 설계** — `has_line_break=!is_last_line` 로 합성 wrap line 의 강제 양끝정렬 차단. 마지막 줄은 기존 동작 유지.
4. **주석에 임시 휴리스틱 명시** — "향후 reflow_line_segs 정식 호출 시 본 휴리스틱 대체" 명기. 기술 부채 가시화.
5. **검증 수행** — cargo test 1297 / fmt / 240 sample sweep.

### ⚠️ 핵심 쟁점

#### (A) CHARS_PER_LINE=35 매직 넘버 — 측정 무관 하드코딩

컬럼 너비·폰트 크기와 무관하게 35자 고정 분할. PR 주석은 "Korean 13pt 표준"이라 하나, 실제 컬럼 폭/글꼴 크기가 다르면 줄당 글자수가 부정확 → 줄 길이 들쭉날쭉 가능. `feedback_hancom_compat_specific_over_general` 관점에서 **측정 의존이 아닌 고정 휴리스틱**은 다른 샘플 회귀 위험. 다만 PR 본문이 임시 휴리스틱임을 명시하고 향후 reflow_line_segs 대체를 약속.

#### (B) Task #671 `recompose_for_cell_width` 셀 경로와 가드 충돌 가능성 (`feedback_fix_scope_check_two_paths`)

이미 **Task #671 의 `recompose_for_cell_width`** (composer.rs:1131) 가 동일 본질(line_segs 빈 paragraph 단일 ComposedLine 줄겹침)을 **셀 가용 너비 측정 기반으로 정밀 해결** 중. 가드:
- `para.line_segs.is_empty()` ✓
- **`composed.lines.len() != 1` → 즉시 return**
- 실제 측정 폭 vs `cell_inner_width_px` 초과 시에만 단어 분할

호출 경로: `table_partial.rs` / `table_layout.rs` / `height_measurer.rs` (**셀 paragraph 전용**).

**위험**: PR #997 이 `compose_lines` 진입점에서 35자 분할하면, **셀 안 line_segs 빈 long-text paragraph 가 multi-line 으로 진입** → `recompose_for_cell_width` 의 `composed.lines.len() != 1` 가드에 걸려 **셀 너비 기반 정밀 재분할(Task #671)이 비활성화**. 35자 고정 분할은 셀 실제 너비와 무관하므로 셀 안에서 부정확. **PR 이 인정한 hy-001 신규 페이지 변동 1건의 유력한 원인** (hy-001 = 표 포함 문서).

#### (C) Page count 잔존 + HWPX 변종 영향 (PR 본문 인정)

- 타깃 페이지 수 62→67, 신규 hy-001 변동 1건
- Residual: HWP3 64 vs HWP5(post G4) 67 vs HWPX 69 — paragraph height 누적 차이 별도 root cause (#999 로 추정)
- "HWPX 변종 영향 가능 (parser path 동일)" — PR scope 외로 명시

### 확인 필요 (검증 단계)

1. cherry-pick devel 적용 (충돌 여부)
2. `cargo test --release --lib` (PR: 1297, 현 devel 기준 1307+ 예상) + clippy -D + fmt 0
3. **광범위 sweep** (`scripts/svg_regression_diff.sh`) — 특히 **표 포함 샘플(hy-001 등) 셀 내부 line_segs 빈 paragraph 회귀** 집중 확인 (쟁점 B)
4. 작업지시자 시각 판정 — sample16 page 19~24 겹침 해소 + hy-001 변동이 회귀인지 정합인지

## 5. 처리 옵션

- **옵션 A (수용)**: 실제 시각 결함 해소 + 작업지시자 시각 판정 + sweep 회귀 부재 확인 시. 단 쟁점 B(셀 경로 충돌) 검증 + hy-001 변동의 회귀/정합 작업지시자 판정 필수.
- **옵션 B (수정 요청)**: 쟁점 B 가 hy-001 셀 줄겹침 회귀로 확인되면 — `compose_lines` fallback 이 셀 컨텍스트에서 `recompose_for_cell_width` 와 양립하도록 조건 추가 요청 (예: 셀 paragraph 는 35자 분할 skip, recompose_for_cell_width 에 위임).
- **옵션 C (close)**: 본질 결함 시. 현재 실제 결함 해소하므로 해당 없음.

## 6. 메모리 룰 정합

- `feedback_contributor_cycle_check` — @jangster77 24+ 사이클, #997~#1011 연속 시리즈 순차 처리
- `feedback_fix_scope_check_two_paths` — **쟁점 B**: compose_lines 정정만으로 부족, Task #671 recompose_for_cell_width 셀 경로 양립 점검 필수 (권위 사례)
- `feedback_hancom_compat_specific_over_general` — **쟁점 A**: CHARS_PER_LINE=35 측정 무관 고정 휴리스틱, 일반화 위험. 임시 명시는 완화 요소
- `feedback_visual_judgment_authority` — sample16 겹침 + hy-001 변동, 작업지시자 시각 판정이 회귀/정합 최종 판정
- `feedback_image_renderer_paths_separate` — compose_lines 는 단일 진입점이나 셀/비셀 경로 분기 영향 점검
- `feedback_assign_issue_before_work` — 이슈 #994 assignee 없음 (외부 기여자 기 작업분이므로 해당 없으나 기록)

## 7. 권고

**옵션 A 조건부** — 검증 단계에서 (1) cargo test/clippy/fmt GREEN, (2) **쟁점 B(셀 경로 hy-001 회귀) sweep + 작업지시자 시각 판정으로 회귀 부재 확인**, (3) sample16 page 19~24 겹침 해소 시각 판정 통과 시 cherry-pick no-ff merge. 쟁점 B 가 회귀로 확인되면 옵션 B(셀 경로 양립 조건 추가) 전환. 페이지 수 잔존은 #999 후속 시리즈로 분리 (PR 본문 정합).
