# PR #2491 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2491](https://github.com/edwardkim/rhwp/pull/2491) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +120/-0, 7 files, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | Safari icons/_locales 누락 보완 및 Dependabot ecosystem 범위 확장 |
| 판단 | collaborator 보정 포함 누적 통합 PR에 수용 |

## 변경 범위와 통합
- PR 본문은 Safari source tree에 누락된 정적 자산을 복구하고 Dependabot 대상 ecosystem을 보완한다.
- PR 코멘트는 검토 시점에 없었다.
- 기여자 변경 `d03ce9224`, `6fa5a9e8d`을 적용했다. Safari runtime 오류 전파 보정 `0115ede07`은 contributor 변경과 분리해 collaborator 커밋으로 추가했다.

## 렌더 영향 판정
- 정적 자산 누락과 패키지 갱신이 범위이며 renderer 출력 변경은 아니다.

## 검증
- 누적 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`, `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg`를 통과했다.
- Chrome/Firefox production build와 Safari unsigned Xcode build를 통과했다. `rhwp-safari/build.sh`의 종료 65는 Mac Development 인증서·CoreSimulator 환경 부재에 따른 서명 단계 제한이다.

## 리스크와 권고
- Safari 서명 산출물의 실기기 설치는 인증서가 있는 배포 환경에서 재확인해야 한다.
- **권고**: collaborator 보정 포함 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
