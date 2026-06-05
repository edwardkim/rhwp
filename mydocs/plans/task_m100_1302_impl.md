# 구현계획서 — Task #1302: 미주 다줄 문단 → 같은 미주 연속 문단 줄간격 과소

- **이슈**: edwardkim/rhwp#1302
- **브랜치**: `local/task1302` (base: `stream/devel` 9d3aa212)
- **수행계획서**: `task_m100_1302.md`

## 1. 근본 원인 (Stage 1 — 조사로 확정)

`3-11월_실전_통합_2022.hwpx` 18쪽 좌측 단 미주: pi=852(다줄, 분수 포함) 마지막 줄
"극솟값…갖는다" → pi=853 "(나)를…"(같은 문제 문30 **연속 문단**, 컬럼 0 마지막 줄).

계측(`RHWP_VPOS_DEBUG`):
```
pi=853 prev_pi=852 path=page
prev_vpos=953344 prev_lh=1050 prev_ls=452   vpos_end(curr_first)=954846
y_in=1057.77 (trailing 포함=정답)  end_y=1037.35 (page_base 매핑)
→ compact_endnote_page_tail_backtrack=TRUE → result=1051.75 (trailing 6px 제거)
```

- 원인 분기: `height_cursor.rs::vpos_adjust` 의 `compact_endnote_page_tail_backtrack`
  (조건 L400-407, 결과 L507-511 `end_y.max(prev_content_bottom_y).min(y_offset)`).
- 컬럼 하단(>95%)에서 `end_y`(page_base 절대매핑)가 다줄 문단 로컬앵커 대비 ~20px drift
  → `end_y < y_offset-8` 성립 → backtrack 발동 → `prev_content_bottom_y`(trailing 제외)로 당김.
- **핵심 모순**: stored gap = curr_first(954846) − prev_vpos(953344) = **1502HU
  = prev_lh(1050) + prev_ls(452)** = 정확히 정상 한 줄 전진. stored vpos 자체가 정상 연속을
  인코딩하는데 backtrack 이 trailing(ls)을 깎는다.
- 기존 #1246 rescue(L596-603)는 **다음 항목이 "문" 제목일 때만** trailing 복원 → 같은 문제
  **연속(비제목) + 컬럼 하단** 케이스는 미커버. #1236(중간 컬럼 trailing)도 미해당.

## 2. 수정 설계 (Stage 2)

`compact_endnote_page_tail_backtrack` 은 stored vpos 가 **overlap(작은/rewind gap)** 을 가리키는
tail 만 frame-fit 으로 끌어올리는 보정이다. stored gap 이 **정상 한 줄 전진 이상**이면 그건
overlap 이 아니라 정상 연속이므로 backtrack 대상이 아니다.

게이트 추가 (최소 변경):
```rust
// [Task #1302] curr 첫 줄 stored vpos 가 prev 한 줄 정상 전진(lh+ls) 이상을 인코딩하면
// overlap tail 이 아니다. page_base drift 로 end_y 가 위로 보여도 y_offset(=trailing 포함
// 정답)을 깎지 않는다.
let curr_first_full_advance = matches!(
    curr_first_vpos,
    Some(v) if v - seg.vertical_pos >= seg.line_height + seg.line_spacing
);
```
→ `compact_endnote_page_tail_backtrack` 조건에 `&& !curr_first_full_advance` 추가.

해당 케이스에서 backtrack 비활성 시 result 체인은 마지막 `else → y_offset`(1057.77)로 떨어져
정상 위치(gap 20px, stored·PDF 정합). (applied=false, deep/safe backtrack=false 확인 완료.)

**범위 한정**: 동일 패턴의 다른 tail backtrack 분기(text_after_tall/deep 등)는 본 버그
재현분기가 아니므로 **건드리지 않는다**. 회귀 테스트·샘플로 추가 누락 여부만 확인하고,
발견 시 별도 판단 (메모리 `tech_trailing_model_no_ssot`: 전면 통일 금지).

## 3. 회귀 테스트 (Stage 2)

`height_cursor.rs` 의 기존 `#[cfg(test)] mod tests` 에 핀 테스트 추가:
- `compact_endnote_page_tail_backtrack` 가 **stored full-advance gap** 에서 비발동
  (y_offset 유지) 확인.
- 기존 overlap/rewind tail backtrack 케이스는 그대로 발동(불변) 확인.

## 4. 검증 (Stage 3)

| 항목 | 기준 |
|------|------|
| 18쪽 극솟값→(나)를 gap | 12px → ~20px (SVG 측정, PDF ×1.334 ≈18px 정합) |
| 18쪽 그 외 줄간격 | 불변 |
| `cargo test` 전체 | 통과 (미주/trailing 핀 포함) |
| 회귀 샘플 | 3-11월 10~14·18쪽, 3-09월 17쪽(#1297), #1246/#1238 미주 |
| 페이지 수 | 21 불변 |

## 5. 단계

1. **Stage 1** 근본 원인 확정 — 완료 (본 문서 §1). `working/task_m100_1302_stage1.md` 기록.
2. **Stage 2** vpos_adjust 게이트 + 회귀 테스트 → 빌드·단위테스트 → `_stage2.md`.
3. **Stage 3** SVG↔PDF·전체 test·회귀 샘플 검증 → `_stage3.md` → 최종 `report/..._report.md`.

## 6. 승인 요청

본 구현계획서 승인 후 Stage 2 착수.
