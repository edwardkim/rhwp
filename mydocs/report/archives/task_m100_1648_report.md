# 최종 보고서 — #1648 페이지 채운 TAC 표 직후 빈 문단 누락 수정

- 마일스톤: M100 / 브랜치: local/task1643 (upstream/devel)
- 일자: 2026-06-29 / 선행: #1643 vs-한글 페이지↔PI 비교의 `rhwp_pNone` 케이스

## 1. 문제

페이지를 거의 채운 treat_as_char(TAC) 표 직후의 **빈 문단 1개가 페이지네이션에서 누락**되어
어느 페이지에도 배치되지 않음(시각적으로 빈 줄 1개 손실). 한글은 해당 빈 문단을 페이지 하단에 배치
→ rhwp 페이지↔PI 가 한글과 어긋남(#1643 표본에서 관측).

## 2. 근본원인

기본 페이지네이션 엔진은 **TypesetEngine**(`typeset_section_with_variant`)이다
(Paginator=engine.rs 는 `RHWP_USE_PAGINATOR=1` fallback). Task #362 "단독 빈페이지 차단" 가드
(`src/renderer/typeset.rs`, `next_will_vpos_reset` 분기)가 빈 문단을 **fit 검사 없이 무조건 skip**:

```rust
if next_will_vpos_reset {                 // 현재 para 마지막 vpos>5000 && 다음 para 첫 vpos<=0
    let is_empty_no_ctrl = para.text.is_empty() && para.controls.is_empty();
    if is_empty_no_ctrl {
        continue;                          // ← 빈 문단 skip (fit 미검사)
    }
    ...
}
```

재현 36399821: pi2(빈 문단, vpos=68606) 직후 pi3(2쪽 첫 표, vpos=0) → `next_will_vpos_reset=true`,
pi2 empty+no-ctrl → skip. 하지만 pi2 는 1쪽 하단(used 911 + 17 ≤ body 952)에 **들어감** → 누락은 오류.
형제 가드(`else if` next_force_break 분기)는 `empty_h_px > avail` fit 검사를 하나 이 분기는 누락.

## 3. 수정

`is_empty_no_ctrl` skip 에 **fit 검사 추가**(형제 가드와 동형): 빈 문단이 현재 페이지에 들어가면
정상 emit(한글 동작), 안 들어갈 때만 skip(단독 빈 페이지 차단 목적 보존).

단, 최초 height-only 검사는 회귀를 유발했고(§4 의 #1659), 최종 구현은 **height·vpos AND** 이다:

```rust
// 1) height fit: 합산 current_height 기준 (종전 #1648 판정)
let empty_h_px = para.line_segs.first()
    .map(|s| hwpunit_to_px((s.line_height + s.line_spacing) as i32, self.dpi))
    .unwrap_or(0.0);
let height_fits = empty_h_px <= st.available_height() - st.current_height;

// 2) [#1659] vpos fit: placement(L2333/L2339)와 동일한 page_top_vpos 기준 vpos 판정
//    (single-col 한정). PartialParagraph continuation 은 line_segs[start_line],
//    줄 기준 vpos 없는 항목(PartialTable)은 판정 보류(height 위임).
let vpos_fits = /* page_top_vpos + body_h_hu + 283 기준 vpos_end 비교, 불가 시 true */;

if !(height_fits && vpos_fits) { continue; }   // 둘 다 fit 일 때만 emit
```

## 4. 검증

### 4.1 #1659 회귀와 보정

최초 height-only 검사(`empty_h_px > avail`)는 **음수 줄간격 문단**이 있는 페이지에서 회귀를 냈다.
합산 `current_height` 가 실제 vpos 진행을 과소평가 → 페이지 하단 빈 문단을 현재 페이지에 fit 으로
오판 emit, 그러나 placement 는 vpos overflow 로 새 페이지에 단독 배치 → **단독 빈 페이지 +1**
(`samples/synam-001.hwp` 35→36, `issue_1156_rowbreak_fragment_fit` 의 한컴 PDF 페이지수 게이트 위반).

400-HWPX 통제셋이 이 패턴(음수 줄간격)을 포함하지 않아 최초 검증에서 누락됨. 보정으로 `vpos_fits`
조건을 AND 추가하여 placement 와 fit 판정을 일치시켰다(§3).

### 4.2 최종 검증

| 항목 | 결과 |
|------|------|
| 재현 36399821 | pi2 → **1쪽 배치**(전 미배치), 전체 PI 0..41 배치, 페이지수 5 유지 |
| synam-001 (#1659 회귀) | **35쪽**(한컴 PDF 일치) 복원 |
| sample16 p3 (#1648 대상) | 빈 문단 pi=87 **emit 보존**(items 19→20, 한컴 2022 PDF 검증) |
| `cargo test` 전체 | **2668 passed, 0 failed** (`issue_1156` + `issue_1116` 포함) |
| hwpx_roundtrip_baseline | pass |
| **통제 비교(old vs new, 400 HWPX, 결정론적)** | PAGE_INFLATE 0 / PAGE_DEFLATE 0 / ADD_ONLY(pNone→placed) 9 / REMOVED 0 / CROSS_MOVED 0 |
| fmt / clippy | clean |

**결론**: 수정은 종전 누락되던 빈 문단을 한글처럼 페이지 하단에 배치하되(통제셋 9건 pNone→placed),
음수 줄간격 페이지에서 단독 빈 페이지를 만들지 않아 **페이지수 중립**(synam-001 35 유지). 가드 본래
목적(단독 빈 페이지 차단)은 보존.

## 5. 산출물

- 소스: `src/renderer/typeset.rs` (빈 문단 fit 검사 = height·vpos AND, #1648 + #1659)
- 회귀 가드: `tests/issue_1116.rs`(스냅샷 갱신), `tests/issue_1156_rowbreak_fragment_fit.rs`(synam-001 35 유지)
- 검출 도구: `tools/verify_pi_page_vs_hangul.py` (#1643, MATCH 행 포함 TSV)
- 관련: #1643(vs-한글 검증), #1659(회귀 보정), closes #1648
