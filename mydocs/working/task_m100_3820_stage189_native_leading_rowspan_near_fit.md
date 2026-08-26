# Stage 189: native HWP 선두 rowspan 밴드 near-fit 보정

## 관찰

Stage 186의 source-frame guard 이후 `2025 행정업무운영 편람(최종).hwp`는 383쪽에서
384쪽으로 늘었다. Stage 186 직전 같은 브랜치와 페이지별 텍스트를 비교하면 첫 실제 분기는
300쪽이며, 이전 출력에서는 10절 65문단의 6x5 문답 표(33번)가 이 쪽에 배치되고 현재 출력에서는
테두리만 있는 21.5px 첫 조각만 남은 뒤 나머지 행이 다음 쪽으로 이동한다.

재현 표의 첫 논리 행은 다음 행까지 이어지는 rowspan을 가진다. 반면 #3820 p94의 잘못된
near-fit 표는 4x3이며 모든 셀의 rowspan이 1이다. 두 경우 모두 명시적인 cell-content source
frame은 없으므로, source-frame 유무만으로 native HWP의 near-fit을 판정하면 전자의 저장된
원자적 헤더 밴드까지 잃게 된다.

## 수정

첫 논리 행에서 시작해 다음 행으로 이어지는 rowspan을 `has_leading_rowspan_band`로 계산한다.
native HWP RowBreak 표의 near-fit은 기존 source frame이 있거나 이 선두 밴드가 있을 때만
허용한다. 따라서 빈 테두리 조각을 만들지 않고 문답 표를 선언된 프레임 안에 유지하면서,
rowspan이 없는 p94 표에는 Stage 186 guard가 계속 적용된다. HWPX는 Stage 188의 profile
분리를 그대로 사용한다.

## 검증 대상

- `issue_3820_rewinding_rowbreak_uses_painted_first_fragment_boundary`
- `tests/issue_3820_rowbreak_rowspan_band.rs`
- `tests/issue_3930_hwpx_hwp_save_layout.rs`

## 검증 결과

- 직접 계약: 1개 통과
- rowspan 밴드 회귀: 4개 통과
- HWPX-HWP 저장 레이아웃: 3개 통과, native HWP 383쪽 복구
