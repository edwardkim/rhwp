# PR #531 처리 보고서

**PR**: [#531 fix(layout): TAC inline 경로 Top caption 오프셋 누락 정정 (Task #530)](https://github.com/edwardkim/rhwp/pull/531)
**작성자**: @postmelee (Taegyu Lee) — 외부 컨트리뷰터
**처리 결정**: ✅ **머지** (cherry-pick @postmelee 3 commits)
**처리일**: 2026-05-03

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 결정 | cherry-pick 머지 |
| cherry-pick 대상 | 3 commits (ac298d4 + 104b4f7 + b3d848b) |
| author 보존 | ✅ @postmelee (cherry-pick default) |
| 충돌 | 0 (auto-merge `mydocs/orders/20260502.md` 정합) |
| 결정적 검증 | 모두 통과 |
| 시각 판정 1차 (SVG) | ✅ 통과 |
| 시각 판정 2차 (rhwp-studio web Canvas) | ✅ 통과 |
| WASM 빌드 | ✅ 성공 (4,461,549 bytes) |
| 부수 발견 (이슈 #543) | s0:pi=56 그림 캡션 겹침 — 별도 이슈 등록 |

## 2. cherry-pick 결과

### 2.1 적용된 commits (local/devel 기준)

| 신 commit | 원본 PR commit | 설명 |
|----------|--------------|------|
| `f4af57e` | `ac298d4` | Task #530 Stage 3: TAC Top caption 표 본문 y 오프셋 정정 |
| `a9367a9` | `104b4f7` | Task #530 Stage 4: 회귀 검증 |
| `ed652a3` | `b3d848b` | Task #530 Stage 5: 최종 보고 |

cherry-pick 의 default 동작으로 author = @postmelee 유지, committer = edward (메인테이너).

### 2.2 변경 파일 (PR #531 본질만)

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/table_layout.rs` | +18 / -2 (inline_top_caption_offset 추가 + table_y 보정) |
| `tests/issue_530.rs` (신규) | +99 (회귀 테스트 1건) |
| `mydocs/plans/task_m100_530{,_impl}.md` (신규) | 수행 / 구현 계획서 |
| `mydocs/working/task_m100_530_stage{1,3,4}.md` (신규) | 단계별 보고서 |
| `mydocs/report/task_m100_530_report.md` (신규) | 최종 보고서 |
| `mydocs/orders/20260502.md` | +2 / -1 (auto-merge 정합) |

## 3. 검증 결과

### 3.1 결정적 검증 (모두 통과)

| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | ✅ **1110 passed** |
| `cargo test --test issue_530` | ✅ 1 passed |
| `cargo test --test issue_418` | ✅ 1 passed (회귀 0) |
| `cargo test --test issue_501` | ✅ 1 passed (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ **6/6 passed** (table_text_page_0 포함) |
| `cargo clippy --lib -- -D warnings` | ✅ 0 건 |
| `cargo clippy --test issue_530 -- -D warnings` | ✅ 0 건 |
| `cargo build --release` | ✅ Finished |

### 3.2 WASM 빌드

| 산출물 | 크기 |
|--------|------|
| `pkg/rhwp_bg.wasm` | 4,461,549 bytes (PR #507 시점 4,461,235 +314 — table_layout.rs 정정 반영) |
| `pkg/rhwp.js` | 231,609 bytes (변동 없음) |
| `rhwp-studio/public/rhwp_bg.wasm` | ✅ 동기화 |
| `rhwp-studio/public/rhwp.js` | ✅ 동기화 |

### 3.3 시각 판정 결과 (작업지시자 직접 판정)

#### 1차 (SVG, CLI export-svg)

페이지 5 우측 단의 TAC 표 (`pi=60 ci=0`, 3x3, `wrap=TopAndBottom`):

| 항목 | 결과 |
|------|------|
| Top caption 두 줄 / 머리행 / 본문 분리 | ✅ 통과 (정상 분리) |
| caption baseline y | 551.32 (유지) |
| 머리행 top y | 525.49 → 560.83 (caption 아래로 이동) |

#### 2차 (rhwp-studio web Canvas + 한컴 2010/2020)

WASM 재빌드 + studio 동기화 후, 작업지시자가 rhwp-studio 에서 동일 fixture 시각 확인 + 한컴 2010/2020 비교.

작업지시자 인용:
> 웹 시각판정은 통과입니다.

→ web Canvas 와 SVG 의 시각 정합. table_layout.rs 영역의 renderer 별 시각 차이 없음.

## 4. 본 PR 의 본질 정리

### 4.1 결함

`samples/basic/treatise sample.hwp` 5페이지 우측 단 TAC 표의 Top caption 두 줄이 머리행 텍스트와 같은 y 영역에 렌더링되어 겹침.

### 4.2 근본 원인

`src/renderer/layout/table_layout.rs::layout_table` 의 inline (TAC) 경로:

```rust
let table_y = if inline_x_override.is_some() {
    y_start  // ← Top caption 의 caption_height + caption_spacing 누락
} else {
    self.compute_table_y_position(/* ... caption_height, caption_spacing 반영 ... */)
};
```

`inline_x_override.is_some()` 인 TAC 경로에서 외부에서 계산한 y_start 를 그대로 사용하나, **Top caption 의 높이가 외부에서 반영되지 않아 누락**.

### 4.3 정정

```rust
let inline_top_caption_offset = if inline_x_override.is_some() && depth == 0 {
    if let Some(ref caption) = table.caption {
        if matches!(caption.direction, CaptionDirection::Top) {
            caption_height + caption_spacing
        } else {
            0.0
        }
    } else {
        0.0
    }
} else {
    0.0
};

let table_y = if inline_x_override.is_some() {
    y_start + inline_top_caption_offset
} else {
    self.compute_table_y_position(/* 기존 */)
};
```

**핵심 결정 사항:**
- `depth == 0` 조건으로 중첩 표 (cell 내부 표) 회피
- `Top` direction 만 적용 — Bottom/Left/Right caption 경로는 변경 없음
- caption 자체의 y 는 `y_start` 그대로 유지

## 5. 부수 발견 — 이슈 #543

본 PR 의 시각 판정 1차 중 작업지시자가 발견한 부수 결함:

`samples/basic/treatise sample.hwp` page=5 좌측 단의 TAC 그림 (`s0:pi=56 ci=0`, `bin_id=3`, 75.4×33.3mm, tac=true) 이 캡션과 겹침. 본질이 표 영역의 정정과 동일 클래스 (TAC + Top caption + caption_height + caption_spacing 누락) 이지만 **Shape (그림) 코드 경로** 의 별도 결함.

[이슈 #543](https://github.com/edwardkim/rhwp/issues/543) 으로 별도 등록 (milestone v1.0.0, assignee edwardkim). PR #531 의 본 task 와 무관하므로 별도 task 로 진행.

작업지시자 인용:
> 이미지의 위치가 윗쪽으로 캡션 높이만큼 올라가야 정답입니다.

→ 표는 본문이 caption *아래*로 이동해야 하지만, 그림은 본문이 caption *위*로 이동해야 함 (한컴 시각 판정 기준). 부호/적용 위치 정밀 점검 필요.

## 6. 컨트리뷰터 정합

본 PR 은 외부 컨트리뷰터 @postmelee 의 task #530 PR. 다음 정합한 절차를 수행:

1. **본질 정확 진단** — `dump-pages` + `dump` + `--debug-overlay` 의 통합 사용으로 셀 내부 line segment 가 정상이지만 표 본문 y 가 caption 과 동일이라는 본질 정밀 식별. 디버그 도구 적극 활용.
2. **변경 영역 격리** — `depth == 0` + Top direction + caption.is_some + inline_x_override 의 4 분기로 영향 범위 최소화.
3. **회귀 테스트** — fixture-based 결정적 좌표 비교 (max_caption_baseline + 0.1 < table.bbox.y). RenderNode 트리 재귀 헬퍼 정합.
4. **시각 검증 자료** — 좌표 기반 before/after (525.49 → 560.83) + SVG 스크린샷 첨부.
5. **내부 워크플로우 정합** — 수행 / 구현 계획서 + Stage 1/3/4 단계별 보고서 + 최종 보고서.

작업지시자 인용:
> postmelee 컨트리뷰터님의 경우 디버깅 레이아웃 활용까지 적극 활용하는 것을 보니 너무 기쁘군요. ... 이 예제까지 컨트리뷰터가 보고 처리하는 것에 메인테이너가 깊은 인상을 받았습니다.

## 7. 머지 절차

### 7.1 cherry-pick + 검토 문서 commit

```bash
git checkout local/devel
git stash push -u -m "PR #531 review docs" mydocs/pr/pr_531_review.md mydocs/pr/pr_531_review_impl.md
git cherry-pick ac298d4   # → f4af57e
git cherry-pick 104b4f7   # → a9367a9
git cherry-pick b3d848b   # → ed652a3
git stash pop
# 검토 문서 + report 함께 commit (다음 단계)
```

### 7.2 devel 머지 + push

```bash
git checkout devel
git merge local/devel --no-ff -m "Merge local/devel: PR #531 cherry-pick (Task #530 TAC Top caption 표 본문 y 오프셋 정정 — cherry-pick @postmelee 3 commits) — refs #530"
git push origin devel
```

### 7.3 PR / 이슈 close + 컨트리뷰터 인사

```bash
gh pr close 531 --repo edwardkim/rhwp --comment "..."
# 이슈 #530 close 결정은 작업지시자 (PR 본문 Refs #530, not closes)
```

## 8. 사후 처리

- [ ] PR #531 close (수동, cherry-pick 머지로 GitHub 자동 close 미동작)
- [ ] 이슈 #530 처리 (작업지시자 결정 — close 또는 open 유지)
- [ ] 이슈 #530 milestone v1.0.0 추가 권장
- [ ] 이슈 #543 (s0:pi=56 Shape caption 겹침) — 별도 task 로 진행
- [ ] README 기여자 목록 갱신 (@postmelee — 본 사이클 일괄 갱신 시점에 반영)
- [ ] 컨트리뷰터 인사 댓글 (작업지시자 직접 작성 권장)

## 9. 메모리 정합

- ✅ `feedback_check_open_prs_first` — 본 PR 처리 정합 (이슈 #530 → PR #531 연결 확인 완료)
- ✅ `feedback_pr_comment_tone` — close 댓글 차분/사실 중심
- ✅ `feedback_release_sync_check` — cherry-pick 시점 main 동기화 점검 (`0fb3e675` 정합)
- ✅ `feedback_v076_regression_origin` — 작업지시자 직접 시각 판정 1차 + 2차 통과
- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 (1차 SVG + 2차 web Canvas) 둘 다 통과
- ✅ `feedback_pdf_not_authoritative` / `reference_authoritative_hancom` — 작업지시자 한컴 2010/2020 직접 판정으로 정답지 확정
- ✅ `feedback_image_renderer_paths_separate` — 이슈 #543 의 본질 (Shape 경로의 동일 결함 클래스) 발견. table_layout.rs 정정으로 모든 경로 해결 안 됨 — Shape 경로의 별도 정정 필요. 정합 사례.
- ✅ `feedback_hancom_compat_specific_over_general` — 본 PR 의 정정은 case-specific (TAC + Top caption + depth=0 한정), 정합
