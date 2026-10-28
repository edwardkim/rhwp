# PR #7712 리뷰 — 매개변수 누름틀 삽입

## 최종 판정

**병합 완료 — 원 head의 매개변수 누름틀 삽입 범위 검토 및 로컬 검증 완료.** 사용자 승인 후 정확한 head의 필수 CI와 MERGEABLE/CLEAN 및 current-base merge-tree를 재확인하고 approve·merge했다. 소스 보정은 없다. merge commit은 `665d196794dff842a75c4c1601c96bcfccaaed78`이며 #7707은 devel push 자동화로 종료됐다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#7712](https://github.com/edwardkim/rhwp/pull/7712) / wook8170 / devel |
| 원 head | `606103089308a5044ca852975578968435efe4e2` |
| 기준 devel | `21ed64071f81f35f37618717693bd30f727392c9` |
| 관련 이슈 | [#7707](https://github.com/edwardkim/rhwp/issues/7707), closes 선언 |
| 규모 | 1 commit · 9파일 · +230/−24 |
| reviewer·트리아지 | edwardkim / assignee wook8170 / milestone v1.0.0 / enhancement, rhwp-studio, api, typescript, javascript |
| 작성 시점 상태 | 2026-10-11 병합 직전 non-draft · MERGEABLE/CLEAN · 최신 원 head 필수 CI 성공 |
| 로컬 branch | review/wook8170-pr7712-20261011 |

기본 경로: maintainer_general. 보조 경로: intake_and_review, local_validation, multi_pr_update_branch, first_time_contributor. 모 workflow·선택표·각 자식·review_template 및 개발 환경·편집 checklist·CDP 가이드를 읽었다.

## 변경과 검토 범위

`commands.execute('insert:field', params)`가 이미 알고 있는 name/guide/memo/editable/type을 전달하면 대화상자 없이 현재 캐럿에 삽입한다. 값이 없는 호출과 메뉴의 anchorEl만 있는 호출은 기존 대화상자 경로를 유지한다. 같은 삽입 함수를 공유하고 기존 form-mode/canExecute 게이트를 통과한다.

소비 경로: npm/editor/index.js의 commands.execute → embed/rpc-router.ts의 automation.execute → main.ts의 automationExecute → AutomationHost.execute의 runsWithoutDialog → CommandDispatcher.dispatchWithResult의 form/canExecute → insert.ts의 parseFieldInsertParams 및 insertAtCaret → InputHandler.executeOperation(snapshot/insertField) → 기존 wasm.insertClickHereField 및 history·full refresh.

parse는 문서 mutation 전에 형식·허용 키·길이를 검사하며 dialog와 동일한 상한(name/guide 250, memo 1000)을 쓴다. snapshot 실패는 dispatcher의 threw로 전달된다. renderer·layout·WASM·serializer, 저장 속성 해석, baseline/golden/허용치 변경은 없다. 다른 세 PR과 소스 commit 의존성은 없으나 README 등 공통 파일이 있으므로 merge마다 다음 head를 다시 확인한다.

## 조판 원칙·검증 입력

조판 규칙 변경은 **비해당**이다. 기존 대화상자의 ClickHereProps를 기존 동일 wasm.insertClickHereField로 전달하는 공개 진입점 확장이며 조판 소비 조건·좌표·메트릭은 바꾸지 않는다. 새 렌더링 회귀나 fixture 추가도 없어 Visual Sweep/TSV를 요구하지 않는다. 논리 캐럿 위치·필드 속성·Undo/Redo·실패 상태·저장 왕복을 검증한다.

실물 HWP/HWPX/PDF 파일은 이 PR 검증에서 사용하지 않았다. 브라우저에서 빈 문서를 생성하고 메모리의 저장 bytes로 왕복하는 입력은 코드 생성 입력이다. 입력 파일 커밋 확인은 **비해당**이며 한컴 출력과의 시각 일치를 입증한 것으로 보고하지 않는다.

## 완료한 검증

- source/fork remote SHA 일치, 최신 devel 조상 확인, `git diff --check upstream/devel...HEAD` 성공.
- `git merge-tree --write-tree upstream/devel upstream/pr7712-head` 성공, tree `83ec7502647e854b09d20805d6bf0bb6b4c8680d`.
- `node --test rhwp-studio/tests/insert-field-params.test.ts`: 5/5 통과. parser 행위 검사와 소스 배선 가드가 섞여 있으므로 실제 편집 증거는 E2E로 구분한다.
- `npm --prefix rhwp-studio test`: tests 1832, pass 1830, fail 0, skipped 2.
- `npm --prefix npm/editor test`: 32/32 통과.
- CI-unit TypeScript 및 npm/editor/index.d.ts TypeScript 검사 성공, editor pack dry-run 성공.
- `python3 scripts/check_e2e_manifest.py`: 151개 목록 정합.
- [CI run 38047967696](https://github.com/edwardkim/rhwp/actions/runs/38047967696): exact head `60610308…`, pull_request, completed/success. required Build & Test 성공, Frontend package gates 성공. WASM Build·Rust lint·Native·Rust builders·Frontend unit job은 skipped이며 해당 실행을 로컬 WASM/브라우저 검증으로 대신 기록하지 않는다.

## Docker WASM·실제 브라우저 검증

- 기본 작업공간 root devel `21ed64071f81…`에서 `docker compose --env-file .env.docker run --rm wasm`을 완료했다(exit 0, wasm-pack 7분 50초). Rust 1.93.1이며 표준 named-volume cache를 재사용했다. PR과 root의 Rust·Cargo·toolchain·build wrapper diff가 없으므로 같은 engine 산출물을 검토 worktree에 복사했다. 다른 세 PR에도 Rust 변경이 없어 동일 산출물을 재사용할 수 있다.
- pkg/public SHA-256 일치: rhwp.js `70cde06a369fa7fd4fc8bc8f3d6acaee158596ba1a116a2c72159002b0b5654e`, rhwp_bg.wasm `cc5e9c9d86d2d3ac117a9184510bca661fb8cfaa6aef660707de2c9063575de2`.
- `npm --prefix rhwp-studio run build`: 전체 TypeScript와 production build 성공. 기존 대형 bundle 안내는 경고이며 build 실패가 아니다.
- `node --test scripts/frontend-wasm-bindings.test.mjs scripts/frontend-editor-embed.test.mjs`: 3/3 통과. pkg 생성 전 실행의 ENOENT는 환경 준비 중 결과였고 fresh pkg 반영 후 통과했다.
- Node v24.15.0, CDP Windows Chrome 155.0.8059.40, `CHROME_CDP=http://localhost:19222`, `VITE_URL=http://localhost:7791`, host 모드로 실행했다. 서버는 검토 worktree의 Studio를 제공하며 Chrome의 기존 탭은 유지했다.
- `node e2e/automation-commands.test.mjs --mode=host`: 32개 assertion 통과. 신규 매개변수 삽입·이름/안내문/메모 반영·잘못된 입력 무변경·Undo 1회·기존 다른 대화상자 정책·확장 명령 회수를 확인했다.
- ignored `output/pr-review/pr7712-20261011/behavior-probe.mjs`: 15개 assertion 통과. 실제 키보드 캐럿 글자 1과 새 필드 startCharIdx/문단 소속 일치, editable=false, 형식·허용 키·상한 초과 9종 거절과 필드 수 유지, 1회 Undo/Redo 및 반복 삽입의 개별 취소·복원, anchorEl-only UI 대화상자 표시를 확인했다. HWP/HWPX 메모리 저장·재열기에서 이름·안내문·메모·editable 보존을 확인했다.
- 같은 probe에서 공개 `createStudio().commands.execute` iframe RPC 경로로 삽입·오류·Undo/Redo와 인자 없는 needs-dialog를 확인했다.
- 음성 대조: 메모리의 command 정의에서 runsWithoutDialog만 일시 제거해 기존 정책으로 되돌렸다. 같은 올바른 매개변수 요청은 needs-dialog로 거절됐다. finally에서 원 정의를 복구한 뒤 새 경로 삽입이 성공했다. 사용자 소스나 commit을 되돌리지 않았으며 실제 제품 진입점의 변경 동작을 검출했다.
- `node e2e/undo-contracts.test.mjs --mode=host`: 모든 assertion 통과. 문단 모양·대화상자 취소·메모·표 속성 undo 및 Through 배치 보존을 확인했다.

## 제한과 다음 단계

이 판정은 UI/자동화 공개 경로를 확장하는 원 PR 범위에 대한 것이다. 강제 WASM 내부 오류, 모든 분리 캐럿 공간·모든 표 구조의 전수 검증이나 한컴 출력과의 픽셀 일치를 주장하지 않는다. 새 조판 정책 또는 renderer 변경이 없으므로 Rust 전체 회귀·Clippy·Visual Sweep은 비해당이다. 기존 공통 snapshot 경로를 재작성하거나 별도 mutation 우회 경로를 추가하지 않았다.

사용자는 #7712 병합·#7707 종료와 후속 처리를 승인했다. 2026-10-11T00:55:57Z에 원 head `606103089308a5044ca852975578968435efe4e2`를 approve했고 00:56:01Z에 merge 방식으로 병합했다. `--match-head-commit`으로 원 head를 고정했고 기여자의 commit을 보존했다. [merge commit](https://github.com/edwardkim/rhwp/commit/665d196794dff842a75c4c1601c96bcfccaaed78)은 최신 devel에 포함된다. #7707은 00:56:13Z에 자동 종료됐으며 한국어 검증 요약 및 첫 기여 감사 후속 댓글을 게시하는 단계다. contributor fork `wook8170/rhwp`의 `feat/insert-field-params` branch는 삭제하지 않는다.

운영 기록은 maintainer 직접 반영 경로로 이 archive와 [오늘할일](../../orders/20261011.md)만 devel에 한 commit으로 보존한다. [duration 메타데이터 작업](https://github.com/edwardkim/rhwp/actions/runs/38100103101)은 success이며 `ready:false / no-verified-pr-duration-measurements`로 갱신을 보류했다. 후보 CI의 worker B가 skipped여서 실측을 사용할 수 없다는 notice를 확인했다. [이슈 종료 작업](https://github.com/edwardkim/rhwp/actions/runs/38100103097)도 success다. 검증 CI를 병합 후 재실행하지 않았다. 필수 후속 처리 뒤 소유한 Studio 서버·검토 worktree·local review branch·fetch ref·임시 출력은 정리하고 공용 target과 다음 세 PR의 증거는 보존한다.

실행 로그·진단 probe: ignored `output/pr-review/pr7712-20261011/`. HTML: `output/e2e/automation-commands-report.html`, `output/e2e/behavior-probe.mjs-report.html`, `output/e2e/undo-contracts-report.html`. Docker 빌드 로그와 트리아지 전후 snapshot: 기본 작업공간 `output/pr-review/wook8170-20261011/`.
