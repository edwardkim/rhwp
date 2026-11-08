# #8102 셀 진입 경로의 편집 대상 표현 정규화 결과

Issue: #8102 — https://github.com/edwardkim/rhwp/issues/8102

PR: https://github.com/edwardkim/rhwp/pull/8180

## 범위와 원인

기준 `upstream/devel`: `def947c2149df5bc812d801e6e79be19a403b4d1`.
검증한 최종 제품 코드·테스트 commit: `076c6fd95626e141ff187a30af3ba3ed52286188`.
브랜치: `codex/issue-8102-cell-target`.

표 생성 직후의 위치는 flat 셀 좌표만 가지고 있었다. 셀 전체 선택은 flat 조회로 성공했지만
삭제 명령은 경로 누락을 `[]`로 직렬화해 core의 `경로가 비어있습니다` 오류를 발생시켰다.
클릭으로 다시 진입하면 hit-test가 경로를 제공하므로 같은 삭제가 성공했다.

`cellPathForEdit`가 유효하고 완전한 flat 위치를 depth 1 경로로 해석한다. 기존 중첩·글상자
경로를 우선하며 원본을 복사한다. 선택 조회와 편집 직렬화·문단별 경로가 같은 정규화를 쓴다.
표 생성도 처음부터 경로를 제공한다. 중첩 전체 선택은 최내곽 문단을 변경하면서 최외곽 flat
문단을 보존한다. 선택 삭제는 서로 다른 셀·잘못된 순서·누락 좌표·범위 초과를 snapshot 및
mutation 전에 거부한다. 셀 전체 선택의 잘못된 대상은 오류를 전달해 상위 단축키가 본문 전체 선택으로
확장하지 않게 한다. redo는 기존 스냅샷 복원 계약을 유지한다.

재검토에서 safe integer인 `2^32` 주소가 WASM의 `u32` 변환으로 0이 되어 실제 셀을 지우는
누락을 찾았다. 공통 정규화는 이제 모든 주소를 `0..0xffff_ffff`로 제한하고 기존 문서 존재 여부
조회도 유지한다. 같은 셀 거부 테스트는 같은 문단의 정방향 범위를 사용해 주소 차이와 역방향
범위를 각각 검증한다. 누락 좌표를 채우거나 큰 값을 자르지 않고 사전에 거부한다.

호스트 #594 및 rhwp #8104의 오류 payload·복구 계약, 전역 오류 정책, core 삭제 알고리즘은
수정하지 않았다. 다른 미병합 PR의 코드를 사용하지 않았다. 이 PR에 이슈 종료 명령을 넣지 않는다.

## 수정 전후 재현

동일한 devel Rust source에서 새로 만든 web WASM과 Linux Chromium을 사용했다.
수정 전 비교는 Vite의 읽기 전용 load hook으로 세 수정 대상 파일을 기준 SHA의 원문으로
대체했다. 작업 트리를 되돌리거나 다른 PR 코드를 섞지 않았다.

| 경로 | 수정 전 | 수정 후 |
| --- | --- | --- |
| 2×2 표 생성 직후, 재클릭 없이 `alpha` paste → Ctrl+A → Backspace | 빈 경로 예외, `alpha` 보존 | 삭제·undo·redo·undo 뒤 재삭제 성공 |
| 실제 마우스로 첫 셀 재진입 | 삭제·undo/redo 성공 | 같은 결과 |
| Tab → Shift+Tab 진입 | 빈 경로로 삭제 실패 | 같은 셀 삭제·undo/redo 성공 |
| 삭제 undo로 셀 재진입 | 최초 삭제 실패로 진행 불가 | 복원한 셀에서 재선택·삭제 성공 |
| legacy flat 위치 직접 전달 | 삭제 실패 | depth 1로 정규화, 삭제 성공 |
| 세 문단 `alpha/beta/gamma` | 선택 후 삭제 실패 | 세 문단 삭제·undo 복원·redo 성공 |

수정 후 여섯 UI 시나리오 모두 인접 셀 `NEIGHBOR`를 보존했다. 별도 브라우저 WasmBridge의
실제 CursorState/CommandHistory를 통해 중첩 세 문단 삭제, 바깥 셀 `OUTER` 및 인접 셀 보존,
글상자 `BOX` 삭제·undo/redo도 확인했다. 내용이 존재하는 상태에서 invalid target 9건은 모두
거부됐고 snapshot 0회·mutation 0회·HWP export 바이트 보존을 확인했다. 실제 `handleSelectAll`의
invalid target도 수정 전에는 본문 선택으로 확장됐지만,
수정 후에는 오류를 전달하고 셀 위치·문서 바이트를 보존하며 선택을 만들지 않았다.
최종 #8102 브라우저 검사에서 pageerror는 0건이다.

