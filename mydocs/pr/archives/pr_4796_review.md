# PR #4796 검토 - SVG 배치 메트릭 글꼴 주석 옵트인

## 메타데이터

| 항목 | 값 |
| --- | --- |
| 원 PR | [#4796](https://github.com/edwardkim/rhwp/pull/4796) |
| 통합 PR | [#4801](https://github.com/edwardkim/rhwp/pull/4801) |
| 관련 이슈 | `Closes #4709` |
| 작성자·검토 방식 | `planet6897` · collaborator 체리픽 통합 self-review |
| 원 base / head | `devel` / `397982cc0fd93ab15295110a8b5a2d2088756ff7` |
| 적용 commit | `d69b52996` |
| 통합 code candidate | `c7e2f1fe5586eca576daeb58495da5718f7eebfc` |
| 규모 | 9 files, +145/-0 (원 PR 기준) |
| 라우팅 | collaborator external PR · intake/review · local validation · visual fixture evidence |

원 기능 commit은 최신 `upstream/devel@44bcba400072128bdc4e4d6c05bf822e3ff60996` 위에 충돌 없이
체리픽했다. 이 기록은 code candidate가 Full CI를 통과한 뒤 추가하는 review-only trailing 문서다.
따라서 merge 직전에는 이 문서 head의 fast-pass와 최신 mergeability를 다시 확인한다.

## 변경 범위와 판단

- SVG text 요소에 실제 배치 계산에 사용한 글꼴을 `data-metric-font`으로, 문서 루트에는 사용 글꼴 목록을
  `data-rhwp-metric-fonts`로 선택적으로 기록한다.
- 기본값은 비활성화이며 Rust `DocumentCore`, WASM API, Studio, CLI `export-svg --annotate-metric-font`에서
  명시적으로만 켠다. 기존 SVG 직렬화와 렌더 결과는 기본 경로에서 바뀌지 않는다.
- 옛한글 표시 글꼴과 배치 글꼴이 다를 수 있는 경우도 확인했다. 주석은 표시 대체 글꼴이 아니라 실제
  `style.font_family` 측정값을 보여 주므로 layout debugging 용도로 정확하다.

메인터너 보정이 필요한 API 호환성·기본값 변경·출력 오염은 발견하지 못했다.

## 완료된 검증

- `cargo test --profile release-test --target-dir target\pr-review --test issue_4709_metric_font_annotation -- --nocapture`
  를 통과했다(1 passed).
- `git diff --check upstream/devel...HEAD`를 통과했다.
- code candidate `c7e2f1fe5`의 GitHub Actions에서 Lint(WASM check 포함), Build & Test 전체 shard,
  Native Skia, Canvas visual diff, CodeQL이 모두 성공했다.

## 위험과 후속 범위

- 이 주석은 layout 분석용 메타데이터다. 설치 글꼴 이름을 외부 SVG에 노출하는 것이 부담인 소비자는
  옵션을 켜지 않아야 한다.
- 실제 페이지 모양이나 글꼴 fallback 정책은 변경하지 않는다. 해당 범위는 별도 renderer 변경으로 다룬다.

## 최종 권고

수용을 권고한다. #4801의 review-only trailing head가 fast-pass 조건을 만족하고 최신
`MERGEABLE/CLEAN` 상태를 재확인한 뒤, 작업지시자 승인에 따라 통합 PR로 병합한다.
