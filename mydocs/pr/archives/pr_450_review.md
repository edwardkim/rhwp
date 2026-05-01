# PR #450 검토 — Task #445 지문 박스/페이지 번호 박스 시각 결함 정정 (exam_kor)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#450](https://github.com/edwardkim/rhwp/pull/450) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) — 본 사이클 13번째 PR |
| 이슈 | [#445](https://github.com/edwardkim/rhwp/issues/445) (closes) |
| base / head | `devel` ← `planet6897:local/task445` |
| 변경 규모 | +1,826 / -7, 15 files (17 commits → 본질 3 commits) |
| mergeable | `MERGEABLE` (BEHIND) |
| 검토 일자 | 2026-04-30 |

## 17 commits 누적 정황

| 영역 | commits | 처리 |
|------|---------|------|
| Task #435 (4) | `e0cdc2e`~`0a096fc` | PR #442 흡수 완료 |
| Task #435 merge (3) | `2da9c8d`, `a6416bf`, `50b02f3` | skip |
| Task #439 (4) | `3884305`~`df2aff3` | PR #443 흡수 완료 |
| Task #439 merge (1) | `50b5cab` | skip |
| **Task #445 (3)** | `c151156`, `10d8709`, `717ca1f` | **본 PR 의 본질 변경 — 분리 cherry-pick** |
| 추가 merge (2) | `9bf5a74`, `33a3802` | skip |

## 본질 — 두 시각 결함 정정

### 결함 1: paragraph border 페이지 바깥 침범

`samples/exam_kor.hwp` 의 지문 박스 (border_fill_id=7) 가 페이지 분할 시 PartialParagraph 의 border 세로 테두리가 col_bottom 너머 + 일부 페이지 바깥까지 그려짐:

| 페이지 | overflow vs col_bottom | overflow vs 페이지 (1587) | 정정 효과 |
|-------|---------------------|----------------------|----------|
| 2 | +30 px | OK | border 클램프 |
| 5 | +84 px | OK | border 클램프 |
| **8** | **+248 px** | **+84 px 페이지 바깥** | **가장 심각 → 정상화** |
| 15 | +173 px | +9 px 페이지 바깥 (col 1) | border 클램프 |

**정정**: `layout.rs::build_single_column` 에서 paragraph border merge 후 col_area 바닥/꼭대기 클램프:

```rust
let col_top = col_area.y;
let col_bot = col_area.y + col_area.height;
for g in groups.iter_mut() {
    if g.2 < col_top { g.2 = col_top; }
    if g.4 > col_bot { g.4 = col_bot; }
}
groups.retain(|g| g.4 > g.2);
```

### 결함 2: 페이지 번호 박스 위치 ("1/20" 등)

PDF 측정 (`samples/hancomdocs-exam_kor.pdf` A3 200dpi):
- 박스 top: 380.6 mm = body_bottom + 4.3 mm
- column line - 박스 갭: **4.3 mm (16.0 px)**

기존 SVG: 갭 0 mm (line 끝에 붙음).

**정정**: `layout.rs::layout_header_footer_paragraphs` 에서 wrap=TopAndBottom + vert=Para 표 anchor 에 `line_height/2` 추가 (HWP 의 line center anchor 시멘틱):

```rust
let line_anchor_offset = if matches!(t.common.text_wrap, TextWrap::TopAndBottom)
    && matches!(t.common.vert_rel_to, VertRelTo::Para)
    && i == 0
{
    let lh_hu = para.line_segs.first().map(|ls| ls.line_height as i32).unwrap_or(0);
    hwpunit_to_px(lh_hu, self.dpi) / 2.0
} else { 0.0 };
let table_y = y_offset + line_anchor_offset;
```

꼬리말 paragraph line_height = 2480 HU = 16.5 px ≈ PDF 측정 갭 16.0 px (±1px 정합).

## 본 PR 의 정정 본질 (작성자 명시)

> "respect_vpos_reset 정책: 페이지네이션의 vpos-reset 미존중이 paragraph 의 col_bottom 너머 layout 의 진짜 원인. **본 PR 의 클램프는 시각적 증상만 가림** — 텍스트 자체의 overflow 는 별도 이슈로 분리 검토 필요."

→ **본 PR 은 시각 증상 정정** (본질 알고리즘 정정 아님). 작성자 본인이 인정.

작성자가 시도했다가 PDF 비교에서 폐기한 본질 정정:
- master-page polygon 강제 클램프
- 폴리곤 body_top..body_bottom 정규화
- 콘텐츠-기반 column separator 단축

→ 본질 정정의 회귀 위험이 큰 정황 → **시각 증상 정정 + 잔여 본질 작업 분리** 패턴으로 우회.

## 작업지시자 분석 — 작성자 진단 모드 변천

작업지시자 통찰:
> "이 컨트리뷰터는 자신의 가설을 미세하게 변하는 상태를 관찰하는 모드로 진입중이라고 봅니다."

본 사이클 13 PR 패턴:
- 처음 9 PR (Task #398/#402/#404/#409/#352/#412/#430/#435/#439): 본질 정정 (알고리즘 변경)
- **본 PR (Task #445)**: 시각 증상 클램프 + PDF 정량 측정 정합 (200dpi, ±1px)

→ **본질 정정의 회귀 위험을 인식하고 시각 증상만 정정하는 보수적 우회로 전환**. 메모리 `feedback_pdf_not_authoritative` 와의 균형 고려 필요.

## 처리 방향

**옵션 A — Task #445 본질 3 commits 분리 cherry-pick** (PR #442/#443 와 같은 패턴).

작업지시자 결정 정황: 작성자 후속 PR 이 같은 영역 (`respect_vpos_reset`) 진행 예정이지만, 본 사이클 일관성 + 작성자가 이미 누적 commit 분리 패턴 인지 + 본 PR 의 시각 증상 정정이 본질 정정과 충돌 안 함 → cherry-pick 채택.

## dry-run cherry-pick 결과

`local/pr450` 브랜치 (`local/devel` 분기) 에서 3 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `452a41b` (← `c151156`) | @planet6897 | Stage 1+2: paragraph border 가 col_bottom 너머로 그려지는 문제 수정 |
| `c7979ef` (← `10d8709`) | @planet6897 | Stage 3: 머리말/꼬리말 wrap=TopAndBottom 표 anchor 위치 보정 |
| `8ff0212` (← `717ca1f`) | @planet6897 | Stage 4: 최종 결과보고서 + 오늘할일 갱신 |

cherry-pick 결과:
- Stage 1+2, 3 자동 적용
- Stage 4 의 `mydocs/orders/20260429.md` add/add 충돌 → HEAD 유지 후 `--continue`

## 검증 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1069 passed** (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed (issue-267/ktx-toc-page.svg snapshot 갱신 1건 — invisible rect height 5.34px) |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 21s, 4,186,281 bytes |

## 광범위 byte 단위 비교

10 샘플 / 305 페이지 SVG 비교 (devel ↔ PR #450):

| 결과 | 카운트 |
|------|------|
| byte 동일 | **238 / 305** |
| 차이 발생 (의도된 정정) | **67 / 305** |

차이 분포 (작성자 명시 외 광범위 영향):
- **2025년 기부·답례품 30 페이지** — wrap=TopAndBottom 표 anchor 정정 영향 (작성자 미명시 — 머리말/꼬리말 영역 영향)
- **exam_* (eng/kor/math) 28 페이지** — 정정 대상 + 머리말/꼬리말
- **synam-001 5, kps-ai 2, k-water-rfp 1, aift 1** — 머리말/꼬리말 영역

본 PR 의 두 변경이 **머리말/꼬리말 영역을 가진 모든 샘플** 에 영향 정황. 메모리 `feedback_v076_regression_origin` 원칙에 따라 광범위 변화 직접 시각 검증 필요.

## 시각 판정 (작업지시자 직접) — 4 페이지 점검

**한컴 정답지 (samples/hancomdocs-exam_kor.pdf) + debug-overlay SVG 직접 비교**:

| 페이지 | 정정 정황 | 작업지시자 시각 판정 |
|-------|---------|--------------------|
| 2 | border +30px 클램프 (페이지 내) | ✅ 분석 일치 |
| 5 | border +84px 클램프 (페이지 내) | ✅ 분석 일치 |
| **8** | **border +248px (+84px 페이지 바깥) → 정상화** | ✅ **분석 일치 — 가장 명확한 정정** |
| 15 | border +173px (+9px 페이지 바깥, col 1) → 정상화 | ✅ 분석 일치 |

산출물:
- PR #450 적용 (debug-overlay): `output/svg/pr450-debug/exam_kor_{002,005,008,015}.svg`
- devel baseline (debug-overlay): `output/svg/pr450-debug-baseline/exam_kor_{002,005,008,015}.svg`
- 광범위 회귀 비교: `output/svg/pr450-regression-{baseline,test}/`

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1069 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 (push 전 필수) | ✅ 작업지시자 4 페이지 직접 판정 통과 |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | ✅ wrap=TopAndBottom + vert=Para + i==0 명시 분기 |
| 작은 단위 PATCH 회전 | ✅ 시각 증상 정정 + 잔여 본질 (`respect_vpos_reset`) 별도 분리 |
| `feedback_pdf_not_authoritative` | ⚠️ 작성자 한컴독스 PDF 200dpi 의존 — 본 PR 의 정정이 한컴독스 환경에 정합 (한컴 2010/2020 추가 검증 미실시) |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr450` 에서 커밋 |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr450` → `local/devel` → `devel` 머지 + push
3. PR #450 close + 작성자 댓글 (이슈 #445 자동 close)

작성자 후속 PR 안내:
- 본 PR 의 시각 클램프가 적용된 상태에서 후속 본질 정정 (`respect_vpos_reset`) 진행 시
- 자기 fork base 를 origin/devel (cherry-pick 후) 로 정기 rebase 권장 — 누적 commit 분리 부담 감소

## 참고

- PR: [#450](https://github.com/edwardkim/rhwp/pull/450)
- 이슈: [#445](https://github.com/edwardkim/rhwp/issues/445)
- 같은 작성자 본 사이클 머지 PR: [#401 v2](https://github.com/edwardkim/rhwp/pull/401), [#406](https://github.com/edwardkim/rhwp/pull/406), [#408](https://github.com/edwardkim/rhwp/pull/408), [#410](https://github.com/edwardkim/rhwp/pull/410), [#415](https://github.com/edwardkim/rhwp/pull/415), [#424](https://github.com/edwardkim/rhwp/pull/424), [#434](https://github.com/edwardkim/rhwp/pull/434), [#442](https://github.com/edwardkim/rhwp/pull/442), [#443](https://github.com/edwardkim/rhwp/pull/443)
- 잔여 작업: `respect_vpos_reset` 정책 본질 정정 (작성자 후속 PR 예정)