기존 `select-all-cmd-a` E2E에서도 실제 Studio의 중첩 셀·글상자 선택 삭제와 undo가 통과했다.
셀 선택 삭제 undo 시 선택 하이라이트 복원은 기존 UI가 제공하지 않으므로 이 수정의 완료 주장에
포함하지 않는다. 내용과 셀 위치 복원 및 재선택·삭제를 검증했다.

## 실행 명령과 결과

실행 환경: 별도 Linux 클라우드 세션, Node `v24.19.0`, Rust `1.93.1`, wasm-pack `0.14.0`,
시스템 Chromium headless. Cargo target은 `/workspace/rhwp/target/pr-review` 한 곳을 사용했다.

| 명령 | 결과 |
| --- | --- |
| `npm --prefix rhwp-studio ci` | 초기 환경 준비 PASS. 이번 보정은 lockfile 변경 없음 |
| `CARGO_TARGET_DIR=/workspace/rhwp/target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --dev` | PASS, fresh web WASM |
| 같은 wrapper `--target nodejs --out-dir pkg-node --dev` | PASS, fresh Node WASM |
| `cd rhwp-studio && npx tsc --noEmit` | PASS |
| `npm --prefix rhwp-studio test` | **2375 PASS / 0 FAIL / 0 skip** |
| `npm --prefix rhwp-studio run build` | PASS, production bundle. chunk 크기 안내 경고 있음 |
| `npm --prefix rhwp-studio run e2e:issue-8102` | PASS, UI 6경로 + 중첩·글상자·invalid target |
| `npm --prefix rhwp-studio run e2e:select-all-cmd-a` | PASS, 중첩·글상자·본문·선택 경계 |
| `npm --prefix rhwp-studio run e2e:issue-6638` | PASS, 빈 문단 포함 셀 paste 보존 |
| `cd rhwp-studio && node e2e/run-with-vite.mjs -- node e2e/undo-contracts.test.mjs --mode=headless` | PASS, 기존 편집 undo 계약 |
| `git diff --check upstream/devel...HEAD`, `git diff --check` | PASS |

추가 실제 Node WASM 경계 회귀(`node --experimental-transform-types --no-warnings
work/recheck8102-invalid.test.mjs`)는 2 PASS / 0 FAIL / 0 skip이었다. `sectionIndex`와
`parentParaIndex`의 `2^32`를 거부했고 두 경우 모두 snapshot·mutation 0회와 bytes 보존을 확인했다.
보조 Node 진단 `work/recheck8102-deep.mjs`도 깊이 2·3의 0이 아닌 중간 문단·control·셀, Unicode,
인접 셀 보존 및 undo/redo 3회를 통과했다. 정식 완료 근거의 HML 생성식은 커밋된 #8102 E2E에
있으며 외부 사용자 문서나 공개되지 않은 파일에 의존하지 않는다.

초기 npm 실행의 2 skip은 `pkg-node` 부재였고, fresh Node WASM 준비 후 위 전체 검사를 다시 실행해
skip 0건으로 확인했다. 초기 E2E 검사 스크립트의 Puppeteer 키 조합 표기·줌 조회 오류도 수정 후
재실행했다. 이 초기 실패·skip을 성공으로 집계하지 않았다.

이번 보정의 첫 전체 npm 실행은 기본 샌드박스에서 하위 테스트/드라이버 stdout 전달이 누락되어
325개 파일 중 18개가 실패했다. 저장소와 무관한 두 assertion의 최소 Node 재현에서도 파일 1개
결과만 나오고 stdout이 누락됐다. 정상 프로세스 실행 환경에서는 두 assertion과 stdout이 모두
나왔다. 테스트·게이트·npm 명령을 바꾸지 않고 같은 code SHA의 전체 검사를 재실행해 위의
2375 PASS / 0 FAIL / 0 skip을 확인했다. 최초 실패 로그도 별도 보존했다.

최종 web `rhwp_bg.wasm` SHA-256:
`b02e4860cb22f2f1fe94c1d2b9675a9b4f4b39a4222f6bfb4b7258323d16d109` (66,031,939 bytes).
JS SHA-256: `bafa900f2814ed17312932aa6ca88f2d8dc57123998f313d637d1f7e5ecb7ec4`.
루트 `pkg`와 Studio `public` 해시가 각각 일치했다. 네 E2E에서 초기화 중 실제 수신한 WASM
응답 바이트의 해시도 위 값과 일치했다. 이후 문서 기록 commit은 제품 source·검사 source를 바꾸지 않는다.

