# Stage 191: 선언 높이 없는 HWPX reset의 source frame 배제

## 원인

최신 `upstream/devel`의 전체 회귀는 0건이고, #3820 브랜치의
`test_advance_row_cut_hwpx_midpage_vpos_reset_is_absorbed`만 HWPX RowBreak synthetic
표에서 6개 unit 대신 4개를 소비했다. 이 테스트 표는 선언 `common.height`가 0인데,
direct HWPX RowBreak라는 형식과 `vpos=0` reset만으로 물리 stored frame으로 승격됐다.

선언 object frame이 없는 표는 저장 좌표 reset이 실제 페이지 경계인지 검증할 기하학적 근거가
없다. 반대로 #3930의 실제 HWPX 표는 선언 높이를 가지며, 기존의 saved-frame Q&A 계약을
계속 사용해야 383쪽과 p283 응답 행의 소유권을 보존한다.

## 수정

direct HWPX RowBreak의 stored-frame 판정은 `common.height > 0`인 실제 저장 object에만
적용한다. 선언 높이 0인 synthetic 표는 기존 mid-page reset 흡수 경로를 사용한다. 이로써
테스트 fixture의 로컬 좌표 재시작은 물리 쪽 경계로 오인하지 않으면서 실제 문서의 저장
frame 계약은 변경하지 않는다.

## 검증 대상

- `test_advance_row_cut_hwpx_midpage_vpos_reset_is_absorbed`
- `tests/issue_3930_hwpx_hwp_save_layout.rs`
- `issue_3820_rewinding_rowbreak_uses_painted_first_fragment_boundary`
- `tests/issue_3820_rowbreak_rowspan_band.rs`

## 검증 결과

- HWPX 중간 reset 단위 계약: 1개 통과
- HWPX-HWP 저장 레이아웃: 3개 통과, 383쪽과 p283 응답 소유권 유지
- native #3820 직접 계약: 1개 통과
- rowspan 밴드 회귀: 4개 통과
