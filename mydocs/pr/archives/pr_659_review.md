# PR #659 검토 보고서

**PR**: [#659 feat: ir-diff 표 속성 / 컨트롤 식별 비교 추가 (closes #653)](https://github.com/edwardkim/rhwp/pull/659)
**작성자**: @oksure (Hyunwoo Park) — 6번째 사이클 PR (PR #581/#582/#583/#600/#601/#602 + 본 PR)
**상태**: OPEN, **mergeable=MERGEABLE**, **mergeStateStatus=BEHIND** (PR base `30351cdf` = 5/5 PR #601 처리 후속 시점, 5 commits 뒤)
**관련**: closes #653 (메인테이너 직접 등록 + 외부 컨트리뷰터 선구현 패턴)
**처리 결정**: ⏳ **검토 중**
**검토 시작일**: 2026-05-07

## 1. 검토 핵심 질문

1. **본질 영역 정합성** — Issue #653 (`ir-diff` 보강 — 표 속성 / paragraph line_segs / controls 식별 가능 단위 비교) 의 요청 영역을 본 PR 이 정확히 커버하는가?
2. **메인테이너 → 컨트리뷰터 선구현 패턴** — 메인테이너 직접 등록 (5/7 다른 환경, HWPX 시리얼라이제이션 영역 작업 중) 한 이슈를 외부 컨트리뷰터가 같은 일자에 선구현 — 이슈 등록 후 발견 + cherry-pick 후 메인테이너 사이클 단축 정합 가치
3. **회귀 위험** — `src/main.rs` ir-diff 명령에만 변경 (+144/-3, 진단 도구 영역). 코어 IR / 렌더 / 직렬화 영역 무영향
4. **Copilot 리뷰 응답 정합** — Copilot 4 코멘트 모두 PR 제출자가 마지막 commit (`eb97d9b` "address review") 에서 반영
5. **PR base skew (5 commits 뒤)** — 본 환경 cherry-pick 충돌 0?

## 2. PR 정보

| 항목 | 값 | 평가 |
|------|-----|------|
| 제목 | feat: ir-diff 표 속성 / 컨트롤 식별 비교 추가 (closes #653) | 정합 (Issue #653 본질 그대로) |
| author | @oksure (Hyunwoo Park, oksure@gmail.com) — 활발한 컨트리뷰터 (6번째 PR) | ✅ |
| changedFiles | **2** / +144 / -3 | 작은 규모 |
| 본질 변경 | `src/main.rs` ir-diff 명령 (+140/-3) + `CLAUDE.md` (+4/-1) | 진단 도구 영역만 |
| **mergeable** | MERGEABLE (UI), **mergeStateStatus=BEHIND** (5 commits 뒤) | 본 환경 cherry-pick 충돌 0 확인 필요 |
| Issue | closes #653 (메인테이너 직접 등록, M100 v1.0.0) | ✅ |
| Issue assignee | 미지정 | 본 환경 처리 시 점검 (memory `feedback_assign_issue_before_work`) |
| commits | **3** (`3645d69` 초안 + `8d5886c` LINE_SEG 확장 + `eb97d9b` Copilot 리뷰 응답) | 단계 commit 패턴 정합 |

## 3. PR 의 3 commits 분석

### Commit 1: `3645d69` "feat: ir-diff 표 속성 / 컨트롤 식별 비교 추가 (closes #653)"

**핵심 추가**:
- `control_tag(c)` 함수: Control variant → "tbl"/"pic"/"shape" 등 22 variant 매핑
- `diff_table` 함수: 표의 `row_count`/`col_count`/`page_break`/`repeat_header`/`cell_spacing`/`border_fill_id`/`outer_margin` (4방향) + `CommonObjAttr` 비교
- `diff_common_obj` 함수: `treat_as_char`/`text_wrap`/`size`(width×height)/`v_offset`/`h_offset`/`vert_rel_to`/`horz_rel_to` 비교
- `ir_diff` 의 컨트롤 비교부 확장: `pa.controls.len() != pb.controls.len()` → 식별 가능 단위 비교 (Table/Picture/Shape match + 타입 불일치 보고)

### Commit 2: `8d5886c` "feat: ir-diff LINE_SEG 전체 필드 비교 확장"

**핵심 추가**:
- 기존 `text_start`/`line_height`/`segment_width` 3 필드 비교 → 8 필드 (+5):
  - `vertical_pos` (vpos)
  - `text_height` (th)
  - `baseline_distance` (bl)
  - `line_spacing` (ls)
  - `column_start` (cs)

### Commit 3: `eb97d9b` "address review: 출력 키 정합 + match 리팩터 + 문서 정정"

**Copilot 4 코멘트 응답**:
- ✅ #1 `border_fill` → `border_fill_id` (필드명 1:1 매칭)
- ✅ #2 `voff`/`hoff` → `v_offset`/`h_offset` (명확한 키)
- ✅ #3 `if-let 체인` → `single match` (exhaustiveness 활용)
- ✅ #4 `help text` / `CLAUDE.md` 정정 (표/그림/도형 비교 범위 분리 명시)

## 4. 영역 평가

### 4.1 Issue #653 의 요청 영역 정합성

**Issue #653 요청 영역 (메인테이너 직접 등록)**:
- ✅ **표의 `page_break` attr** (RowBreak vs CellBreak — Issue #652 의 본질 #1) — `diff_table` 에서 처리
- ✅ **표의 `treat_as_char` / `wrap`** 속성 — `diff_common_obj` 에서 처리
- ⚠️ **표의 `cells` / `is_header` / `border_fill_id`** — `border_fill_id` 만 처리 (`cells` / `is_header` 미처리)
- ✅ **paragraph 의 `line_segs` 범위 차이** (`lines=0..2` vs `lines=1..2`) — Commit 2 LINE_SEG 8 필드 확장으로 검출 가능
- ✅ **controls 의 식별 가능 단위 비교** — `control_tag` + 타입별 match 으로 검출 가능

**커버리지**: ~85% (cells/is_header 영역 미커버이나 Issue #652 의 본질 #1, #2 모두 검출 가능)

### 4.2 메인테이너 → 컨트리뷰터 선구현 패턴

**패턴 본질**: 메인테이너가 5/5 PR #601 검토 영역에서 발견 → Issue #653 등록 (5/5 다른 환경) → 외부 컨트리뷰터가 5/7 같은 일자에 선구현 PR 제출.

**가치**:
- 메인테이너 사이클 단축 (Issue #653 → 직접 구현 미수행, 외부 흡수)
- "공동체 비전" (`user_role_identity` 정합) — v1.0.0+ 부터 커뮤니티 참여 개방의 권위 사례
- @oksure 의 활발한 사이클 (6번째 PR, PR #581/#582/#583/#600/#601 누적) 정합

### 4.3 회귀 위험 영역 분석

**범위**: `src/main.rs` ir-diff 명령 영역 + `CLAUDE.md` 문서.
**무영향 영역**: 코어 IR (`src/model/`), 렌더 (`src/renderer/`), 직렬화 (`src/serializer/`), HWP3/HWP5/HWPX 파서.
**영향 영역**: ir-diff 명령 출력 포맷 (사용자-facing). 기존 출력 포맷과 호환:
- 기존: `controls: A=N vs B=M` → 신규: `controls count: A=N vs B=M` (메시지 키 변경 — grep 의존 사용자 영향 가능)
- 기존: `ls[i].ts/lh/sw` → 신규: 5 필드 추가 (vpos/th/bl/ls/cs)

**회귀 위험**: 매우 낮음 (진단 도구만, 코어 무영향).

### 4.4 Copilot 리뷰 응답 정합성

Copilot 4 코멘트 모두 마지막 commit `eb97d9b` 에서 반영. 외부 컨트리뷰터의 자체 검토 응답 패턴 정합 (`feedback_assign_issue_before_work` 영역 + `user_work_style` 정합 — 자체 검토 응답 직접 결정 영역).

### 4.5 PR base skew

**fork base**: `30351cdf` (5/5 PR #601 처리 후속 시점)
**devel ahead**: 5 commits (5/5+5/7 — PR #601 처리 후속, Issue #652 등록, HWPX 시리얼라이제이션 가설 보고서 + Issue #655)
**충돌 위험**: src/main.rs 의 ir_diff 함수 영역만 변경 — devel 의 5 commits 는 mydocs/ 와 PR #601 처리 후속 영역 (코어 변경 없음). **cherry-pick 충돌 0 예상**, 검증 단계에서 확정.

## 5. 옵션 평가

### 옵션 A: 3 commits cherry-pick (권장)

**범위**: `3645d69` + `8d5886c` + `eb97d9b` 모두 cherry-pick (squash 또는 보존).

**장점**:
- Issue #653 본질 영역 거의 모두 커버 (~85%)
- Copilot 리뷰 응답 정합 (마지막 commit `eb97d9b` 포함)
- 회귀 위험 매우 낮음 (진단 도구 영역)
- 활발한 컨트리뷰터 6번째 PR 흡수 패턴 정합

**단점**: `cells` / `is_header` 영역 미커버 — 후속 권유 또는 별도 사이클

**처리**:
- 3 commits 보존 cherry-pick (author @oksure 보존)
- Issue #653 의 미커버 영역 (`cells` / `is_header`) 후속 PR 권유 댓글 등록
- 메인테이너 → 컨트리뷰터 선구현 패턴 인정 댓글

### 옵션 B: squash cherry-pick

3 commits 을 1 commit 으로 squash. Copilot 리뷰 응답 commit 분리 의도가 손실되므로 **비권장**.

### 옵션 C: close + 후속 권유

본 PR 이 Issue #653 본질 ~85% 커버하므로 close 비합리적. **비채택**.

## 6. 처리 권장

**옵션 A (3 commits 보존 cherry-pick)** 권장.

**이유**:
1. Issue #653 본질 ~85% 커버
2. 메인테이너 → 컨트리뷰터 선구현 패턴의 권위 케이스
3. 회귀 위험 매우 낮음 (진단 도구 영역, 코어 무영향)
4. Copilot 리뷰 응답 정합
5. 활발한 컨트리뷰터 (6번째 PR) 사이클 흡수

**후속 권유**: `cells` / `is_header` 비교 영역 — 별도 PR 또는 후속 task.

## 7. 본 환경 검증 계획 (구현계획서 분리 불필요 영역)

PR 이 진단 도구 영역만 변경 + 회귀 위험 매우 낮음 → 구현계획서 단계 생략, 검증 단계만 진행:

1. cherry-pick 3 commits → 본 환경 충돌 0 확인
2. cargo test --lib 1141 passed 유지
3. cargo clippy -- -D warnings 통과
4. cargo build --release 통과
5. **결정적 검증**: `rhwp ir-diff samples/hwpx/aift.hwpx samples/aift.hwp` 실행 → Issue #653 명시 영역 검출 확인 (page_break / treat_as_char / wrap / line_segs vpos)
6. WASM 빌드 영향 0 (main.rs 영역, WASM 미포함)
7. devel merge + push
8. PR #659 close + Issue #653 close

**시각 판정 영역 부재**: 진단 도구 영역 — 결정적 검증 (ir-diff 출력 비교) 만으로 충분.
