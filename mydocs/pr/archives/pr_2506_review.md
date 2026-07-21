# PR #2506 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2506](https://github.com/edwardkim/rhwp/pull/2506) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +4/-3, 2 files, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | Chrome content script와 DevTools 주입 스크립트의 extension version 표기 정합성 |
| 판단 | collaborator 보정 포함 누적 통합 PR에 수용 |

## 변경 범위와 통합
- PR 본문은 Chrome extension 버전 문자열을 manifest에서 읽도록 해 배포 버전과 화면 표시가 어긋나지 않게 한다.
- PR 코멘트는 검토 시점에 없었다.
- 기여자 원 변경 `2c56f6a9b`을 적용한 뒤, producer가 쓰는 기존 DOM 속성은 `data-hwp-extension-version`인데 consumer가 다른 이름을 읽던 결함을 collaborator 보정 `042ab976f`으로 수정했다.

## 렌더 영향 판정
- DevTools/extension metadata 표기 경로 변경이며 문서 렌더 출력 변경은 아니다.

## 검증
- 누적 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`, `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg`를 통과했다.
- `node --test dev-tools-version.test.mjs sw/*.test.mjs` 15개 통과, `rhwp-chrome` syntax 검사와 `npm run build`를 통과했다.

## 리스크와 권고
- DOM attribute contract를 기존 producer 이름으로 맞춰 backward compatibility를 유지한다.
- **권고**: collaborator 보정 포함 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
