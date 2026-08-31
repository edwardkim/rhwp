# PR #6526 검토 - float host split line segment advance

- 검토일: 2026-08-31
- 작성자: `planet6897`
- base: `devel` (`upstream/devel@99419b6b2`에서 통합 시작)
- 원 PR head: `76a7e0e62edcad0207a0d543f68ef450359830b8`
- 통합 commit: `1a0923e9ed`, `7fb1ae1e03`
- 상태: 승인 (통합 검증본 기준)

## 범위

- float host가 같은 visual line에 나뉘어 저장된 경우에도 line segment advance가 유지되도록 렌더 레이아웃을 보정한다.
- `issue6524/30098_float_host_split_lineseg.hwp` fixture와 회귀 테스트를 추가한다.

## 검토 결과

- 정적 검토에서 기존 `same vertical_pos + different column_start` 판정과 일관된 조건으로 적용됨을 확인했다.
- 목표 회귀 테스트 `issue_6524_float_host_split_lineseg_advance`는 `release-test`에서 종료 코드 `0`으로 통과했다.
- Hancom 2020 기준 PDF와 p3 직접 비교를 완료했다. 자동 위험 신호는 `0`건이며, review 패널에서 split line 주변 흐름과 도형 경계가 유지됨을 확인했다.
- 시각 증적: [p3 review 패널](assets/pr_6526_issue6524_p3_review.png)
- 기준 PDF: `pdf/pr_6526_issue6524_p3_2020.pdf`, SHA-256 `d3157eb507517ece5dad5b33996933bf3dd37b8b0c73e2f9561b6792ae9962df`
- visual sweep: pixel match `91.34392%`, ink match `43.04972%`; 글꼴 rasterizer 차이는 있으나 flagged page 없음.

## 공통 검증

- `cargo fmt --all && cargo fmt --all -- --check`
- native/WASM/workspace/all-target Clippy 및 workspace build 통과
- 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` 종료 코드 `0`

## 병합 조건

- 원격 병합 또는 통합 PR 게시 직전에 원 PR head와 CI green 상태를 다시 확인한다.

