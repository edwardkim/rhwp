# PR #2476 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2476](https://github.com/edwardkim/rhwp/pull/2476) |
| 작성자 / base | kevin9327 / `devel` |
| 범위 | HWPX 쪽 번호 홀수/짝수 시작 `pageStartsOn` 왕복 보존 |
| 검토자 | @jangster77 (검토 전 지정) |
| 검토 스냅샷 | 2026-07-20 GitHub 조회: +44/-2, 2 files, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 판단 | 누적 통합 PR에 수용 |

## 검토와 검증

- PR 본문은 `pageStartsOn`이 HWPX 왕복 뒤 유실되는 문제를 설명했고, PR 코멘트는 없었다.
- 기여자 변경 `da5e83970`을 충돌 없이 적용했다. 구조 보존 변경으로 visual sweep은 merge 판단의 직접 근거가 아니다.
- HWPX round-trip focused 회귀와 최종 release-test, Clippy, WASM 빌드를 통과했다.

- headless save-as-format E2E에서 HWP→HWPX와 HWPX→HWP 저장 UI, MIME/파일 매직, 재열기를 모두 확인했다. 이는 사용자 저장 경로 smoke이며, PR별 속성 보존의 oracle은 기존 focused Rust 회귀다.

## 렌더 영향 판정

- HWPX 쪽 번호 시작 metadata의 serializer 보존만 다루며 pagination 알고리즘을 바꾸지 않는다. visual sweep은 필요하지 않다.

## 리스크와 권고

- 쪽 번호 시작 parity metadata의 왕복 보존만 다루며 pagination 알고리즘을 바꾸지 않는다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
