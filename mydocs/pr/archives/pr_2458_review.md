# PR #2458 검토 - 편집 스윕 상설화 (#2355)

| 항목 | 내용 |
|---|---|
| PR | [#2458](https://github.com/edwardkim/rhwp/pull/2458) |
| 작성자 / base | [@chrisryugj](https://github.com/chrisryugj) (첫 기여) / `devel` |
| 관련 이슈 | [#2355](https://github.com/edwardkim/rhwp/issues/2355) |
| 검토자 | [@jangster77](https://github.com/jangster77) |
| 최종 판단 | 수용 및 merge 완료 |
| merge | [`e50958c`](https://github.com/edwardkim/rhwp/commit/e50958c613e05e74ffc18afb39d2cfe86d80b6d1) |

## 변경 범위

- 일회성으로 사용했던 전 샘플 편집 전후 페이지 수 비교를 `examples/edit_sweep.rs` Cargo 예제로 상설화한다.
- 기본 `insert1` 편집으로 `.hwp`, `.hwpx`, `.hml`을 재귀 스캔해 전후 `page_count()`와 파일별 상태를 TSV로 기록한다.
- `--compare`는 공통 변동, 해소, 신규 변동, 상태 전이를 Markdown으로 분류하며 신규 변동은 exit 1로 반환한다.
- [편집-스윕 하니스](../../manual/verification/edit_sweep.md)와 verification 문서 지도, `CONTRIBUTING.md` 사용 안내를 추가한다.

## 렌더 영향 판정

- renderer, layout, PDF/SVG 출력 계약을 변경하지 않는 개발 검증 도구와 문서 변경이다.
- 따라서 visual sweep은 대상이 아니며, 도구의 실제 스윕·비교·오류 격리 동작을 검증 근거로 사용했다.

## 검증

- `cargo fmt --all -- --check` 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 통과.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` 통과.
- release `edit_sweep samples` 실행: 581건, 변동 9건, 오류 0건, 실행 14초.
- self-compare는 신규 0건과 exit 0을, synthetic TSV 신규 변동은 신규 1건과 exit 1을 확인했다.
- 빈 HWP 입력도 스윕 전체를 중단하지 않고 `parse_error` 상태 행으로 기록됨을 확인했다.
- 최신 PR head 기준 CI, CodeQL, Render Diff가 모두 성공했다.

## 판단과 후속

- [#2355](https://github.com/edwardkim/rhwp/issues/2355)의 상설 도구화, baseline 대조, 반복 가능한 사용법이라는 수용 기준을 충족해 merge했다.
- 디렉터리 symlink 순환이 있는 입력 트리에서 재귀 스캔이 끝나지 않을 수 있는 견고성 후보는 이번 merge의 차단 사유로 보지 않았다. 후속에서 symlink 디렉터리를 건너뛰거나 canonical-path 방문 집합으로 순환을 막는 방안을 검토한다.
- [감사 및 후속 안내 코멘트](https://github.com/edwardkim/rhwp/pull/2458#issuecomment-5020630962)를 남겼다. 첫 기여자에게 이 후속 후보의 구현을 요구하지 않으며 maintainer 측 backlog로 관리한다.
- 이슈는 PR merge 뒤 GitHub Actions가 자동 close했으며, [자동 close 기록](https://github.com/edwardkim/rhwp/issues/2355#issuecomment-5020621834)으로 확인했다.
