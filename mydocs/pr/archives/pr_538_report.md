# PR #538 처리 보고서

**PR**: [#538 fix: 21_언어_기출_편집가능본.hwp 줄간격 716 HU drift 정정 (Task #534 v2 + #537 + #539)](https://github.com/edwardkim/rhwp/pull/538)
**작성자**: @planet6897 (Jaeuk Ryu) — 외부 컨트리뷰터
**처리 결정**: ✅ **본질만 머지** (cherry-pick @planet6897 12 commits, fork devel 누적분 제외)
**처리일**: 2026-05-03

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 결정 | 본질 cherry-pick 머지 (작업지시자 의도 정합) |
| cherry-pick 대상 | 12 commits (PR 본문 명시 3 task 만) |
| 제외 | 170 commits (fork devel 누적분 40 여 task — 컨트리뷰터에게 다음 PR 전 동기화 안내) |
| author 보존 | ✅ @planet6897 (cherry-pick default) |
| 충돌 | 2 건 — `mydocs/orders/20260502.md` + `20260503.md` (수동 해소: 양쪽 일지 통합) |
| 결정적 검증 | 모두 통과 |
| 시각 판정 1차 (SVG) | ✅ 통과 |
| 시각 판정 2차 (rhwp-studio web Canvas) | ✅ 통과 |
| WASM 빌드 | ✅ 성공 (4,461,870 bytes) |
| 부수 발견 (별도 이슈) | #545 (aift.hwp p41 표 위치) + #546 (PR #506 회귀 origin) |

## 2. cherry-pick 결과

### 2.1 적용된 commits (local/devel 기준)

| 신 commit | 원본 PR commit | 설명 |
|----------|--------------|------|
| `e7f1adb` | `fbcb5c5` | Task #534: 수행계획서 |
| `4bb238b` | `b669ab5` | Task #534 Stage 1: Root cause 1차 조사 |
| `48f0a50` | `4abee04` | Task #534 Stage 3: layout_shape_item TAC Picture x 좌표 inner_pad 정합 |
| `1bca866` | `5357223` | Task #534 Stage 3-5: 회귀 검증 + 최종 보고서 |
| `576aa29` | `9dfc56a` | Task #534 v2: layout_shape_item TAC Picture LINE_SEG.column_start 정합 |
| `57ccde6` | `47d1aac` | Task #534 v2 Stage 보고서 |
| `d70599d` | `226b644` | Task #537 Stage 1: lazy_base drift baseline + TDD 단위테스트 |
| `585c495` | `1803bc6` | Task #537 Stage 2: lazy_base trailing-ls 보정 (A'안 적용) |
| `58af6c9` | `a39085a` | Task #537 Stage 3: 광범위 회귀 검증 + 최종 보고서 |
| `f60f580` | `e8a0a8c` | Task #539 Stage 1: 가설 D 확정 + TDD 단위테스트 |
| `606dc8f` | `0db709b` | Task #539 Stage 2: prev_has_overlay_shape 가드 완화 (treat_as_char 제외) |
| `fc32bd3` | `eb0ddc2` | Task #539 Stage 3: 광범위 회귀 검증 + 최종 보고서 |

cherry-pick 의 default 동작으로 author = @planet6897 유지, committer = edward (메인테이너).

### 2.2 변경 파일 (PR #538 본질만)

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | +60 / -4 (Task #534 v1+v2 + #537 + #539 누적) |
| `src/renderer/layout/integration_tests.rs` | +228 (TDD 통합 테스트 3건) |
| `mydocs/plans/task_m100_534{,_impl}.md` (신규) | 수행 / 구현 계획서 |
| `mydocs/plans/task_m100_537{,_impl}.md` (신규) | |
| `mydocs/plans/task_m100_539{,_impl}.md` (신규) | |
| `mydocs/working/task_m100_534_stage{1,3}.md` + `task_m100_534_v2_stage1.md` (신규) | 단계별 보고서 |
| `mydocs/working/task_m100_537_stage{1,2,3}.md` (신규) | |
| `mydocs/working/task_m100_539_stage{1,2,3}.md` (신규) | |
| `mydocs/report/task_m100_534_report.md` + `task_m100_537_report.md` + `task_m100_539_report.md` (신규) | 최종 보고서 |
| `mydocs/orders/20260502.md` + `20260503.md` | 일지 통합 (양쪽 보존) |

## 3. 검증 결과

### 3.1 결정적 검증 (모두 통과)

| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | ✅ **1113 passed** (PR #531 머지 후 1110 + #537/#539 TDD 3건 = 정합) |
| `cargo test --test issue_530` | ✅ 1 passed (PR #531 회귀 0) |
| `cargo test --test issue_505` | ✅ 9/9 (PR #507 회귀 0) |
| `cargo test --test issue_418` | ✅ 1 passed (회귀 0) |
| `cargo test --test issue_501` | ✅ 1 passed (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ **6/6 passed** (table_text_page_0 포함) |
| `cargo clippy --lib -- -D warnings` | ✅ 0 건 |
| `cargo build --release` | ✅ Finished |

### 3.2 WASM 빌드

| 산출물 | 크기 |
|--------|------|
| `pkg/rhwp_bg.wasm` | 4,461,870 bytes (PR #531 시점 4,461,549 +321 — Task #534/#537/#539 정정 반영) |
| `pkg/rhwp.js` | 231,609 bytes (변동 없음) |
| `rhwp-studio/public/rhwp_bg.wasm` | ✅ 동기화 |
| `rhwp-studio/public/rhwp.js` | ✅ 동기화 |

### 3.3 시각 판정 결과 (작업지시자 직접 판정)

#### 1차 (SVG, CLI export-svg)

`samples/21_언어_기출_편집가능본.hwp` 15 페이지 SVG (`output/svg/pr538_after_cherrypick/`) 와 한컴 2010/2020 PDF 비교:

- **Task #537** (TAC `<보기>` 표 직후 첫 답안 줄간격): P2 q3 / P3 q6 / P5 q9 / P6 q12 / P8 q15 / P9 q17/18 / P12 q23/24 / P13 q27 / P14 q29 (11곳) — 모두 IR vpos delta 와 정확 일치
- **Task #539** (글박스 호스트 직후 paragraph 줄간격): P7 (pi=145→146 르포르) + P9 (pi=181→182 더불어 수피즘) — gap 14.67 → 24.21 px (IR 정합)
- **Task #534** (다른 fixture exam_kor 의 layout_shape_item): 본 PR fixture 외 영역, 회귀 0 검증으로 충분

#### 2차 (rhwp-studio web Canvas + 한컴 2010/2020)

WASM 재빌드 + studio 동기화 후, 작업지시자가 rhwp-studio 에서 동일 fixture 시각 확인 + 한컴 2010/2020 비교.

작업지시자 인용:
> 이번 PR의 웹 에디터쪽 시각 검증 통과 판정합니다.

→ web Canvas 와 SVG 의 시각 정합. layout.rs 영역의 renderer 별 시각 차이 없음.

## 4. 본 PR 의 본질 정리

### 4.1 Task #534 v1+v2 — exam_kor 18p TAC Picture inner_pad + LINE_SEG.column_start

**결함**: exam_kor 18p 우측 단 — Square wrap 인라인 표 + tac=true 그림 동일 paragraph 의 그림 x 좌표 inner_pad 누락 (~11.33 px 좌측 시프트)

**정정**:
- **v1** (`48f0a50`): `layout_shape_item` TAC 분기에 has_visible_stroke + border_spacing 검사 + inner_pad_left 가산 (+25 / -2 LOC)
- **v2** (`576aa29`): LINE_SEG.column_start 정합 보강 (+12 / -1 LOC)

**검증** (PR 본문): 광범위 8 샘플 192 페이지 중 190 byte-identical, p18 image x 593.39 → 604.72 (+11.33 px) 정합.

### 4.2 Task #537 — TAC 표 직후 첫 답안 줄간격 716 HU drift

**결함**: 21_언어 기출 hwp `<보기>` 표 직후 첫 답안 ① 과 ② 사이 줄간격 716 HU(=9.55 px) 좁음 (작업지시자 보고 11곳)

**근본 원인**: 세 메커니즘 상호작용으로 lazy_base 에 716 HU drift 동결:
1. `prev_tac_seg_applied` 가드로 TAC 직후 vpos 보정 건너뜀
2. `paragraph_layout.rs` 마지막 줄 trailing-ls 제외 (Task #479)
3. lazy_base 가 sequential drift 를 base 로 박음

**정정** (`585c495`): `layout.rs:1494-1521` lazy_base 산출 시 prev_pi 의 last seg `line_spacing` 만큼 `y_delta_hu` 보정 (+14 / -2 LOC):

```rust
let trailing_ls_hu = paragraphs.get(prev_pi)
    .and_then(|p| p.line_segs.last())
    .map(|s| s.line_spacing.max(0))
    .unwrap_or(0);
let y_delta_hu = ((y_offset - col_area.y) / self.dpi * 7200.0).round() as i32
    + trailing_ls_hu;
```

**검증**: 11 곳 모두 IR vpos delta 와 정확 일치 (P2 q3: 63.09 → 72.64 등). 광범위 회귀 정합 (synam-001 / 복학원서 / exam_math/kor/eng/science / 2010-01-06).

### 4.3 Task #539 — 글박스 호스트 직후 paragraph 줄간격 (#537 후속)

**결함**: 글박스 (InFrontOfText tac=true Shape) 호스트 paragraph 직후 다음 paragraph 줄간격 1 ls(=716 HU=9.55 px) 좁음 (그룹 B 7p/9p 2곳)

**근본 원인**: `layout.rs:1443-1462` 의 `prev_has_overlay_shape` 가드가 `treat_as_char` 무관하게 `InFrontOfText/BehindText` 면 true → 직후 paragraph 의 vpos correction 분기 자체가 skipped → trailing-ls drift 716 HU 잔존.

**정정** (`606dc8f`): `Control::Shape` 분기에서 `treat_as_char=true` early return false 추가 (+9 / -0 LOC):

```rust
Control::Shape(s) => {
    let cm = s.common();
    if cm.treat_as_char {
        return false;  // tac=true 는 LINE_SEG vpos 에 통합 → overlay 영향 없음
    }
    matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)
        || ...
}
```

**검증**: 7p/9p 두 케이스 모두 gap 14.67 → 24.21 px (IR 정합). 광범위 8 샘플 회귀 0.

## 5. 부수 발견 — 별도 이슈

본 PR 의 시각 판정 중 작업지시자가 발견한 부수 결함:

### 5.1 이슈 #545 — aift.hwp p41 표 위치

`samples/aift.hwp` 페이지 41 (s2:pi=496 ci=0, 2x1 TAC 표) 의 위치 잘못 계산. **이전부터 발생** (PR #538 와 무관). [이슈 #545](https://github.com/edwardkim/rhwp/issues/545) 등록.

### 5.2 이슈 #546 — exam_science.hwp p2 페이지네이션 회귀 (PR #506 origin)

`samples/exam_science.hwp` 페이지 2 의 본문 누락 (37 → 2 items, 4페이지 → 6페이지). **PR #538 와 무관**, PR #506 (HWP 3.0 파서 + Square wrap, @jangster77 51 commits) 의 머지 시점에 정확히 발생. bisect 로 확정.

| 시점 | commit | 총 페이지 | p2 단 0 items |
|------|--------|----------|--------------|
| PR #506 직전 | `65e275d` | 4 | 37 ✅ |
| **PR #506 머지 후** | `c7330cf` | **6** | **2** ❌ |

[이슈 #546](https://github.com/edwardkim/rhwp/issues/546) 등록 — 회귀 origin 추가 bisect 후 정정 task 시작.

## 6. 컨트리뷰터 정합

### 6.1 본 PR 의 정합 사항

@planet6897 (Jaeuk Ryu) 의 PR 본문이 명시한 3 task (Task #534 v2 + #537 + #539):
- **본질 정확 진단** — IR vpos delta 측정 + 디버그 진단 도구 활용 (Stage 1)
- **TDD 통합 테스트 3건** — 결정적 회귀 검증
- **광범위 회귀 검증** — 8 핵심 샘플 (synam-001 / 복학원서 / exam_math/kor/eng/science / 2010-01-06)
- **단계별 분리** — Stage 1 진단 → Stage 2 정정 → Stage 3 회귀 검증 + 최종 보고서

### 6.2 비정합 사항 — fork devel 동기화 미실행

PR 의 base/head 가 모두 `devel` 이라 **fork 의 devel 전체 누적분 (170 commits + 40 task)** 이 PR 에 포함됨. 본 PR 본문에 명시되지 않은 task 들이 같이 들어가는 위험. mergeStateStatus = DIRTY (CONFLICTING).

→ 작업지시자 결정으로 본질 12 commits 만 cherry-pick. 다음 PR 전 컨트리뷰터에게 devel 동기화 부탁 안내 예정.

## 7. 머지 절차

### 7.1 cherry-pick + 충돌 해소

```bash
git checkout local/devel
git stash push -u -m "PR #538 review docs" mydocs/pr/pr_538_review.md mydocs/pr/pr_538_review_impl.md
git cherry-pick fbcb5c5 b669ab5 4abee04 5357223     # Task #534 v1
# 충돌: mydocs/orders/20260502.md → 양쪽 보존 + continue
git cherry-pick 9dfc56a 47d1aac                      # Task #534 v2
git cherry-pick 226b644 1803bc6 a39085a              # Task #537
# 충돌: mydocs/orders/20260503.md → 양쪽 통합 + continue
git cherry-pick e8a0a8c 0db709b eb0ddc2              # Task #539
# 충돌: mydocs/orders/20260503.md → 양쪽 통합 + continue
git stash pop
```

### 7.2 검증 + WASM 빌드

(위 §3 결과)

### 7.3 commit + 머지 + push

```bash
# 검토 문서 + report commit
git add mydocs/pr/pr_538_review.md mydocs/pr/pr_538_review_impl.md mydocs/pr/pr_538_report.md
git commit -m "PR #538 처리 보고서 + 검토 문서 (cherry-pick @planet6897 12 commits)"

# devel 머지 + push
git checkout devel
git merge local/devel --no-ff -m "Merge local/devel: PR #538 cherry-pick (Task #534 v1+v2 + #537 + #539 — cherry-pick @planet6897 12 commits) — closes #534/#537/#539"
git push origin devel
```

### 7.4 PR / 이슈 close + 컨트리뷰터 인사

```bash
gh pr close 538 --repo edwardkim/rhwp --comment "..."
# 이슈 #534/#537/#539 는 이미 closed (정정 적용으로 close 유지 정합)
# milestone v1.0.0 추가 (사후 처리)
```

**컨트리뷰터 안내**: 다음 PR 전 fork devel 동기화 부탁.

## 8. 사후 처리

- [ ] PR #538 close (수동, cherry-pick 머지로 GitHub 자동 close 미동작)
- [ ] 이슈 #537/#539/#534 milestone v1.0.0 추가 (사후 처리)
- [ ] 이슈 #545 (aift.hwp p41 표 위치) — 별도 task 로 진행
- [ ] 이슈 #546 (PR #506 회귀 origin) — 별도 task 로 진행 (bisect 추가 후)
- [ ] README 기여자 목록 갱신 (@planet6897 — 본 사이클 일괄 갱신 시점에 반영)
- [ ] 컨트리뷰터 인사 댓글 + fork devel 동기화 안내 (작업지시자 직접 작성 권장)

## 9. 메모리 정합

- ✅ `feedback_check_open_prs_first` — 본 PR 처리 정합
- ✅ `feedback_pr_comment_tone` — close 댓글 차분/사실 중심 + 컨트리뷰터 안내
- ✅ `feedback_release_sync_check` — cherry-pick 시점 main 동기화 점검 (`0fb3e675` 정합)
- ✅ `feedback_v076_regression_origin` — 작업지시자 직접 시각 판정 1차 + 2차 통과
- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 (1차 SVG + 2차 web Canvas) 둘 다 통과
- ✅ `feedback_pdf_not_authoritative` / `reference_authoritative_hancom` — 작업지시자 한컴 2010/2020 직접 판정으로 정답지 확정
- ✅ `feedback_image_renderer_paths_separate` — Task #534 v1+v2 의 layout_shape_item 정정도 동일 영역
- ✅ `feedback_assign_issue_before_work` — 이슈 #537/#539/#534 모두 assignee 부재 사례
- ✅ `feedback_hancom_compat_specific_over_general` — 본 PR 의 정정은 case-specific (TAC 표/글박스/Picture 한정)
