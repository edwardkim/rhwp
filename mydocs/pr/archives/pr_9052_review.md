# PR #9052 리뷰 — 그림·도형 CLI와 MCP 편집 계약

## 최종 판정

**승인 — 메인터너 보정이 반영된 code head `24aa6c04bc0f632fc6aae2ee8db356fdd85a72d2`에서 #9019·#9020의 CLI/MCP 계약과 필수 로컬 검증·원격 CI를 충족했다.** 원 기여 head `de8209ab`를 보정 없이 승인한 판정은 아니다.

병합 전 조건은 이 문서 전용 trailing head의 최신 required checks·재사용 근거·MERGEABLE/CLEAN 재확인과 작업지시자의 별도 병합 승인이다. 문서 판정은 GitHub 승인 event나 병합 완료를 뜻하지 않는다.

## 접수와 첫 기여자 처리

| 항목 | 확인 결과 |
| --- | --- |
| 원 PR·작성자 | [#9052](https://github.com/edwardkim/rhwp/pull/9052), JinHo-von-Choi, `FIRST_TIME_CONTRIBUTOR` |
| 대상 | `edwardkim/rhwp:devel` |
| 원 source | `JinHo-von-Choi/rhwp:task/9019-picture-edit-surface` |
| 원 head | `de8209abcb2aabe2f57d25190f18baddb68807bc` |
| 메인터너 보정 | `24aa6c04bc0f632fc6aae2ee8db356fdd85a72d2` |
| 원 fork 분기점 | `1003349996897c6ff097271e7296ae4d1a3d807f` |
| 최신 devel 대조 | `959300e66c58241a47e6d46030b4b0854e59604b` |
| 관련 이슈 | [#9019](https://github.com/edwardkim/rhwp/issues/9019)·[#9020](https://github.com/edwardkim/rhwp/issues/9020), PR의 `Closes` 대상 |
| reviewer | `jangster77`, API 배정 확인. 로컬 `review/pr-9052-picture-cli` 시작점·fork ref·PR head 일치 확인 |

첫 기여에 감사와 환영을 전하고, 추가 확인한 JSON 경계와 보정 이유를 [댓글](https://github.com/edwardkim/rhwp/pull/9052#issuecomment-6855822896)로 설명했다. 원 3개 커밋을 보존하고 원 head 위에 보정 커밋만 추가했다. 기여자 이력을 rebase/amend/force-push하거나 무관한 devel 이력을 fork에 섞지 않았다. 이 문서는 단독 원 PR의 검토 기록이며 별도 통합 PR은 만들지 않는다.

원 PR은 17개 파일·1,837줄 추가로 대형 검토 경로에 해당한다. 변경을 CLI/MCP 표면·공통 속성 검사·문서·회귀로 나눠 확인했으며 필수 검증을 줄이거나 CI를 우회하지 않았다. 원 해결 경로 #8439/#8442, #8556/#8595, #8557/#8632와 #6806의 getter→setter 계약도 대조했다.

## 원 기여 내용과 메인터너 보정

기여자는 `set-picture`·`set-hf-picture`의 잘못된 속성을 오류로 반환하고, 그림 삽입 결과에 실제 컨트롤 주소를 제공했다. `set-shape`, 본문·표 칸으로의 `move-picture`, 표 칸 `insert-picture`와 대응 MCP 도구·capabilities·프로필·도움말·매뉴얼도 연결했다. 기존 코어 편집 함수를 호출하며 Studio 조판이나 Undo/Redo 라우터를 새로 구현하지 않는다.

기존 11개 집중 검사는 원 head에서도 통과했다. 추가 반례에서 검사기는 JSON escape를 해석하고 중복 키의 마지막 값을 보지만, setter는 원 문자열을 달리 읽는 문제가 드러났다. 예를 들어 `{"\u0074extWrap":"TopAndBottom"}`은 성공을 반환하고도 저장값이 `SQUARE`였다. 앞에 잘못된 값이 있는 중복 `textWrap`도 같은 불일치를 일으켰다.

보정은 공통 속성 검사에서 **해석한 키의 중복을 거부하고 검증한 JSON을 정규화해 반환**하도록 한다. 본문 그림·도형·머리말 그림 CLI가 그 반환 문자열을 setter에 전달한다. 이번 PR에서 새로 도입한 Rust helper `check_picture_props`·`check_shape_props`의 반환형은 `Result<String, String>`이며, 설명과 소비자를 함께 갱신했다. 기존 WASM setter의 조회 결과 되먹임 계약은 유지한다.

| 반례 | 원 head | 보정 head |
| --- | --- | --- |
| escape가 있는 속성 키·enum 값 | exit 0이지만 `SQUARE` 저장 | `TOP_AND_BOTTOM` 저장 |
| 앞쪽 값이 잘못된 중복 키 | exit 0·출력 생성 | exit 2·출력 없음 |
| 뒤쪽 값이 잘못된 중복 키 | 오류 | 동일하게 중복 키 자체를 거부 |
| 일반·escape 표기가 같은 중복 키 | 해석 불일치 위험 | 해석한 키 기준으로 거부 |
| 도형·머리말 그림 같은 경계 | 새 회귀 실패 | 실제 적용·저장 및 오류 시 미생성 확인 |

새 4개 회귀를 **원 제품 코드에 테스트만 추가해 0 PASS/4 FAIL**로 확인한 뒤 보정했다. 보정 후 기존 머리말 대조군까지 포함한 **19 PASS/0 FAIL**이다. 새 검사는 `tests/cases/`에만 두었고 절대 px 배치나 전체 SVG 해시를 고정하지 않는다. 기존 단조 입력·키/enum/range/caption 조건과 dry-run 거부 검사는 보존했다.

## 검증 입력·실제 소비 경로

기존 커밋 입력 `samples/paper-square-table-tac-host.hwpx`, `samples/images/moogung.jpg`, `assets/logo/logo-16.png`를 사용했다. 실행 파일과 Git 입력의 SHA-256·크기·producer는 증적 JSON에 보존하며 새 원본 사본은 추가하지 않는다.

검증 경로는 CLI 인자 → 공통 JSON 검사/정규화 → 기존 코어 편집 함수 → HWP/HWPX 저장 → 다시 열기 → 재저장 → 다시 열기다. 실제 명령을 통해 본문 그림과 표 칸 그림의 주소·소속·누락/중복, 이동 출발지 제거와 목적지 단일 소유, 지원하지 않는 TAC·잘못된 대상·충돌 인자, 도형 속성, dry-run과 오류 시 파일 미생성을 확인했다.

로컬 stdin/stdout MCP에서도 도구 선언과 본문 삽입·표 칸 삽입·본문 이동·표 칸 이동·그림 속성·도형 속성 호출을 실행했다. `set-hf-picture`는 실제 CLI 회귀와 MCP 도구 선언을 확인했으며 MCP의 머리말 편집 호출까지 실행한 것으로 세지 않는다. 이 MCP 검사는 `rhwp mcp-serve`에 대한 것으로 한컴 PDF 변환 서비스 호출이 아니다.

별도 Python `zipfile`·XML reader로 생성 HWPX 12개를 읽어 본문/칸 그림 소속과 도형 속성을 교차 확인했다. 그림 출력 10개는 입력 JPEG 바이트를 그대로 보존했다. 이 reader는 rhwp parser의 재조회가 아니다. 테스트가 만든 중간 파일·실행 바이너리·스크립트·원 로그는 ignored output에 두며 정식 증적에는 입력 출처와 결과·해시를 남긴다.

## 로컬 검증

| 항목 | 실제 결과 |
| --- | --- |
| 원 head 집중 | 11 PASS |
| 새 반례, 제품 보정 전 | 4 FAIL, 기대한 결함 재현 |
| 보정 후 집중 | 19 PASS / 0 FAIL |
| 전체 nextest | 13,438 PASS / 0 FAIL / 45 skipped, 테스트 341.474초 |
| fmt·Native Clippy·WASM32 lib Clippy | PASS |
| workspace build·workspace all-target Clippy | PASS |
| suite manifest·unit tiers | PASS |
| 실제 CLI·로컬 MCP·독립 ZIP 확인 | 위 입력·출력 범위 PASS |

Mac에서 Cargo 순차 실행, jobs 8·test threads 8·공용 `target/pr-review`를 사용했다. focused·전체 회귀와 보정 후 MCP는 기존 macOS 최소 글꼴 sandbox 및 새 작업 전용 캐시에서 실행했다. 이는 CLI 계약 검사이며 Visual Sweep을 최소 글꼴에서 실행했다는 뜻이 아니다.

원 head의 기존 CI 결과나 작성자의 자체 로컬 수치를 보정 head의 실행으로 재인용하지 않는다. 로컬 전체는 workspace·Mac 범위이고 GitHub Linux root-package archive 합계와 플랫폼/대상/base가 달라 단순히 같은 건수라고 쓰지 않는다.

## 조판 원칙·시각 검증 적용 여부

실제 diff와 호출 경로를 확인했다. 변경은 CLI/MCP 입력과 기존 편집 함수 연결이며 renderer/typeset/layout/paint·분할/페이지 수·Studio 표시·WASM setter 구현·직렬화 구현을 바꾸지 않는다. 따라서 독립 한컴 Print·Visual Sweep·Native/fresh WASM TSV·보호 문서·편람 gate는 이 변경의 필수 범위가 아니며 실행 통과로 표시하지 않는다. 저장 결과의 속성·소속·반복 저장 계약은 실제 CLI/MCP 및 독립 ZIP reader로 검증했다. Studio 사용자 액션이 아니므로 Undo/Redo·caret·repaint 라우터의 새 경로도 없다.

## 최신 base·CI·후행 기록

GitHub의 PR base metadata snapshot은 `00d09f77`이지만, 실제 보정 code CI의 merge commit `579d48fcfff7f80539e288c9b066589bf2e3de6e` 부모는 최신 `959300e6`과 보정 `24aa6c04`다. 실제 tree `d8eed19052fcda10e1bf53ac012858ec59995b24`가 로컬 최신 base merge-tree와 같아 오래된 snapshot만 검사한 것으로 오인하지 않는다. 충돌 없이 실제 최신 base와 통합되는지 확인했다.

code CI: [43447004453](https://github.com/edwardkim/rhwp/actions/runs/43447004453), 정확한 code SHA `24aa6c04bc0f632fc6aae2ee8db356fdd85a72d2`. 이 run의 Archive A~E 합계는 **13,285 PASS / 0 FAIL / 45 skipped**이며 Lint·CodeQL·Adapter·Proptest·최종 CI Impact Policy도 성공했다. 첫 외부 기여의 실행 승인 대기 상태만 해당 SHA로 확인해 승인했다. 테스트 실패를 재실행해 숨긴 것이 아니다.

이 기록을 작성하는 trailing head의 CI·재사용 판정은 별도로 확인한다. 문서 전용이라는 이유로 통과나 Full CI 재실행을 추정하지 않는다. 최종 병합은 별도 승인 범위이며, 이 기록 시점에 merge·이슈 close·후속 정리를 완료했다고 표시하지 않는다. 기여자 fork branch는 보존한다.

## 보존 증적

- [검증 정본](../assets/pr9052_picture_cli_20261113/validation.json): 원본 입력 SHA·producer, 보정 전후 실제 CLI/MCP 응답, 독립 ZIP 결과, 집중 반례·전체 회귀·게이트 명령과 원 로그 SHA. MCP 전체 184개 도구 목록은 범위 내 5개만 발췌하고 원 응답 해시를 보존했다.
- [보정 code CI 영수증](../assets/pr9052_picture_cli_20261113/code-ci.json): 정확한 head·run attempt·A~E job별 summary와 로그 해시, code head의 check 목록.
- [이번 PR의 오늘할일](../../orders/20261114.md): 보정과 검증 범위, 후행 head에서 확인할 조건.
