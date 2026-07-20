# PR #2477 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2477](https://github.com/edwardkim/rhwp/pull/2477) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +1/-1, 1 file, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | Safari content-script의 `edit`/`print` capability 선언 |
| 판단 | 누적 통합 PR에 수용 |

## 변경 범위와 통합
- PR 본문은 Safari content-script capability가 실제 지원 기능보다 좁아 편집/인쇄 접근이 막히는 문제를 다룬다.
- PR 코멘트는 검토 시점에 없었다.
- 기여자 원 변경 `0a9c378eb`을 적용했다.

## 렌더 영향 판정
- 확장 capability 선언만 변경하므로 visual sweep 대상이 아니다.

## 검증
- 누적 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`, `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg`를 통과했다.
- Safari source/build 검증은 #2473과 같은 서명 환경 제약을 제외하고 unsigned Xcode build까지 통과했다.

## 리스크와 권고
- Safari 실기기 권한 동작은 서명된 배포 산출물에서 확인해야 하나, 코드 범위는 선언 정합성에 한정된다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
