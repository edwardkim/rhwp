# Stage 204: 저장 첫 조각 프레임의 행 경계 권위

## 계기

전체 통합 회귀에서 `issue_1156_rowbreak_fragment_fit::kps_ai_page37_defers_overflowing_split_row_slice`가 실패했다. `samples/kps-ai.hwp`의 `pi=329` RowBreak 표는 37쪽 본문 예산 887.1px에 행 0..16을 배치해야 하지만, 행 16까지 포함한 938.1px 조각이 확정되어 본문 하단을 51px 넘겼다.

## 원인

native HWP5의 저장 첫 조각 프레임은 소스 프레임과 측정 행 높이 사이의 작은 차이를 수용하기 위해 `source_first_fragment_overflow_allowance`를 제공한다. 그러나 이 값은 저장 프레임 아래의 물리 여유 전체를 일반적인 행 추가 허용치로 사용했다. 따라서 저장 프레임이 행 15 끝에 대응하는 표에서 다음 행 16도 프레임 여유로 수용할 수 있었다.

## 수정

저장 첫 조각 프레임의 높이와 가장 가까운 행 끝을 계산한다. 첫 조각 프레임 여유는 그 행 끝을 확정하거나 같은 행의 cut을 검증할 때만 사용할 수 있으며, 다음 행으로 진행할 때는 일반 RowBreak 예산을 다시 적용한다. 이 판정은 표 행 기하와 저장된 `common.height`만 사용하며 문서명, 페이지 번호, 픽셀 상수에 의존하지 않는다.

## 검증 대상

- `issue_1156_rowbreak_fragment_fit`: kps-ai 37쪽은 `rows=0..16`에서 끝나고 `end_cut`을 남기지 않아야 한다.
- `issue_3820_rowbreak_rowspan_band`: 저장 첫 조각 프레임이 소유한 본문/행 밴드는 기존 PDF 경계를 유지해야 한다.
- 전체 `cargo test --profile release-test --lib && cargo test --profile release-test --tests`를 다시 수행한다.
