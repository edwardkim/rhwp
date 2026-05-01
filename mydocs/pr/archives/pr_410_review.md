# PR #410 검토 — Task #409 TopAndBottom Picture vert=Para chart 정정 + atomic TAC top-fit

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#410](https://github.com/edwardkim/rhwp/pull/410) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) — 신뢰 컨트리뷰터 |
| 이슈 | [#409](https://github.com/edwardkim/rhwp/issues/409) (closes) |
| base / head | `devel` ← `planet6897:local/task409_v3` |
| 변경 규모 | +4,657 / -38, 46 files (27 commits) |
| mergeable | `CONFLICTING` (DIRTY) |
| 검토 일자 | 2026-04-29 |

## PR 의 27 commits 누적 정황

| 영역 | 처리 |
|------|------|
| Task #398 (3) | PR #401 v2 로 흡수 완료 (devel) |
| Task #402 (3) | PR #406 으로 흡수 완료 (devel) |
| Task #404 (4) | PR #408 으로 흡수 완료 (devel) |
| 샘플 PDF (1) | devel `c2944a4` 와 동일 |
| 계획서/Stage 보고서/merge | skip |
| **Task #409 본질 (3)** | **분리 cherry-pick 대상** |

## 본질 — Task #409 3 단계 정정

### v1: `layout.rs::prev_has_overlay_shape` 가드 확장 (`368a869`)

**원인**: 기존 가드가 `Control::Shape` + `InFrontOfText|BehindText` 만 검사 → `Control::Picture` 미처리 + `TopAndBottom` 케이스 미포함.

**정정**: Picture (non-TAC) 분기 추가 + TopAndBottom + vert_rel_to=Para 케이스 포함:

```rust
let prev_has_overlay_shape = paragraphs.get(prev_pi).map(|p| {
    p.controls.iter().any(|c| match c {
        Control::Shape(s) => {
            let cm = s.common();
            matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)
                || (matches!(cm.text_wrap, TextWrap::TopAndBottom)
                    && matches!(cm.vert_rel_to, VertRelTo::Para)
                    && !cm.treat_as_char)
        }
        Control::Picture(pic) => {
            let cm = &pic.common;
            if cm.treat_as_char { return false; }
            matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)
                || (matches!(cm.text_wrap, TextWrap::TopAndBottom)
                    && matches!(cm.vert_rel_to, VertRelTo::Para))
        }
        _ => false,
    })
}).unwrap_or(false);
```

**효과**: 21페이지 LAYOUT_OVERFLOW 19→1, 2x1 표 차트 직하 (PDF 일치).

### v2: `typeset.rs::typeset_section` chart 높이 누적 (`d1f19e9`)

**원인**: controls 루프가 비-TAC Picture/Shape 의 높이를 `current_height` 에 미누적 → 페이지네이션 추정 used 가 layout 실제값보다 작음 → 불필요한 packing.

**정정**: 비-TAC + TopAndBottom + vert=Para 인 Picture/Shape 의 `height + margin.bottom` 을 `current_height` 에 누적 (layout 의 calc_shape_bottom_y 와 동일 산식).

**효과**: 22페이지에 (4) 헤딩 + 10x5 표가 정상 표시.

### v3: `typeset.rs::typeset_paragraph` atomic TAC top-fit (`fa8f923`)

**원인**: 23페이지 차트 (TAC Picture, lh=316px, 시작 y=721 / 끝 y=1037, 본문 1028px 보다 9.37px 초과) 가 strict bottom-fit 으로 next page 분리. HWP 시멘틱은 atomic 항목에 대해 시작점이 본문 안이면 현재 페이지 배치 + 하단 일부 흘림 허용 (15mm 하단 여백).

**정정**: 단일 라인 + TAC Picture/Shape 항목은 60px (≈1.6cm) 이내 초과면 현재 페이지 배치:

```rust
let is_atomic_tac_singleton = fmt.line_heights.len() == 1
    && para.controls.iter().any(|c| match c {
        Control::Picture(p) => p.common.treat_as_char,
        Control::Shape(s) => s.common().treat_as_char,
        _ => false,
    });
if is_atomic_tac_singleton
    && st.current_height < available
    && !st.current_items.is_empty()
{
    let overflow = st.current_height + fmt.height_for_fit - available;
    if overflow <= 60.0 {
        st.current_items.push(PageItem::FullParagraph { para_index: para_idx });
        st.current_height += if st.col_count > 1 { fmt.height_for_fit } else { fmt.total_height };
        return;
    }
}
```

**효과**: 23페이지 막대 차트 정상 배치, 24페이지 시작 = 2x1 표 → (6) 헤딩 → 파이차트 (PDF 일치).

## 처리 방향

**옵션 A — 본질 3 commits 분리 cherry-pick** (PR #406/#408 와 같은 패턴).

이유:
1. Task #398/#402/#404 commits 는 이미 devel 흡수
2. 샘플 PDF 는 devel 동일 (no-op)
3. 본질 변경은 v1/v2/v3 의 3 commits

## dry-run cherry-pick 결과

`local/pr410` 브랜치 (`local/devel` 분기) 에서 3 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| (← `368a869`) | @planet6897 | v1: layout.rs prev_has_overlay_shape 가드 확장 |
| `bd8ea80` (← `d1f19e9`) | @planet6897 | v2: typeset.rs::typeset_section chart 높이 누적 |
| `1f5dcc5` (← `fa8f923`) | @planet6897 | v3: typeset.rs::typeset_paragraph atomic TAC top-fit |

cherry-pick 결과: 충돌 없이 자동 적용.

## 검증 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1050 passed** (회귀 없음) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 40s, 4,116,929 bytes |

## 시각 판정

**한컴 정답지 (한컴 2010 / 2022 + origin PDF) vs rhwp 출력 비교** — 작업지시자 직접 판정.

| 페이지 | devel baseline | PR #410 적용 | PDF 일치 |
|--------|---------------|--------------|---------|
| 21쪽 | 2x1 표 페이지 하단 잘림 | 차트 직하 정상 | ✅ |
| 22쪽 | 차트로 시작 (헤딩+표 누락) | (4) 헤딩 + 10x5 표 정상 | ✅ |
| 23쪽 | 차트 24쪽으로 밀림 | 막대 차트 하단 정상 | ✅ |
| 24쪽 | 차트로 시작 | 2x1 표 → (6) 헤딩 → 파이차트 | ✅ |

산출물:
- SVG (devel): `output/svg/pr410-devel-baseline/2025년 기부·답례품 실적 지자체 보고서_양식_{021..024}.svg`
- SVG (PR #410): `output/svg/pr410-visual/2025년 기부·답례품 실적 지자체 보고서_양식_{021..024}.svg`
- WASM: `pkg/rhwp_bg.wasm` (4,116,929 bytes)

**작업지시자 시각 판정 결과:**
- SVG 내보내기: ✅ **통과**
- Canvas (rhwp-studio): ✅ **통과**

## #426 회귀 검증

`aift.hwp` 19페이지 (이슈 #426 — 이미지+캡션+다음 문단 오버래핑) 회귀 영향 확인:
- devel baseline `aift_019.svg`: 536,259 bytes
- PR #410 적용 `aift_019.svg`: **536,259 bytes (byte 단위 동일)**

→ 회귀 영향 **없음 ✅** (#426 별개 결함, PR #410 변경 영역과 분리).

## 회귀 검증 중 발견 (별도 이슈 등록)

`aift.hwp` 41페이지 표 셀 안 이미지가 **미조판** (SVG 출력 누락) — **사전 존재 결함**, 본 PR 의 TopAndBottom + vert=Para 변경 영역과 다른 경로 (표 셀 내 Picture). 별도 이슈 #429 로 등록.

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1050 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 (push 전 필수) | ✅ 작업지시자 SVG + Canvas 양 경로 통과 |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | ✅ TopAndBottom + vert=Para + 비-TAC 명시 + atomic TAC 60px tolerance |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr410` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr410` → `local/devel` → `devel` 머지 + push
3. PR #410 close + 작성자 댓글 (이슈 #409 자동 close)

## 참고

- PR: [#410](https://github.com/edwardkim/rhwp/pull/410)
- 이슈: [#409](https://github.com/edwardkim/rhwp/issues/409)
- 사전 등록 이슈: [#426](https://github.com/edwardkim/rhwp/issues/426) (회귀 영향 없음 확인)
- 회귀 검증 중 발견: [#429](https://github.com/edwardkim/rhwp/issues/429) (사전 결함, 별도 등록)
- 같은 작성자 머지 PR: [#401 v2](https://github.com/edwardkim/rhwp/pull/401), [#406](https://github.com/edwardkim/rhwp/pull/406), [#408](https://github.com/edwardkim/rhwp/pull/408), [#415](https://github.com/edwardkim/rhwp/pull/415)