최종 code commit `076c6fd95626e141ff187a30af3ba3ed52286188`에서 TypeScript·전체 npm·build·fresh
web/Node WASM·E2E 4종 및 두 diff 검사를 다시 실행했다. 보존한 최종 로컬 증적은 ignored
`output/pr-review/issue-8102/correction/`의 `logs/`, `wasm-build.json`, `consumption/*.json`,
`browser.json`, `commands.json`, `result.txt`이다. `sandbox-attempt/`에 최초 전체 검사 실패를
보존했다. 이전 code 검증은 `final/`에 별도로 남아 있다. 수정 전 결과는 상위 폴더의 `browser-before.json`과
`logs/invalid-shortcut-before.log`에 보존했다. 임시 로그·JSON·생성 WASM은
커밋하지 않았다. 재현 fixture 생성식은 정식 E2E source에 포함했다.

## 보정 후 로컬 재검토

- P2 주소 상한 누락: 실제 WASM에서 보정 전 두 잘못된 주소가 모두 `alpha`를 삭제했지만,
  보정 후 사전 거부·snapshot 0회·mutation 0회·bytes 보존으로 바뀌었다. 브라우저도 flat/path
  주소의 상한 초과 4건을 포함한 9건을 같은 조건으로 통과했다. 최대 u32의 존재하지 않는 주소
  2건도 사전에 거부해 정수 범위 검사와 문서 존재 여부 검사가 모두 유지됨을 확인했다.
- P3 같은 셀 거부 테스트: flat 셀·control·부모, 중첩의 중간 문단·control·셀만 각각 다른 정방향
  범위로 보강했다. 메모리에서 `sameCell` 거부만 제거하면 해당 두 테스트가 실패하며, 실제 제품
  source는 변경하지 않았다. 제품 focused 검사는 12 PASS / 0 FAIL / 0 skip이다.
- 정상 동작: 생성 직후·클릭·키보드·undo 진입, 여러 문단·중첩·인접 셀·글상자와 undo/redo를
  같은 최종 code SHA에서 다시 검증했다. 이 수정 범위에서 추가 blocker는 확인하지 못했다.
- 조판 원칙: 기존 문서 주소의 해석과 사전 거부만 바꾸며 조판·paint·저장 속성·Rust/core는
  변경하지 않았다. 문서별 좌표 보정·렌더러 baseline 변경은 없다.
- 이는 로컬 보정 재검토 결과다. 원격의 새 head CI 성공, 정식 self-review 기록, merge를 대신하지
  않는다. 사용자 문서 확인과 아래 미검증 범위를 유지한다.

## 보조 검사 실패와 미검증 범위

- `python3 scripts/check_e2e_manifest.py`: **FAIL 1건**. 기준 devel부터 추적된
  `canvaskit-periodic-borders-issue7998.test.mjs`의 원장 행이 없다. 기준 SHA의 manifest·package·
  workflow·tracked 목록으로 같은 검사기를 실행해 동일한 1건을 확인했다. #8102 파일·manifest 행·
  npm 배선의 신규 오류는 없다. 다른 이슈의 파일·검사를 수정하지 않았으며 전역 검사 PASS로 표시하지 않는다.
- Rust source·Cargo 설정·조판/렌더러·저장 속성을 바꾸지 않았다. 기존 core ByPath 삭제에 전달하는
  주소와 UI 선택 표현만 바꾸므로 Rust 전체 lint/nextest, Native Skia/Print PDF Visual Sweep은
  이 Studio 단독 주소 정규화 검증 범위에 해당하지 않는다. native/release WASM 결과로 과장하지 않는다.
- 사용자 실제 문서는 제공되지 않았다. macOS 네이티브 Command 키·IME·임베딩 호스트 배포·
  사용자 실제 문서에서의 같은 셀 진입·선택 삭제·undo는 사용자의 직접 확인이 남아 있다.
- GitHub CI는 PR 생성 뒤 별도로 실행된다. 로컬 결과로 CI 성공이나 병합 가능을 대신 판정하지 않는다.

## 사용자 직접 확인 절차

1. PR 브랜치의 Studio에서 빈 문서를 열고 기본 2×2 표를 만든다.
2. 첫 셀을 다시 클릭하지 않고 `alpha`를 붙여넣는다. Command+A, Backspace로 셀만 비워지는지 확인한다.
3. Command+Z로 `alpha`가 복원되는지, redo로 다시 지워지는지 확인한다. undo 뒤 다시 선택·삭제한다.
4. 같은 셀을 클릭해 진입한 경우, Tab/Shift+Tab으로 진입한 경우도 같은 결과인지 비교한다.
5. 사용자 문서 사본에서 다중 문단·중첩 셀을 대상으로 반복하고, 옆 셀·바깥 셀 내용이 유지되는지
   확인한다. 원본은 덮어쓰지 않고 검토용 사본에서 실행한다.

[1단계 분석·구현 기록](../working/task_m100_8102_stage1.md)

[2단계 invalid target 호출 경로 검증](../working/task_m100_8102_stage2.md)

[3단계 주소 상한·거부 회귀 보정](../working/task_m100_8102_stage3.md)
