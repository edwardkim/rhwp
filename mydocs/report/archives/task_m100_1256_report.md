# 최종 결과보고서 — Task #1256: 미주 답안 제목 between-notes(7mm) 간격 누락

- 이슈: edwardkim/rhwp#1256 · 브랜치: `local/task1256` (from `devel`)
- 마일스톤: v1.0.0 (M100)
- 관련: #1253, #1248, #1246, #1189, #1209→#1256, 후속 #1257
- 재현 문서: `samples/3-09월_교육_통합_2022.hwpx` / `.hwp`

## 1. 문제

미주 답안 영역에서 `문N)` 제목 **위 줄 간격이 한컴보다 좁다**(작업지시자 보고: 9·10·13·14·
18·19·20·28쪽의 문5/6/7/12/16/18/19/20/21/23/24/25/26/27/28/29). 정량: 페이지9 문6→문7
구간이 한컴 PDF 287px 대비 우리 264px(−23px ≈ 7mm 한 줄).

## 2. 근본원인 (RHWP_VPOS_DEBUG 확정)

- 문서 `미주 사이`(between-notes) = **7mm = 1984 HU**(`noteSpacing betweenNotes="1984"`),
  파서 `shape.raw_unknown` 정상 저장. typeset(#1246)이 경계마다 prev 마지막 seg
  `line_spacing=1984` 주입 → render 순차 `y_offset` 은 7mm 포함.
- 그러나 제목 stored 절대-vpos 에는 7mm 없음(prev_vpos+1502). render 의
  `compact_endnote_safe_vpos_backtrack`/`applied`(height_cursor.rs)가 `end_y`(절대,7mm 미포함,
  =y_offset−20.4px)를 반환 → **주입된 7mm 을 덮어써 제목을 ~20.4px 위로 cram**.
- 전 문서 **15건**(safe_backtrack 시그니처 `prev_ls≈1984 & end_y≈y_in−20.4px`). `.hwp`/`.hwpx`
  동일 → 포맷 무관 공통 미주 경로.

## 3. 수정 (path-1) — `src/renderer/height_cursor.rs` `vpos_adjust`

단일 줄 prev(빈 separator)로 끝나는 미주 제목 경계에서, 베이스라인 result 가 y_offset 아래로
cram 되면(`stored_gap_px < -0.5`) **y_offset(7mm 포함)을 유지**하고, 내린 만큼 **활성 vpos base
를 path-aware 로 이동**해 후속 미주 항목이 동일 기준을 따르게 한다.

```rust
let injected_between_notes =
    self.endnote_between_notes_hu > 0 && seg.line_spacing >= self.endnote_between_notes_hu;
if injected_between_notes && compact_endnote_question_title
    && !vpos_rewind && !prev_is_multiline && stored_gap_px < -0.5 {
    let restored_hu = ((y_offset - result) / self.dpi * 7200.0).round() as i32;
    if restored_hu > 0 { /* is_page_path ? vpos_page_base : vpos_lazy_base -= restored_hu */ }
    return y_offset;
}
```

- base-shift 동반이 핵심(누락 시 +overflow desync — #1246/#1082 회귀의 본질).
- `stored_gap_px < -0.5` 가드: 베이스라인이 실제 cram 일 때만 복원 → 컬럼 하단 spurious shift 방지.
- 다줄 prev(문22)는 별개 경로 → 기존 #1246 rescue(+prev_ls) 유지.

## 4. 검증

| 항목 | 결과 |
|------|------|
| 페이지9 문6→문7 | **287.0px** (한컴 PDF 287px 정확 일치) |
| 페이지13 헤더 baseline | PDF와 1~4px 내 정합(전: ~20px 위 cram) |
| 복원 헤더 | 15건 (safe_backtrack 시그니처) |
| 페이지 수 | 3-09 23/23, 미주사이20 24, 구분선아래20 23, 3-11 21 (전부 불변) |
| 오버플로우 | 0 (전 페이지 max content-y ≤ 1089.8, envelope 내) |
| 단위테스트 | height_cursor 2건 추가(갭 유지+base 이동 / 자연 trailing 무영향) |
| `cargo test` 전체 | **1963 passed, 0 failed** |

## 5. 부수 변경 — #1209 테스트 정정 (작업지시자 승인)

`issue_1209_2022_sep_page10_question12_uses_safe_vpos_backtrack` 는 문12 를 cram 위치
(390–406px)로 단언했으나 **한컴 PDF(10쪽)는 문12) 위 7mm 갭이 명확**(문11 "k=9" 다음 빈 줄).
PDF·작업지시자 보고와 모순이라 PDF 기준으로 정정:
`issue_1256_2022_sep_page10_question12_keeps_between_notes_gap`, q12_y `390..406`→`410..426`
(나머지 단언=수식 중앙정렬·꼬리 간격은 #1209 그대로 통과).

## 6. 범위 외 (후속 #1257)

1. **컬럼-하단 문5 등**(lazy_base<0 SKIP + task-1189 cram): cram 해제 시 **3-11월 10쪽 7.6px
   오버플로우 회귀**. `fits_with_full_gap`(제목 첫 줄만) 가드 불충분 → path-2 되돌림, #1257 분리.
2. **일부 mid-column 제목**(예: 미적분 문24–26, `end_y≥y_in` 부분 갭 ~6px): safe_backtrack
   시그니처 아님 → 무변경(판별자 의도적 미해당, 과확장 시 overflow 방지). 저심각 잔존.

둘 다 근간은 [[#1184]] typeset↔render 절대-vpos 공유모델로, 안전한 완전정합은 배치모델
재설계 영역. 본 task 는 회귀 0 범위에서 명백한 15건을 PDF 정합 복원한다.

## 7. 변경 파일

- `src/renderer/height_cursor.rs` (path-1 수정 + 단위테스트 2건)
- `tests/issue_1139_inline_picture_duplicate.rs` (#1209→#1256 정정)
- `mydocs/` 계획서·단계보고서·최종보고서

## 8. 커밋

- `92605b68` 계획서 + Stage1 베이스라인
- `406ea082` path-1 핵심 수정 + #1209 정정
- `9d26e733` Stage3 정합·회귀 검증
- (Stage4) 단위테스트 + 최종보고서

## 9. 후속

- #1257: 컬럼-하단/부분갭 미주 제목 between-notes 정합 (절대-vpos 재설계 영역)
