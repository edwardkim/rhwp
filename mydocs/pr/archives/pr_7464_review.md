---
kind: snapshot
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-08
---

# PR #7464 검토 — 셀 블록 지우기와 삭제 후 커서

## 최종 판정

**메인테이너 보정 후 수용 가능.** 원 head의 두 차단 문제를 통합 branch에서 보정했습니다. GitHub CI와 최종 merge 조건은 제출 후 별도로 확인합니다.

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자·base | [#7464](https://github.com/edwardkim/rhwp/pull/7464) / lidge-jun (jun) / devel |
| 원 head | `c8b57d4aa15314b620cae4827541b682c2625586` |
| 관련 이슈 | [#7462](https://github.com/edwardkim/rhwp/issues/7462), 통합 PR 반영 후 종료 |
| 기존 차단 review | [2026-09 review](https://github.com/edwardkim/rhwp/pull/7464#pullrequestreview-5339588530) |
| 작성 시 원 PR 상태 | OPEN / head 변동 없음 / 조직 fork source 보존 |

## 보정과 소비 경로

원 기여는 F5 셀 블록의 Backspace/Delete/Ctrl+E 내용 삭제와 Meta+E/Meta+Backspace/Meta+Delete 구조 삭제 대화상자 및 Undo 복원을 구현했습니다. 다음 두 문제를 보정했습니다.

1. 대화상자 capture keydown이 남김·취소·닫기의 Enter를 primary 삭제로 바꾸던 문제: 실제 버튼의 Enter/Space는 propagation만 차단하고 기본 활성화를 허용합니다. 버튼 외부 Enter fallback은 유지합니다.
2. 마지막 행/열 삭제 뒤 flat cellIndex만 바뀌고 depth-1 cellPath와 문단 좌표가 남던 문제: 기존 clamp가 남은 셀의 flat index, cellParaIndex/paragraphIndex=0, 복사한 leaf path를 함께 반환합니다. 일반 표 명령과 셀 블록 삭제 모두 소비하며 이전 path를 수정하지 않아 Undo 기록도 보존합니다.

IME physical KeyE 라우팅, 부분 셀 내용 삭제, 전체 표 삭제 및 본문 보존은 원 기여 경로로 유지합니다. 중첩 표 전반의 구조 삭제 지원을 새로 주장하지 않습니다.

## 실제 검증

- Studio 전체 단위 검사: 1,825 PASS / 0 FAIL / 기존 2 SKIP (총 1,827), focused 32 PASS. helper 검사에는 병합 셀 범위, flat/path/문단 일치, 원 history path 불변성과 표 소멸을 포함했습니다.
- 새 최적화 WASM + 실제 Chromium E2E: 100 PASS / 0 FAIL, pageerror 0. Enter와 Space 각각 지우기/남김/취소/닫기의 결과를 확인했습니다. C3의 두 번째 문단에서 시작해 마지막 행·열 삭제 후 rect, Undo 내용·블록 복원, Redo와 ArrowRight 뒤 실제 입력을 확인했습니다.
- 수정 전 격리 Studio: Enter 남김/취소/닫기의 구조 삭제 6단언 FAIL. 대화상자만 보정하고 기존 cursor helper를 유지한 대조에서는 마지막 행·열 삭제/Redo/입력 좌표 FAIL 및 셀 8 범위 초과 pageerror 2건. direct helper도 1 FAIL로 부족한 path/문단 반환을 검출했습니다. 환경 준비 실패와 실제 회귀 실패를 구분했습니다.
- TypeScript `npx tsc --noEmit`, `npm run build`, E2E manifest 150/150, JS syntax 및 diff check 통과. 초기 stale pkg 선언 오류는 새 WASM 생성 후 해소했으며 무관한 Studio API 코드는 바꾸지 않았습니다.
- 조판 변경의 독립 Print 전쪽 증적은 [#7468 검토](pr_7468_review.md)와 [시각 asset](../assets/pr_7468/README.md)에 있습니다.

## 통합 경로와 검증 대상

- 경로: `collaborator_external_pr` 9.1.1의 최신 devel 기반 cherry-pick 통합. 사용자께서 두 PR의 보정·통합 PR 생성·병합을 요청하셨습니다.
- 기준 devel: `f0e7228f6dd2ea1437724e53ad640c40c56d204b`; branch `review/lidge-pr7464-pr7468-20261008`, 기본 작업공간 `/workspace/rhwp`.
- 실행 계정 `postmelee`는 push 권한이 있으나 admin/maintain 권한은 없습니다. 원 `lidge-ai/rhwp` 조직 fork에는 push하지 않습니다. 원 commit의 author와 `cherry picked from` 계보를 보존하고 merge commit은 제외했습니다.
- 메인테이너 코드 보정 `282d5d77f6f7167a70047170dbad1a10bcce6ba5`, 회귀·독립 Print 입력 보강 `fdea8a4cbe203f6ec93bbbaf47382ebddacad6c1`. 최종 문서 commit은 생산 코드·검사·입력을 바꾸지 않습니다.
- `formatting.rs` import 충돌은 최신 `restamp_indentation`과 새 flow helper를 함께 보존했습니다. 최신 batch deferred rebuild 및 stored-reset guard를 유지했습니다. 원 PR의 전역 synthetic-tail 제외 변경은 최신 upstream에서 불필요하여 통합 결과에서 제외했습니다. 최종 Rust 생산 변경은 두 서식 API에 한정됩니다.
- 공용 Cargo target은 `/workspace/rhwp/target/pr-review` 하나입니다. Cargo 실행은 순차로 진행하고 파생 suite/manifest, 로그·중간 JSON·TSV는 커밋하지 않습니다.

## 원 기여 계보

| 통합 commit | 원 commit | 내용 |
| --- | --- | --- |
| `41c87755f579a1173b0f85c46a6263394380f0d4` | `d334a1f8a7404ecc569beb23625a4a84fa614182` | 수정: 셀 블록 지우기 키와 삭제 확인 동작 정합 |
| `c6be116b83805610b044cf668e27c3a95d567c35` | `8474698c1d89f8eec3781df882b968d197d7e5ed` | 수정: 셀 블록 지우기가 표 명령 모듈을 불러오지 않게 커서 보정 헬퍼 분리 |
| `c1261e6a959c64dad2f36e67c54ec3a65a1074d0` | `3f1dbff08b9f4cfc2407fb903253594db9d449f6` | 테스트: 빈 문단 뒤 글자 크기 변경 쪽나눔 회귀 재현 |
| `606fc5f4ccd158b9483f0f7357f2f94282e91b97` | `72a47a1c864c72c47e5430f9834853b29d9447bb` | 수정: 글자 크기 변경 뒤 합성 원점의 가짜 쪽나눔 방지 |
| `8cee9363a215587723067f6353e6e495dfe40017` | `1875b509167664a54f6d02e6ea62df827cab38bf` | 정리: 글자 모양 회귀 테스트 서식 맞춤 |
| `8f318e4a45d1b22998e42d5f134dacdc98b285c8` | `68c1e7f367464f25ba123eb345d9e4fea36fdcc5` | 테스트: 빈 문단 삭제 후 저장 재열기까지 확인 |
| `91fd3c29d8265beef6a19ff2e90f81d5b49ed540` | `20b388d9c14f8342986fd4277817eb4e843a7822` | 정리: 삭제 후 재열기 회귀 검사 서식 맞춤 |

위 기능 commit의 저자는 모두 jun입니다. 원 PR 작성자는 lidge-jun이며 기능 기여를 메인테이너 보정과 구분하여 보존합니다.

## Merge 전 조건과 후속 처리

최종 head GitHub CI와 required checks, 최신 devel의 merge-tree 및 head SHA를 확인한 후 일반 merge를 진행합니다. branch protection을 우회하지 않습니다. 통합 PR이 실제 devel에 반영된 뒤에만 원 #7464·#7468을 대체 안내와 함께 close(merged=false)하고 관련 #7462·#7467을 확인·종료합니다. #7445는 OPEN으로 유지합니다.

merge 후 comment는 한국어 존댓말의 `--body-file`로 원 기여, 보정 이유, 실제 로컬/CI 결과, 통합 PR·merge SHA와 SHA 고정 시각 asset을 안내하고 API로 게시 내용을 재확인합니다. 이 archive와 오늘할일은 같은 통합 PR에 포함하며 별도 기록 PR을 만들지 않습니다. 최종 devel sync, 이 작업에서만 만든 clean 임시 branch와 실행 서버 정리, duration refresh 확인까지 수행합니다. contributor fork branch와 공유 target은 보존합니다.


## Merge 후 contributor PR comment 계획

실제 통합 PR의 일반 merge와 devel 반영을 확인한 뒤 아래 내용으로 한국어 존댓말 comment를 게시합니다. 통합 PR 번호·최종 CI run·merge SHA는 실제 완료값으로만 채웁니다. 게시 전 이 기록과 devel의 asset을 대조하고 게시 후 API에서 UTF-8 본문을 다시 확인합니다.

- jun/lidge-jun님의 셀 블록 키 동작·내용/구조 삭제 구분·Undo 기여에 감사하고 author/원 SHA를 보존한 통합 PR을 안내합니다.
- 남김/취소/닫기의 Enter가 삭제를 실행하던 문제와 마지막 행/열 삭제 후 path/문단 좌표가 남던 문제를 메인테이너가 보정했음을 설명합니다.
- 실제 focused 버튼 Enter/Space와 삭제·Undo·Redo·이동·입력의 fresh WASM browser E2E 100 PASS, Studio 전체 1,825 PASS / 기존 2 SKIP, 최종 Rust/Native Skia/lint·GitHub CI의 실제 결과를 요약합니다.
- 같은 통합 PR의 서식 보정 시각 증적을 함께 안내합니다. edited p1의 Native/fresh WASM review 및 standalone overlay를 같은 입력·쪽으로 표시하고 함초롬바탕 regular/bold를 실제 공급한 최종 전 4쪽은 각각 100.00000%임을 적습니다.
- [Visual Sweep 비교 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 링크하고 관용 실루엣과 엄격한 Linux glyph/raster 잔차를 구분합니다. 정상 쪽 경계 대조군은 통과했으며 sample16의 #7445는 별도 OPEN임을 안내합니다.
- 원 PR은 조직 fork를 수정하지 않고 통합 PR로 대체되어 close(merged=false)한 사실과 관련 #7462 종료를 확인해 적습니다.

## 최종 글꼴 비교

사용자가 제공한 함초롬바탕 regular/bold를 적용한 Native/fresh WASM 전 4쪽은 100.00000%로 통과했습니다. 모든 16개 review/overlay를 직접 판독했습니다. 엄격한 내용 픽셀 일치 18.45444–38.23178%의 획/raster 잔차와 source·font 출처·명령은 [#7468 최종 비교](pr_7468_review.md#사용자-제공-함초롬바탕-적용-최종-비교)에 있습니다.

## 최종 로컬 검사 결과

| 검사 | 결과 | 실측 근거 |
| --- | --- | --- |
| Rust suite manifest prepare | PASS | exit 0 |
| rustfmt all | PASS | exit 0 |
| rustfmt check | PASS | exit 0 |
| 빈 문단 뒤 서식 focused | PASS | Summary [   0.279s] 3 tests run: 3 passed, 225 skipped |
| release-test 전체 nextest | PASS | Summary [4717.635s] 10538 tests run: 10538 passed (43 slow), 50 skipped |
| Native Skia lib | PASS | test result: ok. 3927 passed; 0 failed; 13 ignored; 0 measured; 0 filtered out; finished in 275.12s; test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s; test result: ok. 165 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s |
| Native Skia 그림 placeholder | PASS | Summary [   3.785s] 2 tests run: 2 passed, 227 skipped |
| Native Skia 직접 PDF | PASS | Summary [   0.319s] 4 tests run: 4 passed, 221 skipped |
| Native Clippy (-D warnings) | PASS | exit 0 |
| WASM lib Clippy (-D warnings) | PASS | exit 0 |
| workspace build | PASS | exit 0 |
| workspace all-targets Clippy (-D warnings) | PASS | exit 0 |
| Rust suite manifest fixed-base check | PASS | exit 0 |

생산 코드·검사 candidate는 `82aeff6091b6827b4f913d4483ba0a354f06f5ca`입니다. Rust 1.93.1, `--locked`, 단일 `/workspace/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=5`, `CARGO_PROFILE_DEV_DEBUG=0`을 사용했습니다. release-test 최적화·nextest test threads는 기존 설정을 유지했습니다. Cargo 명령은 순차로 실행했습니다. Native Skia의 lib 및 두 focused target을 한 Cargo no-run으로 준비하던 중 host의 FreeType/fontconfig 개발용 linker 이름이 없어 regular lib 링크가 실패했습니다. 설치된 실제 runtime DSOs(FreeType .so.6 / fontconfig .so.1)에 대한 두 unversioned alias를 기존 Skia linker 검색 경로 아래에만 제공하고 실제 link/runtime smoke를 통과했습니다. system package·source·RUSTFLAGS를 바꾸지 않았고 공유 target을 삭제하지 않았습니다. 환경 보완 뒤 세 정식 Native Skia 명령의 최종 결과를 위 표에 기록했습니다. 실패한 no-run을 PASS로 세지 않았습니다. 원본 runtime/alias 경로와 SHA-256은 ignored native-system-library-aliases.json에 보존합니다.

초기 jobs=3의 전체 회귀는 compile 단계에서 중단하고 같은 target의 캐시를 보존하여 jobs=5로 완료했습니다. 중단된 compile과 오래된 WASM 실행을 PASS로 세지 않았습니다. 전체 nextest의 실제 compile/test 시간은 그 로그의 Finished/Summary 값이며, coordinator 대기시간을 포함한 외부 wall time과 구분합니다.

최종 체크까지 local candidate의 source·test·fixture는 바뀌지 않았습니다. 초기 시각 빌드는 production `282d5d77f…`입니다. 사용자 제공 글꼴 최종 비교의 Native는 candidate `82aeff609…` binary, WASM은 production `282d5d77f…`의 fresh 최적화 빌드를 실제 재실행한 결과입니다. 사이에는 검사·입력 보강 및 import 줄바꿈만 추가되어 동작은 같으며 출처를 구분했습니다. 문서·이미지 추가 commit을 새 코드 실행으로 표시하지 않습니다.

검증 중 devel은 `74f9b71b15a2db18b17c37ceab6cf2b3c20c03c3`으로 전진했습니다. 새 upstream 표 조판 변경과 이번 생산 변경 파일은 겹치지 않고 코드 merge-tree는 clean입니다. 이번 branch의 code를 억지 merge/rebase하지 않고 최신 base와 최종 문서 head의 merge simulation 및 GitHub current-base merge CI를 독립적으로 확인합니다. 새 upstream 변경을 이번 PR 기능 diff로 집계하지 않습니다.

자체 작성 통합 PR의 author self-review입니다. 원 번호의 archive 네 개를 같은 PR에 보존하며 별도 integration 번호 archive나 기록 PR을 만들지 않습니다. 이 절은 로컬 검사 완료 시점의 기록입니다. 이후 완료한 GitHub CI는 다음 절에 기록하며 실제 merge는 반영 확인 뒤 후속 comment에서 확정합니다.

전체 nextest 안의 svg_snapshot 기준 출력 6개와 프로세스 내 결정성 검사 1개가 모두 PASS했습니다. golden 실패 또는 변경이 없으므로 별도 재생성은 수행하지 않았습니다. Native Skia lib의 workspace 합계는 4,109 PASS / 0 FAIL / 기존 13 ignored입니다.

## 통합 PR 및 GitHub Full CI

[통합 PR #7686](https://github.com/edwardkim/rhwp/pull/7686)은 원 번호별 검토 archive·오늘할일·시각 asset을 같은 branch에 포함합니다.

Full CI candidate `cdaa64680f6faa189aa08d9222216ae9a734d9d9`, 실행 base `74f9b71b15a2db18b17c37ceab6cf2b3c20c03c3`, GitHub 자동 merge ref `8047e559b540d50a863a470d324edac4c7fd7170`, tree `a23a61b252876f056fe31d5a26c9b83a607ae636`입니다. PR event의 실제 base/head와 GitHub merge ref의 부모/tree를 API·Git으로 확인했고 로컬 merge simulation의 tree와 같았습니다. 네 builder/worker의 기본 PR merge checkout 정의 및 성공 step도 확인했습니다.

| 검증 | 실제 run | 결과 |
| --- | --- | --- |
| CI | [37803919567](https://github.com/edwardkim/rhwp/actions/runs/37803919567) / attempt 1 | SUCCESS |
| CodeQL | [37803919737](https://github.com/edwardkim/rhwp/actions/runs/37803919737) / attempt 1 | SUCCESS |
| Render Diff | [37803918893](https://github.com/edwardkim/rhwp/actions/runs/37803918893) / attempt 1 | SUCCESS |
| Adapter inter-diff | [37803919449](https://github.com/edwardkim/rhwp/actions/runs/37803919449) / attempt 1 | SUCCESS |
| Proptest roundtrip | [37803919540](https://github.com/edwardkim/rhwp/actions/runs/37803919540) / attempt 1 | SUCCESS |

CI의 네 default-feature archive builder와 네 Run Archive A/B/C/D step이 실제 실행되어 모두 SUCCESS였습니다. 필수 `Build & Test`의 `Verify archive shard totals`와 worker 결과 집계도 SUCCESS입니다. lint·Native Skia 실제 test step·Frontend package gates를 확인했습니다. CodeQL JavaScript/TypeScript·Python·Rust·Actions의 Perform CodeQL Analysis step 네 개가 모두 SUCCESS이고, 별도 GitHub 보안 집계도 SUCCESS(변경 코드에서 새 경고 없음)였습니다.

WASM Build, release operations, Workflow promotion 및 Frontend unit gate는 정책상 SKIPPED입니다. 이를 새 실행으로 세지 않았습니다. fresh WASM 및 Studio 전체 unit은 위 로컬 실측 결과를 사용합니다. GitHub raw 로그 다운로드는 실행 환경의 log backend 접근에서 HTTP 403이 발생했습니다. 따라서 CI의 개별 테스트 숫자나 raw checkout log 줄을 읽었다고 주장하지 않습니다. 완료 Run/Job/Step API와 count 검증 step의 성공, 실제 source/base 및 merge-ref tree로 근거를 보존합니다. 로컬 10,538 PASS는 실제 읽은 nextest Summary입니다.

이 결과 뒤에는 원 번호의 검토 기록·오늘할일만 single-parent trailing commit으로 추가합니다. source/test/fixture/PNG는 변경하지 않습니다. 최종 trailing head의 preflight·required aggregate와 최신 devel의 clean merge-tree를 다시 확인한 뒤 사용자 요청 범위의 일반 merge를 진행합니다. 실제 merge SHA·issue/original PR 종료는 반영 확인 뒤 후속 comment에서 확정합니다. author self-review 예외를 적용하며 별도 integration 번호 archive나 기록 PR은 만들지 않습니다.
