# Stage 205: HWPX 저장 프레임의 문단 내부 경계 권한

## 목적

`samples/hwpx/pr-1674.hwpx`가 원본 HWP와 달리 36쪽으로 분리되는 회귀를 제거한다.
기준 HWP와 HWPX 모두 35쪽이어야 한다.

## 관찰

- 전체 회귀의 `issue_1686_hwpx_matches_pdf_page_count_and_page5_boundary`만
  `36 != 35`로 실패했다.
- `pi=169`의 16x3 `CELL` 표에서 HWPX는 행 9를 `end_cut=[1, 9]`로 잘라
  다음 쪽에 36.7px 꼬리를 만들었다. 원본 HWP는 같은 표를 해당 쪽에서 끝냈다.
- `RHWP_DIAG_SCAN`은 행 9의 두 번째 가시 셀만 문단 내부 `vertpos > 0 -> 0`을
  소유하며, 그 reset이 273.1px 예산 중 152.2px에서 strict cut을 강제함을 보였다.
- 첫 번째 셀도 가시 내용을 가지므로, 한 셀의 저장 좌표를 표 전체 물리 경계로
  확대하면 121px의 본문 공간을 버리고 tail-only 조각을 만든다.

## 변경 계약

- 저장 프레임 컷과 strict saved-frame hard break는
  `row_has_stored_vpos_frame_rewind()`와
  `row_has_single_visible_source_cell()`가 모두 참인 경우만 물리 경계로 인정한다.
- 재귀 중첩 표와 single-cell nested host의 기존 strict 경계는 유지한다.
- 여러 가시 셀 중 하나의 reset은 일반 행 용량 계산과 기존 orphan/sliver 규칙으로 처리한다.

## 검증 명령

```sh
cargo test --profile release-test --test issue_1686 \
  --test issue_3820_rowbreak_rowspan_band \
  --test issue_1156_rowbreak_fragment_fit -- --nocapture
```
