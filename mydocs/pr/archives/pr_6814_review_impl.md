# PR #6814 리뷰 보정 계획

- 단계: Hyper-Waterfall 후속 보정 — 계획 승인 후 구현·검증·게시 순서.
- 승인: 2026-09-07 사용자 “권장 순서대로 진행해주고, 보정 코멘트도 게시해줘.”
- 대상: [리뷰 5126268574](https://github.com/edwardkim/rhwp/pull/6814#pullrequestreview-5126268574).
- 기준 head: `bff48959d77528f6ee02c2744bd1779726440b77`.
- merge·이슈 종료는 이번 승인에 포함하지 않는다.

## 구현 순서와 계약

1. 리뷰 2·3: 문서 미열림과 구버전 binding 오류를 구분하고 UI에서 안내한다.
   적용 실패 시 모든 문단의 rollback을 시도하고 원래 오류와 복원 오류를 함께 보존한다.
   복원이 불완전하면 command를 Undo에 보존하여 재시도할 수 있게 한다.
   Undo 실패도 해당 엔트리를 유지하며, 부분 Redo 실패는 Undo 쪽으로 옮긴다.
   다른 command의 기존 실패 처리 계약은 바꾸지 않는다.
2. 리뷰 4: 첫 ref가 0보다 뒤에서 시작해도 기존 조회와 동일하게 첫 ID가 앞 구간을
   상속하도록 적용·조회·복원을 일치시킨다. 선택 밖 의미가 변하지 않아야 한다.
3. 리뷰 6: Node WASM 산출물 없이도 실제 command/history 실패 경로를 검사하는
   필수 unit test를 추가한다. 선택적인 실제 WASM 검증과 구분한다.
4. 리뷰 1·5·7~12: 범위 판정 공유, 명시적인 lookup 오류, 검증 오류 구체화,
   중복 직렬화·경로 계산·경계 ref 정리를 관련 수정 범위 안에서 진행한다.
5. 실제 결과로 self-review와 작업 기록을 보완하고 보정 코멘트를 게시한다.

## 검증과 롤백

- 새 회귀는 문서 없음/낡은 binding, 다중 문단 apply 및 rollback 복합 실패,
  Undo 재시도, 부분 Redo 실패, 첫 ref prefix와 선택 밖 서식 보존을 포함한다.
- Studio 필수 unit test와 빌드, fresh WASM 테스트 및 변경 범위의 Rust 회귀를 실행한다.
- Rust push 전 필수 lint 3종·workspace build·fmt·파생 suite 검사를 순차 수행한다.
  현재 디스크 여유가 부족하므로 실행 불가 시 이를 성공으로 기록하거나 push하지 않는다.
- 기존 브라우저 스크린샷은 기존 구현의 증적으로 유지하며 새 오류 경로 검증으로 간주하지 않는다.
- 보정 문제가 발견되면 이번 보정 커밋만 역패치한다. 기존 구현·사용자 파일·다른 worktree는
  되돌리거나 삭제하지 않는다.

## 정정할 리뷰 해석

- HashMap 누락 panic의 현재 도달 경로는 확인되지 않았다. 조용한 원본 ID fallback 대신
  불변식 위반을 명시적으로 드러내는 방향으로 보강한다.
- 복원 중 예외가 나면 기존 코드는 원래 적용 오류가 아니라 복원 오류를 던진다.
- 기존 CI 성공에는 실제 WASM Studio 테스트의 skip이 포함됐다. 로컬 실제 WASM 성공과
  CI에서 항상 실행되는 회귀의 범위를 구분한다.

## 최초 보정 및 공간 확보 전 검증 기록 (2026-09-07)

- 계획 commit: `e7380e47a`. 구현 candidate: `2f6f46376`.
- 1~12번 관련 보정 구현: fallible ID lookup, 겹침 iterator 공유, 첫 ref 상속,
  tail 경계 원본 보존, 오류 구체화, UI 안내, 실패 복구 history 보존, 직렬화·경로 캐시.
- 불완전 rollback은 원래 오류와 모든 복원 오류를 AggregateError에 남긴다.
  복구 command 위의 후속 편집은 기존 순서대로 Undo한 뒤 복구를 다시 시도할 수 있다.
  이 계약은 opt-in이며 다른 command의 기존 실패 시 제거 동작은 유지한다.
- 새 필수 테스트는 실제 command/history/bridge/InputHandler를 읽되 WASM import를 stub으로
  대체한다. 본문·중첩 셀 복합 실패, Undo 재시도, 부분 Redo, 빈 선택, 기존 Redo 보존,
  미열림·구버전 binding 안내, 미관련 오류 재전파를 검사하며 pkg/pkg-node가 필요 없다.
- `npm test`: **1428 passed, 0 failed, 0 skipped**. 최초 실행에서는 새 오류 cause 계약에
  맞지 않은 기존 assertion과 CommonJS 테스트의 alias import가 실패했으며 보정 후 재실행했다.
- `npm run build`: TypeScript + Vite 통과. 기존 pkg의 WASM을 사용한 frontend 빌드이며
  새 Rust로 만든 WASM 빌드라는 주장은 아니다.
- `node --test rhwp-studio/tests/mixed-char-format-recovery.test.ts`: 추가 edge case 후 재통과.
- `cargo fmt --all` / `--check`, 별도 integration source rustfmt, `git diff --check`: 통과.
- `cargo check --locked --offline --lib --target-dir target/pr-review`: 통과(19.27초).
  다른 review worktree의 읽기 전용 cache를 APFS clone으로 별도 target에 복사했으며 원본은
  변경하지 않았다. 이 검사는 필수 lint 묶음이나 Rust 행위 회귀를 대체하지 않는다.

### 당시 남은 게이트와 중단 조건 — 아래 재개 검증으로 해소

- 저장 공간이 작업 중 8.6GiB에서 **6.4GiB**로 감소했다. 저장소 추적 파일만 약 3.34GiB다.
  별도 review checkout과 workspace/native/WASM 빌드가 필요한 전체 검증을 현재 여유 공간으로
  강행하지 않는다. 다른 작업 파일을 삭제하지 않고 공간 확보 또는 외장 경로를 요청했다.
- Rust focused 17건, review suite prepare/check, 필수 Clippy 3종·workspace build,
  release-test 전체·Native Skia·fresh WASM 검증은 미완료다. 이전 후보의 통과로 대체하지 않는다.
- source push 및 새 CI는 아직 수행하지 않았다. merge·이슈 종료도 수행하지 않았다.
- 리뷰 13: 현 source에는 `orders/20260907.md`가 없으나 확인한 upstream/devel에는 #6818 기록과
  source에 없는 내부 링크가 있다. 해당 문서를 복사·덮어쓰거나 add/add 충돌을 만들지 않았다.
  현재 단계는 구현 후 재검증 중이므로 8.2.1의 최종 준비 시점에 대장 통합 경로를 확정한다.
  현행 CLAUDE.md는 문서 포인터이며 리뷰가 인용한 #10/#13 본문과 동일하지 않다.

### 사용자 승인 코멘트 게시

- [보정 코멘트](https://github.com/edwardkim/rhwp/pull/6814#issuecomment-5561537199)를 게시했다.
  검증한 사실, 리뷰 해석 정정, 아직 로컬인 구현과 남은 검증을 구분했다.
- UTF-8 Markdown 파일을 `gh pr comment --body-file`로 게시하고 API 본문을 원문과 `diff`로
  비교했다. gh 출력의 추가 끝 개행 1개를 정규화한 뒤 일치함을 확인했다.
  한글 치환·선두 BOM 문제는 없다.
- 보정 문서 3개의 상대 링크 검사와 `git diff --check`가 통과했다.

## 공간 확보 후 최종 재검증 (2026-09-07)

- 사용자 공간 확보 후 여유 81GiB에서 별도 review worktree를 만들고 재개했다.
- 최종 코드: `01df465fabba276d0063cb06aa25f633aca2d825`.
  `cef551e2e`는 Undo 복구 대기 중 후속 Redo를 차단하되 순서를 보존하며,
  `01df465fa`는 셀 batch 종료 오류까지 적용·복원 오류와 함께 보존한다.
  apply/endBatch/rollback/endBatch 복합 오류와 Undo batch 종료 실패의 재시도를 추가 검사했다.
- review checkout은 `a8ba4b0ea`에서 Rust 검증을 실행한 뒤 최종 코드로 fast-forward했다.
  사이 변경은 Studio command/history와 해당 테스트뿐이며 Rust 소스·테스트·Cargo는 동일하다.
- 로그·생성물: `/private/tmp/rhwp-6814-revalidation.PRSoBQ/`.
  review checkout: 같은 경로의 `review/`, target: `review/target/pr-review`.

| 검증 | 결과 |
| --- | --- |
| suite prepare → fmt → fmt check → native Clippy → WASM lib Clippy → workspace build → workspace/all-targets Clippy → manifest check | 순차 통과, 모든 Clippy `-D warnings` |
| Rust focused `issue_6788_mixed_char_format` | 17 passed, 153 filtered/skipped |
| Rust release-test 전체 nextest | **9073 passed, 0 failed, 46 skipped**, slow 1, LEAK 표시 없음 |
| Native Skia lib | rhwp 3930 + workspace 182 passed, 13 ignored |
| Native Skia placeholder / direct PDF | 2 / 4 passed |
| locked host WASM nodejs / web `--no-opt` | 새 Rust로 모두 빌드 성공 |
| 최종 코드 + fresh WASM Studio `npm test` | **1428 passed, 0 failed, 0 skipped**, 실제 WASM-command-history 13개 시나리오 포함 |
| frontend binding + suite manifest 계약 | 22 passed, 0 failed, 0 skipped |
| 최종 Studio TypeScript/Vite / Firefox 확장 build | 모두 통과, 설치된 확장은 교체하지 않음 |
| HWP/HWPX 4상태 × 2포맷 export·재열기 | 8파일, 7글자의 textColor/shadeColor/bold/fontSize/fontFamily 일치 |
| Native PNG 전체 페이지 비교 | before=undo, highlight=redo, HWP=HWPX 총 8쌍 모두 0픽셀 차이 |

### 시각 증적과 해석 범위

- `generate.mjs`는 최종 Studio의 실제 ApplyCharFormatCommand/CommandHistory/WasmBridge와
  새 Node WASM으로 4상태를 생성·저장·재열기한다. `roundtrip/expected-properties.json`에 결과를 남겼다.
- fresh Native Skia CLI `export-png --page 0 --profile screen --scale 2`로 8파일을 렌더했다.
  `compare-png.mjs`의 전체 페이지 비교 결과는 `roundtrip/png-checks.json`에 있다.
  1588×2245 페이지 출력에서 문서 영역만 잘라 좌우 HWP/HWPX, 위에서 적용 전/형광펜/Undo/Redo로
  배치한 패널을 직접 확인했다. 보라색·굵기는 유지되고 선택한 네 글자의 형광펜만 제거·복원된다.
- 새 패널 SHA-256은 `7566cbaeba0f24cf322a24f3ed6aeafe7ce524df8d1d0694ba297fd0209f1e22`로
  기존 [CLI 증적](../assets/issue6788_cli_roundtrip.png)과 동일하다. 중복 이미지는 추가하지 않는다.
- web/Node/Firefox WASM SHA-256은 모두
  `c0df102c41d5b1a3cfbc002e98ff46a53b59210d3425e40ee41be6ece00bcce1`이다.
- Chrome 플러그인 연결이 제공되지 않아 이번 보정의 새 브라우저 UI 실행은 하지 않았다.
  기존 Chrome·Firefox 스크린샷은 최초 구현의 직접 UI 증적이며, 이번 증적은 fresh WASM 기능 테스트와
  native PNG다. Docker daemon 부재로 `--no-opt`를 사용했으므로 최적화 배포 패키지 검증은 아니다.
- 기존과 같이 HWPX fillType/patternColor/patternType 차이는 적용 전부터 존재한다.
  이슈 대상 5속성 보존을 확인했으며 모든 속성의 무손실을 주장하지 않는다.

### 제출 전 판단과 후속 처리

- 최신 devel `56706247f4950286117496c41f5b2c4b1cdbddc5`와 최종 코드의 merge-tree는
  `b122242bb2cb184b74f52db08eb6ad593e3cc57d`로 충돌 없이 생성됐다.
  변경된 코어/Studio 주요 경로에는 최초 기준 이후 devel 변경이 없다. 이는 최신 CI를 대체하지 않는다.
- 로컬 필수 게이트를 완료해 보정 push 가능으로 판단한다. 최신 PR head CI와 merge 승인은 별도다.
- 리뷰 13은 사용자 답변 **“오늘할일은 병합 시점에 갱신”**에 따라 병합 시점에 처리한다.
  최신 devel의 다른 PR 기록을 복사하거나 덮어쓰지 않는다.
- 기존 보정 코멘트는 이 결과로 갱신하며 PR 본문은 기존 화면을 유지하고 검증 수치만 간결하게 보완한다.
  merge·이슈 종료는 수행하지 않는다.
