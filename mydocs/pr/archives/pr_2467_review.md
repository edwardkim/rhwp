# PR #2467 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2467](https://github.com/edwardkim/rhwp/pull/2467) |
| 작성자 / base | kevin9327 / `devel` |
| 범위 | HWPX-HWP 변환의 중첩 컨테이너 개체 보강 |
| 검토자 | @jangster77 (검토 전 지정) |
| 검토 스냅샷 | 2026-07-20 GitHub 조회: +80/-0, 1 file, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 판단 | 누적 통합 PR에 수용 |

## 검토와 검증

- PR 본문은 각주·미주·머리말·꼬리말·바탕쪽·캡션 내부 개체가 HWP 변환 보강에서 누락되는 문제를 다뤘고, 코멘트는 없었다.
- 기여자 변경을 충돌 없이 적용했다. 같은 컨테이너 참조 수집 보강은 [#2483](https://github.com/edwardkim/rhwp/pull/2483)과 함께 검증했다.
- HWPX-HWP 변환 focused 회귀와 최종 release-test, Clippy, WASM 빌드를 통과했다.

- headless save-as-format E2E에서 HWP→HWPX와 HWPX→HWP 저장 UI, MIME/파일 매직, 재열기를 모두 확인했다. 이는 사용자 저장 경로 smoke이며, PR별 속성 보존의 oracle은 기존 focused Rust 회귀다.

## 렌더 영향 판정

- HWPX→HWP 변환의 중첩 개체 보강만 다루며 renderer·typeset 출력 경로를 바꾸지 않는다. visual sweep은 필요하지 않다.

## 리스크와 권고

- 중첩 컨테이너 보강 범위가 넓으므로 [#2483](https://github.com/edwardkim/rhwp/pull/2483)의 참조 수집 보강과 함께 회귀를 유지한다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
