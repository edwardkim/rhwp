# Stage 243: PR #4755 리베이스 후 rustfmt 정합

## 목표

최신 `upstream/devel`의 PR #4754와 PR #4755 병합분 위로 #3820 브랜치를 리베이스한 뒤, 현재 Rust 도구 체인이 보고한 형식 차이를 정규화한다.

## 배경

- PR #4754와 현재 브랜치의 변경 파일은 겹치지 않았다.
- PR #4755와는 `typeset.rs`, `table_layout.rs`, `composer.rs`를 포함한 10개 파일이 겹쳤다.
- 리베이스 과정의 유일한 충돌은 `composer.rs`의 import 목록이었으며, PR #4755의 `control_line_seg_index`와 #3820의 metric 계산 함수들을 모두 보존했다.
- 리베이스 후 `cargo fmt --all -- --check`가 #3820 코드 6개 파일의 형식 차이를 보고했다.

## 변경

- `cargo fmt --all`로 보고된 Rust 소스만 현재 rustfmt 규칙에 맞춘다.
- 계산 조건, 저장 frame 판정, 페이지 소유권 및 수치 계약은 변경하지 않는다.
- 문서별 또는 페이지별 예외는 추가하지 않는다.

## 검증

- `cargo fmt --all -- --check`: 종료 코드 `0`
- 전체 라이브러리 회귀: 제거된 `tokenize_paragraph_with_split_cell_space_metric` 호출 2곳으로 컴파일 실패
- integration 회귀와 Clippy: 라이브러리 컴파일 실패로 미실행
- 후속 Stage 244에서 PR #4755의 `tokenize_paragraph_with_regenerated_space_metric` 계약에 호출부를 정합한다.
