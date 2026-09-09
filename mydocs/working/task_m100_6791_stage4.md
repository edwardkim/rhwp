# Stage 4 — PR #6810 리뷰 보정과 충돌 해소

- Issue: [#6791](https://github.com/edwardkim/rhwp/issues/6791)
- PR: [#6810](https://github.com/edwardkim/rhwp/pull/6810)
- 승인: 2026-09-07 사용자 “보정과 충돌해소하고 보정 코멘트까지 게시해줘”.
- 검토 출처: [리뷰 11건](https://github.com/edwardkim/rhwp/pull/6810#issuecomment-5561356545)
- 문서 보정·실검증 SHA: `25d1d1011` (공개 두 파일). 후속 commit은 검증·운영 기록이다.

## 변경 결과

| 리뷰 | 반영과 판단 |
| --- | --- |
| 1 | PR 템플릿의 CONTRIBUTING·AGENTS 링크를 `blob/devel` 절대 URL로 바꿨다. GitHub 렌더링에서도 절대 링크임을 확인했다. |
| 2 | 최신 devel fetch 후 `git diff --check upstream/devel...HEAD`와 작업 트리 검사를 함께 수행한다. Rust 최종 확인은 준비 시 저장한 base SHA와 검증 SHA의 범위를 검사한다. |
| 3 | 네 회귀 블록을 포함한 Rust 검사 블록에 빈 값·미설정 변수 가드와 HEAD 일치 검사를 넣었다. |
| 4 | 공개 표에 로컬 검증 정본과 CI workflow·archive/reuse 계약의 진입점을 추가했다. |
| 5 | quickstart의 단순 `cargo build --locked`를 복원하고 전체 test·fmt 준비와 구분했다. |
| 6 | 증적 보존·clean 확인 뒤 작업 전용 worktree를 제거하는 명령을 추가했다. 강제 제거는 안내하지 않는다. |
| 7 | Studio 단독은 source, 혼합은 동일 SHA의 Rust review 루트로 실행 위치를 명시했다. |
| 8 | 편집 Command/Undo 체크리스트를 공개 안내·템플릿에 복원했다. E2E 미실행 사유·대체 증적과 PASS를 구분한다. |
| 9 | tests 전용 unit 경로는 WASM 없이 검사한다. package 경로는 fresh `--dev` WASM 후 TypeScript·unit·production bundle을 검사하며, renderer 표준 빌드를 대체하지 않는다. |
| 10 | 일회용 worktree 밖의 고정 절대 target을 모든 Rust `--target-dir`에 적용했다. 환경변수만 바꿔서는 명시 target이 바뀌지 않는 점도 설명했다. |
| 11 | collaborator self 정본이 요구하는 review·오늘할일을 유지했다. 외부 기여자의 금지 규칙과 승인된 collaborator 역할 예외를 공개 문서에 명시했다. |

`732525706`에 승인·계획을 기록한 뒤 최신 devel `07bc5e5490f75118f08370de19aeee73ce1667cb`를
`c90b83879`로 병합했다. 충돌 경로는 `mydocs/orders/20260906.md` 하나였으며 기존 #6799·#6813 기록과
#6791 행을 모두 보존했다. `git show --remerge-diff`에서 수동 해소는 이 파일에 한정됐다.
공개 문서 보정은 `25d1d1011`이며 제품 코드·CI·Cargo·fixture 보정은 추가하지 않았다.

## 실제 검증

원시 로그와 추출한 공개 명령은 `/private/tmp/rhwp-6810-correction-evidence/`에 보존했다.
source는 `/private/tmp/rhwp-6810-corrected-source`의 clean detached worktree를 사용했다.

- 문서의 준비 블록을 그대로 실행해 별도 `-rust-review` worktree를 만들고 prepare → fmt check →
  manifest·commit 범위·SHA·clean 검사를 통과했다. 1,178 sources / 28 suites + 20 exceptions = 48/48 targets다.
- 검증 전후 tracked Rust·Cargo 2,215개 hash가 동일했다. source에는 suite를 생성하지 않았고
  review의 28 harness·manifest는 ignored 상태였다. 공개 정리 블록으로 해당 review worktree만 제거했다.
- Bash·Zsh에서 검사 블록 8개 각각의 변수 미설정·빈 문자열·잘못된 SHA를 넣은 48개 음성 검사가
  모두 중단됐다. Rust/frontend bash 블록 18개는 두 셸의 구문 검사를 통과했다.
- WASM 없는 source에서 공개 frontend 설치·unit 블록을 실행해 TypeScript 및 단위 테스트
  1,489 passed / 1 skipped / 0 failed를 확인했다.
- 같은 source에서 `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --dev`를
  실행해 성공했다(약 51초). 이어 공개 package 블록의 TypeScript·unit·production build를 모두 통과했다.
  dev WASM과 production Studio bundle을 구분하며 release WASM·브라우저 E2E 통과로 주장하지 않는다.
- 현행 classifier의 tests-only, source, Rust 혼합, E2E 네 입력이 문서의 범위 구분과 양립함을 확인했다.
  tests-only는 frontend unit / Rust false, source·E2E는 package, 혼합은 Rust true였다.
- 템플릿을 GitHub Markdown API로 렌더링한 결과 링크 8개가 모두 절대 URL이었다. 상대 경로 검사와
  내부·교차 anchor 26개, commit 범위·작업 트리 공백 검사를 통과했다.
- 앞선 최소 Cargo 실험에서는 누락 test 경로가 있어도 build가 성공하고 fmt는 실패했으며,
  `--target-dir`이 환경변수보다 우선했다. 커밋된 공백·충돌 마커는 작업 트리 diff가 놓치고 범위 diff가 검출했다.

검증 보조 스크립트의 최초 블록/링크 개수 가정과 classifier 호출 형식·문자열 boolean 판정은 정정 후
재실행했다. 이는 공개 명령 실패가 아니며 최종 결과는 수정한 검증 도구의 결과다.
제품 source·Rust test를 고치지 않았으므로 세 Clippy·전체 nextest·Native Skia는 반복하지 않았다.
Windows native wrapper, Docker 최적화 WASM, 실제 브라우저 E2E는 이번 문서 보정 검증에서 실행하지 않았다.

## 제출과 남은 조건

검증 결과를 self-review·보고서와 같은 PR에 push하고, 리뷰 번호별 반영·대안·검증 범위를 보정 댓글로
게시한다. 댓글은 UTF-8 파일로 보내고 API로 본문 일치를 확인한다. 최신 head CI·mergeability는 게시 시
별도로 조회한다. 이전 head의 CI 통과를 새 공개 문서 head에 재사용한 것으로 주장하지 않는다.
merge·#6791 close는 이번 승인 범위가 아니며, 최신 checks 통과와 별도 merge 승인이 남는다.
