# PR #2475 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2475](https://github.com/edwardkim/rhwp/pull/2475) |
| 작성자 / base | kevin9327 / `devel` |
| 범위 | HWPX 각주·미주 번호 모양과 시작 번호 왕복 보존 |
| 검토자 | @jangster77 (검토 전 지정) |
| 검토 스냅샷 | 2026-07-20 GitHub 조회: +92/-19, 1 file, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 판단 | 누적 통합 PR에 수용 |

## 검토와 검증

- PR 본문은 footnote/endnote 번호 종류와 시작 번호의 저장 누락을 설명했고, PR 코멘트는 없었다.
- 기여자 변경 `48316fde5`를 충돌 없이 적용했다. serializer round-trip 변경으로 visual sweep 대상은 아니다.
- HWPX round-trip focused 회귀와 최종 release-test, Clippy, WASM 빌드를 통과했다.

- headless save-as-format E2E에서 HWP→HWPX와 HWPX→HWP 저장 UI, MIME/파일 매직, 재열기를 모두 확인했다. 이는 사용자 저장 경로 smoke이며, PR별 속성 보존의 oracle은 기존 focused Rust 회귀다.

## 렌더 영향 판정

- HWPX 각주·미주 번호 metadata의 serializer 보존만 다루며 renderer·layout 출력 경로를 바꾸지 않는다. visual sweep은 필요하지 않다.

## 리스크와 권고

- 각주·미주 numbering metadata의 왕복 보존만 다루며 렌더 구현을 바꾸지 않는다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
