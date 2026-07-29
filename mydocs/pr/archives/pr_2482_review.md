# PR #2482 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2482](https://github.com/edwardkim/rhwp/pull/2482) |
| 작성자 / base | kevin9327 / `devel` |
| 범위 | HWP5 각주·미주 닫는 장식 문자 `0` 보존 |
| 검토자 | @jangster77 (검토 전 지정) |
| 검토 스냅샷 | 2026-07-20 GitHub 조회: +61/-12, 2 files, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 판단 | 누적 통합 PR에 수용 |

## 검토와 검증

- PR 본문은 닫는 장식 문자 `0`이 `)`로 오염되는 문제를 설명했고, PR 코멘트는 없었다.
- 기여자 변경 `d7a2e556d`를 충돌 없이 적용했다. serializer 보존 변경이며 visual sweep 대상은 아니다.
- HWP5 serializer focused 회귀와 최종 release-test, Clippy, WASM 빌드를 통과했다.

- headless save-as-format E2E에서 HWP→HWPX와 HWPX→HWP 저장 UI, MIME/파일 매직, 재열기를 모두 확인했다. 이는 사용자 저장 경로 smoke이며, PR별 속성 보존의 oracle은 기존 focused Rust 회귀다.

## 렌더 영향 판정

- HWP5 각주·미주 장식 문자 sentinel의 serializer 보존만 다루며 renderer 출력 경로를 바꾸지 않는다. visual sweep은 필요하지 않다.

## 리스크와 권고

- 닫는 장식 문자 `0`의 sentinel 의미를 보존하며 임의의 장식 문자 변환을 추가하지 않는다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
