# PR #6881 검토 기록

## 판정: 메인터너 보정 후 수용 가능

**메인터너 보정 완료.** 아래 코드 head에서 실제 결함 3건의 보정과 CI 성공을 확인했다.
이 판정은 P44의 Native exact glyph replay 및 atomic fallback 범위에 한정한다.
이 문서를 추가하는 trailing commit의 최신 CI는 별도로 확인해야 하며, 병합 완료를 의미하지 않는다.

## 검토 대상과 출처

- 원 PR: [#6881](https://github.com/edwardkim/rhwp/pull/6881)
- 기여자: `seo-rii`, 원격 source branch: `seo-rii/rhwp:render-p44`, 대상: `edwardkim/rhwp:devel`.
- 원 기여 head: `95d7081dca1f2c2500b365b80a14990321bb37ff`.
- 최종 코드 head 및 메인터너 보정: [a2b83a4eedb954250f962f8cfacae38dab15e078](https://github.com/edwardkim/rhwp/pull/6881/commits/a2b83a4eedb954250f962f8cfacae38dab15e078).
- 검토일: 2026-09-08. 로컬 검토에서 원 커밋 9개를 출처 보존 체리픽했지만, 원격 push는 원 기여 head 위의 보정 커밋 하나만 수행했다. 통합 이력이나 devel 변경을 fork에 밀어 넣지 않았다.
- 로컬 `devel`은 `upstream/devel`의 `54f4a0237e5aa8c9270dd767d5d9a21b9df56bd8`까지 fast-forward했다. 원 PR의 커밋을 rebase하거나 force push하지 않았다.
- 추적 이슈 [#536](https://github.com/edwardkim/rhwp/issues/536)은 이 PR의 종료 대상으로 간주하지 않는다. 후속 P45~P47이나 전체 렌더링 동등성까지 수용한 것으로 확대하지 않는다.

## 코드 검토 및 메인터너 보정

### 1. 글리프 준비 예산의 적용 시점

- 원 구현은 준비 결과를 보관할 때 예산을 검사하여, 이미 예산을 초과했어도 글꼴 파싱·복사 등 준비 비용을 반복 지불할 수 있었다. 실제 결함으로 판정했다.
- 보정은 leaf별 entry/byte 예산을 준비 전에 검사·예약한다. font blob과 알려진 배치 버퍼의 최소 비용을 반영하며 준비 실패 시 예약분을 반환하지 않는다.
- 준비 후 계산되는 outline 추가 비용이 남은 예산을 초과하면 이후 준비를 중단한다. 준비 실패 후보의 atomic TextRun fallback 경로는 유지한다.
- 모든 입력에 대한 CPU 시간의 엄격한 상한이나 부하 개선 수치는 측정하지 않았다. 봇의 표현을 근거로 무한 작업량 또는 특정 메모리 사용량이 실측됐다고 기록하지 않는다.
- [보정 결과 답글 및 resolve](https://github.com/edwardkim/rhwp/pull/6881#discussion_r3957654458).

### 2. 중첩 SVG viewport 보존

- 원 구현은 중첩 요소까지 모든 `svg`를 `g`로 바꾸어 x/y/width/height/viewBox의 viewport 의미를 잃었다. 실제 결함으로 판정했다.
- XML 깊이를 추적하여 payload viewBox로 대체하는 최상위 컨테이너만 변환하고, 중첩 SVG는 그대로 유지하도록 보정했다. 불일치하는 요소 깊이는 준비 실패로 처리한다.
- Chrome 축소 재현에서 중첩 viewport를 유지한 사각형 경계는 x=40, y=30, width=20, height=20이었고, 모든 SVG를 g로 바꾸면 x=0, y=0, width=10, height=10이었다.
- 이 실험은 원인 확인이다. 수정 후 Native Skia 출력의 픽셀 비교나 실제 문서 전체의 시각 동등성을 검증한 것으로 해석하지 않는다.
- [보정 결과 답글 및 resolve](https://github.com/edwardkim/rhwp/pull/6881#discussion_r3957654752).

### 3. 공통 선택기와 Native 글꼴 크기 상한

- 공통 선택기는 4096px 초과를 거부하지만 실제 Native 준비 경로에는 같은 상한이 없어 진단과 렌더 선택이 달라질 수 있었다. 실제 결함으로 판정했다.
- `MAX_GLYPH_FONT_SIZE_PX = 4096.0`을 공통 상수로 두고 선택기 및 Native 준비가 함께 사용하도록 보정했다.
- Native 경로의 `font_instance.size_px`와 `paint_style.font_size` 모두 상한을 적용한다. 기존 finite/양수 조건은 유지한다.
- [보정 결과 답글 및 resolve](https://github.com/edwardkim/rhwp/pull/6881#discussion_r3957655096).

세 스레드 모두 오탐이 아니라 **실제 결함 보정 완료**로 답글을 남기고 resolve했다. API로 게시 본문과 resolve 결과를 확인했다.

## 로컬 검증

검토 전용 `CARGO_TARGET_DIR=target/pr6881-maintainer`를 사용했다. 기존 공유 target은 삭제하지 않았다.

| 검사 | 실제 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 최종 보정 후 통과 |
| `cargo clippy --locked -- -D warnings` | 보정 후보에서 통과 |
| `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` | 보정 후보에서 통과 |
| `cargo build --locked --workspace` | 보정 후보에서 통과 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | 보정 후보에서 통과 |
| `cargo clippy --locked --lib --features native-skia -- -D warnings` | 문자열 비교 타입 오류를 수정한 최종 보정에서 통과 |
| `node scripts/rust-test-suite-manifest.mjs --check` | 최종 보정 후 통과 |
| `git diff --check` | 최종 보정 후 통과 |

중간 Native Skia 컴파일 실패는 새 SVG 이름 비교에서 `str`을 byte string과 비교한 메인터너 수정 오류였다. `"svg"` 비교로 바로잡은 뒤 재검증을 통과하고 push했다.
이번 로컬 작업에서 전체 회귀 테스트를 다시 실행하거나 세 지적 각각의 신규 회귀 테스트를 추가하지 않았다. 아래 원격 CI 성공과 로컬 실행 범위를 구분한다.

## 코드 head의 실제 CI 결과

다음 결과는 모두 `a2b83a4eedb954250f962f8cfacae38dab15e078` 기준이다. 원 기여 head의 과거 성공을 재사용해 성공으로 기록한 것이 아니다.

| 워크플로/검사 | 결과와 근거 |
| --- | --- |
| CI | [34223039334](https://github.com/edwardkim/rhwp/actions/runs/34223039334) 성공. Build & Test, Lint, Native Skia, archive A~D 빌드 및 실행 worker 모두 성공 |
| Native Skia tests | [실행 job](https://github.com/edwardkim/rhwp/actions/runs/34223039334/job/102050540270) 성공 |
| CodeQL | [34223039380](https://github.com/edwardkim/rhwp/actions/runs/34223039380) 성공. Rust/JavaScript-TypeScript/Python 분석 worker 모두 성공 |
| 별도 CodeQL check | [check](https://github.com/edwardkim/rhwp/runs/102054863011)는 `NEUTRAL`. 워크플로/언어별 worker 성공과 구분 |
| Render Diff | [34223039020](https://github.com/edwardkim/rhwp/actions/runs/34223039020) 및 Canvas visual diff 성공 |
| Adapter inter-diff | [34223039319](https://github.com/edwardkim/rhwp/actions/runs/34223039319) preflight/worker 성공 |
| Proptest | [34223039116](https://github.com/edwardkim/rhwp/actions/runs/34223039116) preflight/worker 성공 |
| CI Impact Policy | 최종 status `SUCCESS` |

`gh pr checks --required`의 Build & Test는 `SUCCESS`였다. 독립 WASM Build, Frontend gates, Workflow promotion, PR의 duration refresh는 `SKIPPED`이며 실행된 검사로 세지 않는다. 실제 WASM check는 성공한 Lint job과 로컬 WASM Clippy 결과를 근거로 삼는다.
코드 head의 최종 상태는 `MERGEABLE / CLEAN`, 실패·취소·pending 없음이었다.

## 시각 증적과 한계

- 기존 CI Native Skia 및 Render Diff job을 증거로 연결한다. 개별 봇 지적을 새 테스트가 직접 검출한다고 주장하지 않는다.
- 이번 보정에서는 새 한컴 PDF·Native 픽셀 비교 PNG를 생성하지 않았다. Chrome 축소 재현 결과를 실제 HWP/HWPX 문서의 Native 렌더 결과로 대체하지 않는다.
- 임시 로그, SVG, JSON, 중간 PNG는 커밋하지 않는다. 이 trailing commit은 검토 문서와 오늘할일만 포함한다.
- 4096 경계의 신규 Native 실행 테스트, 대형 글꼴 반복 입력의 부하 측정, 모든 언어·RTL·세로쓰기의 시각 동등성은 검증 완료 범위가 아니다.

## 후속 처리 및 코멘트 계획

- 보정 결과는 위 원 리뷰 스레드 3곳에 이미 한 번씩 기록했다. 동일 보정 내용을 중복 댓글로 게시하지 않는다.
- 원 PR의 문서-only trailing head를 push한 뒤 그 최신 head의 checks를 확인한다. 코드 head의 CI 성공을 아직 실행되지 않은 문서 head의 성공으로 기록하지 않는다.
- 병합 승인 시 `post_merge.md`에 따라 merge SHA 및 실제 PR/devel CI를 확인하고 원 PR의 기존 후속 기록과 중복되지 않게 남긴다. 새 시각 자료가 없으므로 존재하지 않는 이미지나 첨부를 만들어 근거처럼 제시하지 않는다.
- #536 종료, 기여자 fork branch 삭제, 별도 통합 PR 생성, owner reviewer 자동 지정은 이번 작업에 포함하지 않는다.
