# PR #2473 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2473](https://github.com/edwardkim/rhwp/pull/2473) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +1/-1, 1 file, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | Safari 확장 manifest의 버전을 배포 버전과 동기화 |
| 판단 | 누적 통합 PR에 수용 |

## 변경 범위와 통합
- PR 본문은 Safari manifest가 오래된 `0.2.1` 버전을 선언하는 문제를 `0.2.8`로 동기화한다.
- PR 코멘트는 검토 시점에 없었다.
- 기여자 원 변경 `850e22e05`을 적용했다.

## 렌더 영향 판정
- manifest 메타데이터만 변경하므로 visual sweep 대상이 아니다.

## 검증
- 누적 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`, `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg`를 통과했다.
- `rhwp-safari/build.sh`로 dist 생성까지 확인했고, 로컬 Mac Development 인증서 부재로 인한 서명 단계 실패는 코드 실패가 아니다. `CODE_SIGNING_ALLOWED=NO` Xcode build는 통과했다.

## 리스크와 권고
- Safari 배포 서명은 인증서가 있는 배포 환경에서 별도로 수행해야 한다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
