# Task #716 최종 결과 보고서

**제목**: Task #643 이후: 20250130-hongbo.hwp page 1 마지막 줄 LAYOUT_OVERFLOW_DRAW (Task #332 잔존 영역)
**Issue**: [#716](https://github.com/edwardkim/rhwp/issues/716)
**브랜치**: `local/task716` (integration/3pr-stack 베이스)
**작업 기간**: 2026-05-08 (단일 세션)
**최종 상태**: ✅ closes #716

---

## 1. 결함 요약

`samples/20250130-hongbo.hwp` 페이지 1 의 마지막 텍스트 줄 (pi=15 line 2) 이 컬럼 하단(=Body 영역 1028.04 px) 을 +20.15 px 초과하여 **시각적으로 cropping** 되는 결함.

```
LAYOUT_OVERFLOW_DRAW: section=0 pi=15 line=2 y=1048.2 col_bottom=1028.0 overflow=20.1px
LAYOUT_OVERFLOW: page=0, col=0, para=15, type=PartialParagraph, y=1059.4, bottom=1028.0, overflow=31.3px
```

**결과**: 본 정정 후 모든 TextLine 이 컬럼 하단 이내. `LAYOUT_OVERFLOW_DRAW` (실제 시각 cropping) 0건.

## 2. Root cause 분석 흐름 (가설 갱신)

### 초기 가설 (수행계획서) — 폐기

> 음수 line_spacing(ls<0) 미반영 → drift 누적 (TAC 표 호스트 + 빈 문단 advance)

가설 H1 (TAC 호스트 ls 가산), H2-1 (lazy_base.max(0)), H2-2 (trailing_ls 음수 허용) 검토.

**Stage 2 instrument 실측 결과 폐기**: paragraph_layout 의 line advance 자체는 음수 ls 를 정확히 가산함 (`src/renderer/layout/paragraph_layout.rs:2657 y += line_height + line_spacing_px`). 음수 ls 처리에 결함 없음.

### 수정된 본질 (Stage 2 보고서)

**Task #9 의 `fix_overlay_active` push 가 빈 paragraph 까지 push 하여 의미 없는 drift 만 누적**:

`src/renderer/layout.rs:1559-1572` 의 push 분기:
```rust
if fix_overlay_active && !is_fixed {
    let table_bottom = fix_table_start_y + fix_table_visual_h;
    if y_offset < table_bottom {
        y_offset = table_bottom;   // ← 빈 paragraph 도 push
    }
    fix_overlay_active = false;
}
```

`fix_overlay_active` 는 직전 item 이 TAC 표 + 호스트 line_seg.ls<0 일 때 활성 (Task #9). 후속 paragraph 가 표 위에 그려지지 않도록 표 하단까지 push 하는 의도.

**문제**: 빈 paragraph (text_len=0) 는 시각적으로 invisible. push 적용해도 보이는 차이는 없으나 **y_offset 만 (table_bottom - y_offset) 만큼 누적**.

### 산수 검증

| pi | TAC 표 | host ls (HU) | table_bottom (px) | y_in (px) | push 누적 |
|----|--------|-------------|-------------------|-----------|-----------|
| 0 | TAC 1x3 | -600 | 149.93 | — | (TAC 자체) |
| 1 | empty para (Percent) | — | — | 141.93 | **+8.00 px** |
| 2 | TAC 1x4 | -900 | 211.45 | — | (TAC 자체) |
| 3 | empty para (Percent) | — | — | 199.45 | **+12.00 px** |

→ **pi=1 + pi=3 누적 push = +20.00 px** ≈ Stage 1 RED overflow +20.15 px. 99.3% 일치.

## 3. 정정 (`src/renderer/layout.rs:1559-1582`)

빈 paragraph 판정 가드 추가:

```rust
// Percent 전환: 표 하단과 비교 (Task #9)
if fix_overlay_active {
    let is_fixed = paragraphs.get(item_para)
        .and_then(|p| styles.para_styles.get(p.para_shape_id as usize))
        .map(|ps| ps.line_spacing_type == crate::model::style::LineSpacingType::Fixed)
        .unwrap_or(false);
    // [Task #716] 빈 paragraph (text_len=0 또는 control 문자/object placeholder
    // 만 존재) 는 시각적으로 invisible. fix_overlay push 가 적용되어도
    // 보이는 차이가 없는 반면 y_offset 만 (table_bottom - y_offset) 만큼
    // 누적되어 forward drift 의 누적 원인이 된다 (page 1 LAYOUT_OVERFLOW
    // 의 99.3%: pi=1 +8 px + pi=3 +12 px). Task #9 의 push 의도(텍스트
    // paragraph 가 TAC 표 위에 침범하지 않도록 보호) 는 그대로 유지하고,
    // 빈 paragraph 는 push 대상에서 제외한다. fix_overlay_active 는 유지하여
    // 후속 비-empty paragraph 가 push 대상이 될 수 있게 한다.
    let is_empty_para = paragraphs.get(item_para)
        .map(|p| p.text.is_empty()
            || p.text.chars().all(|c| c <= '\u{001F}' || c == '\u{FFFC}'))
        .unwrap_or(false);
    if !is_fixed && !is_empty_para {
        let table_bottom = fix_table_start_y + fix_table_visual_h;
        if y_offset < table_bottom {
            y_offset = table_bottom;
        }
        fix_overlay_active = false;
    }
}
```

**핵심 변경**: `if !is_fixed && !is_empty_para { ... }` — 빈 paragraph 면 push 분기 진입 차단.

빈 paragraph 판정: `p.text.is_empty()` 또는 `p.text.chars().all(|c| c <= '\u{001F}' || c == '\u{FFFC}')` (control 문자 + object placeholder 만 존재).

## 4. 검증 결과

### RED → GREEN

```
Stage 1 (RED):  max_bottom=1048.19 overflow=+20.15  FAIL
Stage 3 (GREEN): max_bottom=1028.19 overflow=+0.15  PASS (허용 0.5 px 이내)
```

`tests/issue_716.rs` — page 0 의 모든 본문 TextLine bbox 하단이 Body 하단 이내 단언.

### stderr 출력

| 메시지 | Stage 1 | Stage 6 | 의미 |
|--------|---------|---------|------|
| `LAYOUT_OVERFLOW_DRAW` | 발생 | **0건** ✓ | 실제 시각 cropping |
| `LAYOUT_OVERFLOW` | overflow=31.3px | overflow=11.3px | y_offset 누적 (trailing ls) |

`LAYOUT_OVERFLOW_DRAW` (시각 cropping 측정) 가 0건 — 본 결함 해소.

`LAYOUT_OVERFLOW` 11.3 px 잔존은 마지막 줄 trailing line_spacing 의 y_offset 누적으로, **시각적 영향 없음**. Task #452 의 의도된 trailing 가산 결과로 본 task 와 별개.

### 회귀 검증 (Stage 4 + Stage 5)

**Stage 4** — `cargo test --release`:
- 1500+ 테스트 0 failed
- 골든 SVG 7개 PASS (form-002, issue-147, issue-157, issue-267, issue-617, table-text)
- 회귀 0건

**Stage 5** — 169 샘플 광범위:

| 메트릭 | Before | After | Δ |
|--------|--------|-------|---|
| `LAYOUT_OVERFLOW_DRAW` 총 | 187 | 185 | −2 (정정) |
| `LAYOUT_OVERFLOW` 총 | 279 | 277 | −2 (부수 개선) |
| panic 총 | 0 | 0 | 0 |
| 페이지 수 변동 | — | — | **0 샘플** |

샘플별:
- `20250130-hongbo.hwp/no` (4쪽): DRAW 1→0 (본 결함 정정)
- `table-vpos-01.hwp/hwpx` (5쪽): FLOW 1→0 (부수 개선, 동일 메커니즘)
- 그 외 165 샘플: 변동 없음

→ **신규 OVERFLOW 발생 0건, 페이지 수 변동 0건, 의도된 정정만**.

## 5. 영향 분석

### 본 정정의 효과

1. **빈 paragraph 의 fix_overlay push 차단** — TAC 표(ls<0) 직후 첫 빈 paragraph 가 표 하단으로 점프하지 않음
2. **누적 drift 차단** — pi=1 +8 + pi=3 +12 = +20 px drift 제거
3. **시각 정합 유지** — 빈 paragraph 는 보이지 않으므로 push/no-push 차이가 시각적으로 없음

### Task #9 본래 의도 보존

- 텍스트 paragraph 의 push: **유지** (TAC 표 위에 텍스트 침범 차단)
- Fixed line spacing paragraph: **유지** (기존대로 push 면제)
- 빈 paragraph 만: **신규 면제** (의미 없는 push 차단)

### 잔존 영역 (본 task 비범위)

- `LAYOUT_OVERFLOW` 11.3 px (마지막 줄 trailing ls y_offset 누적): Task #452 의 의도된 동작, 별개 검토 필요
- 다른 샘플의 LAYOUT_OVERFLOW_DRAW 185건: 본 task 외 영역 (개별 메커니즘)

## 6. 단계별 산출물

| Stage | 커밋 | 산출물 |
|-------|------|--------|
| 0 | `a1832eac` | 수행 + 구현 계획서 |
| 1 (RED) | `a0e93b17` | `tests/issue_716.rs` + FAIL 확인 |
| 2 (분석) | `86be59f0` | RHWP_TASK716_DEBUG instrument trace + 가설 갱신 |
| 3 (GREEN) | `a8ddb717` | `is_empty_para` 가드 + RED PASS |
| 4 (회귀) | `33a9b864` | cargo test 0 failed + 골든 SVG 회귀 0 |
| 5 (광범위) | `363cb566` | 169 샘플 회귀 0 + 페이지 수 변동 0 |
| 6 (최종) | (본 커밋) | 최종 보고서 + closes #716 |

## 7. PR 정보

- 브랜치 (origin push 예정): `pr-task716` (stream/devel 베이스)
- 본 task 의 cherry-pick 대상 커밋: `a1832eac`, `a0e93b17`, `86be59f0`, `a8ddb717`, `33a9b864`, `363cb566`, (본 커밋)
- conflict 점검: `git merge-tree --write-tree origin/stream/devel...HEAD` 로 사전 검증 후 push

## 8. 학습 / 노트

### 가설의 변경 — Stage 2 instrument 의 가치

수행계획서 단계의 정적 분석은 "음수 ls 미반영" 으로 가설을 세웠으나, **Stage 2 instrument 실측에서 line advance 자체는 정확히 ls 를 가산함을 확인** → 가설을 "Task #9 fix_overlay push" 로 갱신. 정적 분석만으로는 잡기 어려운 위치를 instrument trace 가 정확히 식별.

→ **TDD + dynamic instrument 의 가치**: 정적 가설은 출발점일 뿐, 실측이 정확한 정정 위치를 결정.

### Task #9 의 의도와 한계

Task #9 는 TAC 표(ls<0) 후 텍스트 paragraph 가 표 위로 침범하는 시각 결함을 정정한 의의 있는 fix. 하지만 빈 paragraph 까지 push 적용은 의도하지 않은 drift 누적 부작용.

→ 본 task 는 Task #9 의도 보존 + 부작용 차단. 좁고 안전한 정정.

### `LAYOUT_OVERFLOW_DRAW` vs `LAYOUT_OVERFLOW`

두 메시지는 의미가 다름:
- `LAYOUT_OVERFLOW_DRAW`: 실제 그려지는 텍스트 줄(TextLine) 이 컬럼 하단 초과 — **시각 cropping**
- `LAYOUT_OVERFLOW`: layout y_offset 이 컬럼 하단 초과 — y_offset 누적기준 (trailing ls 등 포함)

이슈 본문은 두 메시지를 모두 인용했으나, 본 결함의 본질은 `LAYOUT_OVERFLOW_DRAW` (시각 cropping). `LAYOUT_OVERFLOW` 의 trailing ls 11.3 px 잔존은 Task #452 영역.

## 9. 관련 자료

- 수행 계획서: `mydocs/plans/task_m100_716.md`
- 구현 계획서: `mydocs/plans/task_m100_716_impl.md`
- Stage 보고서: `mydocs/working/task_m100_716_stage{1,2,3,4,5}.md`
- 회귀 테스트: `tests/issue_716.rs`
- 정정 위치: `src/renderer/layout.rs:1559-1582`
- 관련 task: Task #9 (fix_overlay 도입), Task #332 (VPOS_CORR), Task #452 (trailing ls), Task #643 (5축 정합)
