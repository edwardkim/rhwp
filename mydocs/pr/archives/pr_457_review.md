# PR #457 검토 — Task #455 인라인 글상자 외부 본문 텍스트 누락 정정

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#457](https://github.com/edwardkim/rhwp/pull/457) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) — 본 사이클 15번째 PR |
| 이슈 | [#455](https://github.com/edwardkim/rhwp/issues/455) (closes) |
| base / head | `devel` ← `planet6897:local/devel` |
| 변경 규모 | +2,243 / -619, 25 files (PR #454 누적 포함) |
| mergeable | `MERGEABLE` (BEHIND) |
| 검토 일자 | 2026-04-30 |

## 본질

`samples/exam_kor.hwp` 페이지 2 좌측 단의 문단 pi=33 line 2 에서 본문 39자가 누락되고 인라인 글상자 내부 "개화" 두 글자만 표시되던 결함 정정.

### 원인

`paragraph_layout.rs::layout_composed_paragraph` 의 tac 분기 처리 블록에서:

```rust
let skip_text_for_inline_shape = has_tac_shape && para.map(|p| {
    tac_offsets_px.iter().any(|(_, _, ci)| {
        if let Some(Control::Shape(s)) = p.controls.get(*ci) {
            s.drawing().map(|d| d.text_box.is_some()).unwrap_or(false)
        } else { false }
    })
}).unwrap_or(false);

if !skip_text_for_inline_shape {
    line_node.children.push(sub_run_node);  // tac 앞 텍스트
}
```

스킵된 "텍스트" 는 글상자의 *외부 문단 본문* 이지 *내부* ("개화") 가 아님. 외부 본문은 글상자 좌·우를 흐르는 일반 텍스트이며 항상 렌더되어야 함.

### 정정

`skip_text_for_inline_shape` 변수 + 두 곳의 가드 제거. 외부 본문 항상 렌더 — 글상자 자체와 내부 텍스트는 `shape_layout` 의 `inline_shape_position` 경로로 별도 렌더되므로 중복 없음.

### 효과 (작성자 명시)

수정 전 (좌측 단):
```
y=295.1 "서양의 과학과 기술... 이항로를 비롯한"
y=321.4 "개화"        ← 본문 39자 누락
y=347.6 "수없는 대세로 자리잡았다..."
```

수정 후:
```
y=295.1 "서양의 과학과 기술... 이항로를 비롯한"
y=321.4 "개화"        ← 글상자 (별도 패스)
y=322.6 "척사파의 주장은 개항 이후에도 지속되었지만, 는 거스를"   ← 본문 39자 복원
y=347.6 "수없는 대세로 자리잡았다..."
```

## 처리 방향

**옵션 A — Task #455 본질 2 commits 분리 cherry-pick** (PR #454 와 같은 패턴).

PR #454 가 먼저 머지된 상태에서 본 PR cherry-pick 진행 → 작성자 누적 commit 분리 처리.

## dry-run cherry-pick 결과

`local/pr457` 브랜치 (`local/devel` 분기 — PR #454 머지 후) — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `c30294e` (← `d87af59`) | @planet6897 | Stage 2: 인라인 글상자(tac=true + TextBox) 외부 본문 텍스트 누락 수정 |
| `b6a95a5` (← `bfe763f`) | @planet6897 | Stage 3+4: 최종 결과보고서 + PR 메시지 + 오늘할일 갱신 |

cherry-pick 결과: 충돌 없이 자동 적용 (PR #454 와 다른 영역).

## 검증 게이트 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1069 passed** (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |

## 광범위 byte 단위 비교

10 샘플 / 305 페이지 SVG 비교 (PR #454 머지 후 devel ↔ PR #457):

| 결과 | 카운트 |
|------|------|
| byte 동일 | **292 / 305 (95.7%)** |
| 차이 발생 | **13 / 305 (4.3%)** |

차이 분포 (영향 범위 한정):
- **exam_* 11 페이지** (작성자 명시 — exam_kor 페이지 2 좌측 단 + 다른 exam_* 의 인라인 글상자 영역)
- **synam-001 1, k-water-rfp 1** (영향 범위)

→ PR #454 의 광범위 영향 (87%) 과 대조적 — Task #455 는 매우 정밀 정합 (특정 결함 영역만).

## 시각 판정 정황 (작업지시자 결정)

작업지시자 결정 (PR #454 와 동일 정책):
> "메인테이너는 이 PR 처리를 끝 낸 후 시각적 검증을 하겠습니다."

본 PR 단독 시각 판정 보류 — 후속 PR (#461, 5 Tasks) 처리 후 통합 시각 검증 진행.

## 본 PR 의 좋은 점

1. **정확한 본질 진단**: `skip_text_for_inline_shape` 가 외부 본문을 글상자 내부와 혼동한 정황 정확 식별
2. **변경 범위 한정**: 단일 파일 (paragraph_layout.rs) 의 가드 제거만 — 영향 범위 13 페이지로 한정
3. **회귀 검증** (작성자 본문): 8 샘플 (exam_kor / exam_eng / 2010-01-06 / exam_math_8 / biz_plan / draw-group / atop-equation-01 / equation-lim) 페이지 수 동일 유지
4. **알려진 minor 정황 명시**: 본문 글자 baseline y=322.6 vs 글상자 내부 "개화" baseline y=321.4 — 1.2px 미세 어긋남, 별도 이슈 분리 가능 (작성자 본문 명시)

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1069 + svg_snapshot 6/6 + clippy 0 |
| 시각 판정 게이트 (push 전 필수) | ⏸️ 후속 PR 처리 후 통합 검증 (작업지시자 결정) |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | ✅ 가드 제거 (외부 본문은 항상 렌더 — `shape_layout::inline_shape_position` 경로 분리 인지) |
| 작은 단위 PATCH 회전 | ✅ Task #455 본질 2 commits 만 분리 |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr457` 에서 커밋 |

## 다음 단계

1. 본 보고서 commit
2. `local/pr457` → `local/devel` → `devel` 머지 + push
3. PR #457 close + 작성자 댓글 (이슈 #455 자동 close)
4. **PR #461 (Task #459 + 4 Tasks 누적) 처리 진행**

## 참고

- PR: [#457](https://github.com/edwardkim/rhwp/pull/457)
- 이슈: [#455](https://github.com/edwardkim/rhwp/issues/455)
- 직전 PR: [#454](https://github.com/edwardkim/rhwp/pull/454) (Task #452)
- 다음 PR (5 Tasks 누적): [#461](https://github.com/edwardkim/rhwp/pull/461) (Task #459/#462/#463/#468/#469)
