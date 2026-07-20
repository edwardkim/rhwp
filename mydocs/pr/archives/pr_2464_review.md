# PR #2464 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2464](https://github.com/edwardkim/rhwp/pull/2464) |
| 작성자 / base | kevin9327 / `devel` |
| 범위 | HWPX 비표 개체의 `holdAnchorAndSO` 왕복 보존 |
| 검토자 | @jangster77 (검토 전 지정) |
| 검토 스냅샷 | 2026-07-20 GitHub 조회: +20/-1, 1 file, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 판단 | 누적 통합 PR에 수용 |

## 검토와 검증

- PR 본문은 비표 개체에서 쪽나눔 방지 속성이 유실되는 문제를 설명하며, PR 코멘트는 없었다.
- 기여자 변경 `8f28a0add`를 충돌 없이 적용했다. renderer 직접 변경은 아니므로 visual sweep 대상이 아니다.
- 해당 HWPX 왕복 focused 회귀와 최종 `cargo test --profile release-test --tests`, Clippy, WASM 빌드를 통과했다.

- headless save-as-format E2E에서 HWP→HWPX와 HWPX→HWP 저장 UI, MIME/파일 매직, 재열기를 모두 확인했다. 이는 사용자 저장 경로 smoke이며, PR별 속성 보존의 oracle은 기존 focused Rust 회귀다.

## 렌더 영향 판정

- HWPX serializer의 비표 개체 속성 보존만 다루며 renderer·layout 출력 경로를 바꾸지 않는다. visual sweep은 필요하지 않다.

## 리스크와 권고

- 비표 개체의 쪽나눔 방지 속성만 대상으로 한 구조 보존 변경이며, focused round-trip 회귀로 범위를 고정했다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
