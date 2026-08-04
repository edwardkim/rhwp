# PR #2662 검토 기록

| 항목 | 내용 |
|---|---|
| PR | [#2662](https://github.com/edwardkim/rhwp/pull/2662) |
| 작성자 / base | [@planet6897](https://github.com/planet6897) / `devel` |
| 관련 이슈 | [#2527](https://github.com/edwardkim/rhwp/issues/2527) |
| 원 commit / 누적 적용 | `cfe846fa` / `2b6014714` (충돌 없음, 선행 의존 없음) |
| 범위 | 빈 `linesegarray` HWPX fixture와 native as-is/auto-fix 회귀 테스트 |
| 처리 경로 | collaborator 누적 통합 검토. merge 전 최신 원 PR 상태와 CI 재확인 필요 |

## 변경과 검증

- 텍스트 문단과 표 셀을 포함한 HWP5-origin 빈 lineseg fixture를 추가해 validation 경고 5건,
  as-is 비겹침, `reflowLinesegs()` 뒤 비겹침을 고정한다.
- focused `issue_2527_empty_lineseg_reflow` 3/3, 통합 `cargo test --profile release-test --tests`,
  `cargo clippy --all-targets -- -D warnings`가 모두 성공했다.
- 렌더 코드 변경이 아닌 native 구조 회귀 가드이므로 별도 visual sweep 대상은 아니다.

## 한계와 권고

원 PR 본문대로 이 합성 fixture는 원래의 Studio/WASM CanvasKit 좌표 붕괴를 native 경로에서 재현하지
않는다. 따라서 이 PR은 안전한 장기 regression fixture 보강으로 수용하되,
[#2527](https://github.com/edwardkim/rhwp/issues/2527)의 실제 브라우저 재현·폰트 readiness 근본 보정은
open으로 유지한다. 최신 head CI와 작업지시자 승인이 충족되면 통합 PR로 merge 가능하다.
