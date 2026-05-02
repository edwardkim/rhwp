# PR #397 검토 — 수식 ATOP 파싱 및 렌더링 보정

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#397](https://github.com/edwardkim/rhwp/pull/397) |
| 작성자 | [@cskwork](https://github.com/cskwork) (Agentic-Worker) — **신규 컨트리뷰터** |
| base / head | `devel` (`94d9347`) ← `cskwork:fix/atop-equation-parser` |
| state | OPEN |
| mergeable | MERGEABLE |
| mergeStateStatus | BEHIND (PR #396 머지 전 base) |
| 변경 통계 | +115 / -19, 4 files |
| **CI** | **statusCheckRollup 비어있음** — CI 실행 안 됨 |
| 이슈 | (없음 — 자체 발견 결함 정정) |
| 정황 | 신규 컨트리뷰터 — 작업지시자 git 정보 확인으로 실제 사람 컨트리뷰터 확정 |

## 작성자 정황

- @cskwork (Agentic-Worker, 본인 표기) — **머지 이력 0건** (이번이 첫 PR, OPEN PR 2개: #397, #400)
- 작업지시자 git 정보 확인 결과 — 실제 사람 컨트리뷰터 (AI 에이전트 아님)
- 신규 컨트리뷰터이므로 메모리 (`feedback_v076_regression_origin.md`) 의 외부 컨트리뷰터 검증 게이트 적용
- 첫 PR 이라 절차 안내 필요 — 시각 검증 자료 / base rebase / CI 트리거

## 변경 내용

### 결함 (작성자 분석)

한컴 수식 문법:
- `a OVER b` → 분수 (분수선 있음)
- `a ATOP b` → 위/아래 배치 (분수선 **없음**)

기존 코드:
- `EqNode::Atop` AST 타입은 있으나 파싱 / 레이아웃 / 렌더링이 미완성
- 파서가 `ATOP` 명령을 빈 노드로 처리
- 레이아웃이 `EqNode::Atop` 을 `layout_fraction` 으로 처리 → 분수선이 그려짐
- SVG/Canvas 렌더러에 `LayoutKind::Atop` 분기 없음

### 정정 (4 files)

#### 1. `parser.rs`

`OVER` 의 중위 연산자 처리에 `ATOP` 병합:
- `parse_expression` (top-level)
- `parse_group` (`{...}` 내부)

```rust
if Self::cmd_eq(self.current_value(), "OVER") || Self::cmd_eq(self.current_value(), "ATOP") {
    let is_atop = Self::cmd_eq(self.current_value(), "ATOP");
    // ... pop / parse_element ...
    children.push(if is_atop {
        EqNode::Atop { top, bottom }
    } else {
        EqNode::Fraction { numer, denom }
    });
}
```

#### 2. `layout.rs`

`LayoutKind::Atop { top, bottom }` 추가 + `layout_atop()` 신규:

```rust
fn layout_atop(&self, top: &EqNode, bottom: &EqNode, fs: f64) -> LayoutBox {
    let pad = fs * FRAC_LINE_PAD;
    let axis = fs * AXIS_HEIGHT;
    let w = t.width.max(b.width) + pad * 2.0;
    let top_h = t.height + pad;
    let bottom_h = b.height + pad;
    let baseline = top_h + axis;
    let total_h = top_h + bottom_h;
    // top 박스 중앙 정렬, y = pad
    // bottom 박스 중앙 정렬, y = top_h
}
```

`layout_node` 의 `EqNode::Atop` 분기를 `layout_fraction` → `layout_atop` 으로 변경.

#### 3. `svg_render.rs`

```rust
LayoutKind::Atop { top, bottom } => {
    render_box(svg, top, x, y, color, fs, italic, bold);
    render_box(svg, bottom, x, y, color, fs, italic, bold);
}
```

분수선 (`<line>`) 없이 top/bottom 만 렌더 — 정확.

#### 4. `canvas_render.rs`

같은 패턴. `<line>` 없음.

### 단위 테스트

- `parser::tests::test_atop` — `a atop b` 파싱 → `EqNode::Atop` 검증
- `svg_render::tests::test_atop_svg_has_no_fraction_line` — SVG 에 `<line>` 없음 + 두 텍스트의 y 좌표 다름 검증

## 평가

### 강점

1. **결함 분석 명확** — 기존 코드의 미완성 흐름 정확히 짚음 (AST 타입 있으나 파싱/렌더 미완)
2. **의도 분리 명확** — `Fraction` (분수선 O) vs `Atop` (분수선 X)
3. **변경 범위 작음** — 4 files / +115 / -19 (deletion 적음)
4. **단위 테스트 양호** — 파서 + SVG 렌더 양쪽
5. **layout_atop 코드 합리** — Fraction 기준에서 `line_thick` 만 빼고 baseline/total_h 조정
6. **dry-run merge** — devel 자동 머지 성공
7. **테스트 통과** — 본 검토에서 1033 passed (1031 → +2 신규)

### 약점 / 점검 필요

#### 1. CI 실행 안 됨

`statusCheckRollup` 비어있음. base 가 PR #396 머지 전 (`94d9347`) 이라 BEHIND 상태이지만, CI 자체가 실행 안 된 것은 비정상.
- 가능성: 신규 컨트리뷰터의 첫 PR 이라 GitHub 가 자동 CI 트리거 안 한 정황
- **rebase 후 CI 재실행 필요** 또는 **본 검토에서 직접 cargo test 검증 통과 확인** (수행 완료)

#### 2. 신규 컨트리뷰터 (실제 사람 — 작업지시자 git 정보 확인)

- 머지 이력 0건
- 코드 자체는 합리적이지만 첫 PR 이라 절차 안내 + 시각 판정 필요
- 작업지시자 결정: 향후 지속적인 PR 을 위해 절차 준수 안내

#### 3. 시각 검증 자료 부재

- 본 PR 본문에 시각 비교 (한컴 출력 vs rhwp 출력) 없음
- ATOP 수식이 포함된 hwp 샘플로 회귀 점검 필요
- 작업지시자 시각 판정 필수

#### 4. clippy 정황 (PR 본문)

작성자가 PR 본문에 명시:
> `cargo clippy -- -D warnings`는 기존 코드의 `clippy::uninlined_format_args` 716건으로 실패

본 검토에서 `cargo clippy --lib -- -D warnings` 통과 — 작성자 환경 정황 (clippy 버전 차이) 으로 추정. 본 PR 의 직접 영향은 없음.

## 메인테이너 작업과의 관계

### 충돌 가능성

본 PR 이 변경한 4 files 중 PR #396 (오늘 머지) 가 변경한 파일:
- `canvas_render.rs` — PR #396 후속 정정 (분수선 / Limit) 영역과 다른 라인 → 자동 머지 ✅
- `layout.rs` — PR #396 변경 영역 (CJK width 보정) 과 다른 함수 → 자동 머지 ✅
- `svg_render.rs` — PR #396 변경 영역 (CJK italic) 과 다른 분기 → 자동 머지 ✅
- `parser.rs` — PR #396 미변경 → 자동 머지 ✅

dry-run merge 검증 통과 — 1033 passed.

## 처리 방향 후보

### 옵션 A: cherry-pick (PR #395, #396 와 같은 패턴)

- `local/pr397` 브랜치에서 cherry-pick (작성자 attribution 보존)
- WASM 빌드 후 dev server 시각 판정 (ATOP 포함 hwp 샘플 필요)
- 통과 시 머지

### 옵션 B: 작성자에게 시각 검증 자료 + base rebase 요청

- ATOP 포함 hwp 샘플로 한컴 출력 vs rhwp 시각 비교 자료 요청
- devel 기반 rebase + CI 통과 후 다시 검토

### 옵션 C: 거절 / close

- 결함이 사전에 알려진 것이 아니거나 회귀 위험 큰 경우. 본 PR 은 합리적이라 추천 안 함.

## 권장

**옵션 A (cherry-pick + 시각 판정 게이트)**:

이유:
1. 코드 자체는 합리적 (한컴 ATOP 의미 정확히 분리)
2. 변경 범위 작음 + 단위 테스트 양호
3. 자동 머지 + 1033 passed 검증 통과
4. PR #395, #396 와 같은 패턴이라 일관성

**단** 다음 절차 엄격 준수:
1. cherry-pick → WASM 빌드 → **시각 판정 게이트 (push 전 필수)**
2. ATOP 수식이 포함된 hwp 샘플 시각 비교
3. 작업지시자 시각 판정 통과 시에만 push + close

### ATOP 수식 포함 hwp 샘플 후보

저장소 샘플 검색 필요 (`exam_math.hwp` 외 다른 샘플 가능성). 또는 작업지시자가 ATOP 사용 hwp 보유 여부 확인.

## 검토 항목 (Claude 점검 완료)

- [x] 작성자 신뢰도 — 신규 컨트리뷰터 (시각 판정 게이트 강화) ⚠️
- [x] 코드 품질 — 합리적 ✅
- [x] 변경 범위 — 작음 ✅
- [x] dry-run merge — 자동 성공 ✅
- [x] cargo test --lib — 1033 passed ✅
- [x] cargo test --test svg_snapshot — 6/6 ✅
- [x] cargo test --test issue_418 — 1/1 (Task #418 보존) ✅
- [x] cargo clippy — warning 0 ✅
- [ ] CI 실행 — **statusCheckRollup 비어있음** ⚠️
- [ ] 시각 판정 — ATOP 수식 hwp 샘플 + 작업지시자 시각 판정 필수

## 작업지시자 결정 — 옵션 B

작업지시자 결정 (2026-04-28):
- **B 선택**: 작성자에게 시각 검증 자료 + base rebase 요청 + 절차 안내
- 정황: 신규 컨트리뷰터의 첫 PR 이므로 향후 지속적인 PR 을 위해 정해진 절차 (시각 검증 / base rebase / CI 트리거) 준수 학습 기회 제공
- AI 에이전트 가능성은 작업지시자 git 정보 확인으로 배제됨

## 다음 단계

1. 작성자에게 댓글 — 시각 검증 자료 / base rebase / CI 트리거 안내
2. 작성자 재제출 시 (rebase + 시각 자료 첨부) 다시 검토
3. 본 PR 은 OPEN 유지 (close 안 함)

## 참고

- PR: [#397](https://github.com/edwardkim/rhwp/pull/397) (OPEN, BEHIND)
- 작성자 다른 OPEN PR: [#400](https://github.com/edwardkim/rhwp/pull/400) (HWPX 수식 직렬화 보존)
- 관련 PR: [#396](https://github.com/edwardkim/rhwp/pull/396) (수식 렌더링 — TAC 높이 + 한글 이탤릭, 머지 완료)
