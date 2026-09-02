# PR #4774 검토 - 저장 LineSeg 기반 Square 그림 스택 앵커 유지

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4774](https://github.com/edwardkim/rhwp/pull/4774) |
| 관련 이슈 | [#4770](https://github.com/edwardkim/rhwp/issues/4770) |
| 작성자 | `planet6897` (`Jaeuk Ryu`) |
| 검토 방식 | 작업지시자 승인에 따른 collaborator 셀프 리뷰 및 maintainer 보정 |
| base / head | `devel` / `pr/devel-4770` |
| code candidate | `85cafced1cdd44f54f601aa9821f671b80159a67` |
| 규모 | 3 commits, 3 files, +221 / -21 |
| 작성 시점 상태 | `MERGEABLE`, `CLEAN` |

`mergeable`, `mergeStateStatus`, head SHA와 CI 결과는 작성 시점의 참고값이다. 최종 merge 직전에
trailing head의 GitHub Actions 상태와 mergeability를 다시 확인한다.

## 변경 범위와 판단

- 원 contributor 변경은 빈 본문 문단에서 완전히 겹친 비-TAC `Square` 그림 무리가 저장된 LineSeg의
  가로 시작점과 세로 위치를 만족하면 앵커 쪽에 남도록 한다. 이로써 #4770의 HPV 코호트 표본에서
  그림을 쪽당 한 장씩 분산하는 과대 페이지화를 막는다.
- collaborator 보정은 `rendering`과 `typeset`의 판정 조건을 하나의 구조 계약으로 통일했다. 빈 문단,
  비-TAC 그림, `Square`, `allow_overlap = false`, 공통 세로 앵커 대역을 모두 만족해야만 억제한다.
- `LineSeg::vertical_pos`는 페이지 절대 좌표이므로 그림 하단을 본문 높이가 아니라 절대 본문 하단과
  비교하도록 고쳤다.
- `TopAndBottom` 감싸기처럼 #1995의 낱장 분할 억제를 적용하면 안 되는 스택은 음성 회귀로 고정했다.
  셀 내부 그림 스택의 #2004 경로는 이 본문 전용 보정에서 제외된다.
- renderer와 typeset 경로가 바뀌므로 시각 검증 대상이다. 원 PR은 HPV 코호트 HWP, #1995 및 #2004
  표본의 페이지 수와 SVG 겹침 근거를 제시했다. 검토에서는 이 수치를 독립적으로 재산출하지 않았고,
  최신 GitHub Canvas visual diff 성공 및 새 구조 단정으로 무회귀를 확인했다.

## 완료된 검증

- `cargo fmt --check`를 통과했다.
- `CARGO_TARGET_DIR=target/pr-review cargo clippy --features native-skia -- -D warnings`를 통과했다.
- `CARGO_TARGET_DIR=target/pr-review cargo test --profile release-test --lib test_4770_anchor_pile_contract_requires_square_stack_and_page_bottom`를
  실행해 1건 통과, 실패 0건을 확인했다.
- `CARGO_TARGET_DIR=target/pr-review cargo test --profile release-test --tests`를 종료 코드 `0`으로
  완료했다. 라이브러리 단위 3,682건 통과, 실패 0건을 포함해 전체 integration test binary가 통과했다.
- code candidate의 GitHub Actions를 확인했다.
  - [CI](https://github.com/edwardkim/rhwp/actions/runs/31812950867): Lint, Native Skia, test archive,
    default-feature 3 shards, slow shard, Build & Test aggregate 성공
  - [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/31812950600): Rust 분석 성공
  - [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/31812950668): Canvas visual diff 성공
- frontend 및 독립 WASM Build job은 변경 영향 분류에 따라 `skipping`이며 실패가 아니다.

## 위험과 후속 범위

- 이 보정은 저장 LineSeg가 있는 본문 그림 스택의 구조 조건만 해석한다. LineSeg가 없거나 그림이
  `Square`가 아닌 문서는 기존 흐름을 유지한다.
- 원 PR이 제시한 HWP/Hancom PDF 페이지 수 근거는 contributor 측 실측이다. 이번 검토는 동일 표본의
  별도 MCP PDF 재생성을 수행하지 않았으며, 이는 원본 근거를 대체하지 않는다.
- 추가 결함은 발견하지 못했다.

## 최종 권고

merge를 권고한다. 이 review·오늘할일만 담은 trailing head의 preflight와 Build & Test aggregate가
성공하고, merge 직전에 최신 head SHA, `MERGEABLE`, `CLEAN`을 다시 확인한 뒤 작업지시자가 승인한
일반 squash merge를 수행한다.
