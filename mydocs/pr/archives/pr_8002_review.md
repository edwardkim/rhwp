# PR #8002 검토 — CRLF 정규화와 Studio 커서·Undo 보존

## 최종 판정

**메인터너 보정 후 승인** — [#8001](https://github.com/edwardkim/rhwp/issues/8001) 해결 범위의
필수 로컬 검증과 통합 code head CI를 완료했다. 원 코어 보정 뒤 남은 Studio 길이 소비 결함을
별도 보정했고 원 contributor head 자체와 명시한 통합 head를 구분한다.
남은 원격 조건은 문서 trailing CI·mergeability 확인 및 승인된 정상 merge·후속 처리다.

## 접수와 적용 범위

- 기존 기여자 `donggyun112`, 원 [PR #8002](https://github.com/edwardkim/rhwp/pull/8002), base devel.
- 원 head `5769c4e7bed9206403cc4f6c052d0ceaf6bb96ae`, 5개 commit을 원 author·`-x` SHA 보존해 누적했다.
- reviewer `jangster77`을 local fetch·체리픽 전에 지정하고 API로 재확인했다.
  접수 당시 상세 metadata가 지정에 앞섰던 점은 숨기지 않으며 새 지침은 최소 분류 뒤 먼저 지정하도록 명확히 했다.
- 일반 댓글·review·inline comment·GraphQL thread는 모두 0건이었다. 원 본문의 미실행/실패 이력은
  현재 통합 검증으로 자동 승격하지 않았으며 실제 코드·호출 경로·정식 검사를 실행했다.
- 본문·일반/중첩 셀 삽입, 범위·첫째·전체·n번째 치환, CLI 응답·저장을 검토했다.
  newline을 거부하는 local/deferred API의 제한은 유지한다. 이미 잘린 저장본의 복구를 주장하지 않는다.

## 공통 결과와 추가 소비 지점 보정

`normalize_paragraph_newlines`가 CRLF/단독 CR을 기존 LF로 바꾼 뒤, 본문·일반/중첩 셀의
문단 수정·reflow·커서·event 길이와 CLI text/insertedChars 응답이 같은 문자열을 소비한다.
정규화한 LF는 기존 문단 변형과 저장 경로에 전달된다.

원 코어가 실제 길이를 바꾸는데 Studio `InsertTextCommand`가 정규화 전 문자열 길이로
캐럿·서식·repaint·Undo 범위를 정하던 소비 지점을 찾았다. 메인터너 보정
`0cdfadcca4d7d858238120939a1a79d18ede1180`은 공통 `normalizeTextNewlines`를 명령 생성과
textarea 입력 경계에서 사용한다. 특정 문서·좌표 예외나 결과 clamp를 넣지 않았다.

| 실행 경계 | 수정 전 | 수정 후 |
| --- | --- | --- |
| 실제 기준 CLI CRLF 삽입 | 성공·insertedChars 14지만 저장 후 첫 줄만 남음 | normalized 12자·모든 줄·기존 suffix 보존 |
| 실제 raw WASM CRLF/CR | 본문·셀·중첩 셀 6개 저장 보존 assertion FAIL, LF 3개 대조군 PASS | 9개 조합에서 직접 API caret·HWP 재열기 PASS |
| fresh core + 원래 Studio 명령 | 51개 assertion 중 14 FAIL, caret 8·Undo `앞`만 남음 | caret 6·Undo `앞뒤😀`·Redo 전체 보존, 51 PASS |
| 실제 main InputHandler operation | 명령/캐럿·Undo 소비 불일치 | operation·performUndo/Redo·canUndo PASS |

정식 Rust는 원 4개와 추가 LF 동등성 1개, 총 5개다.
[issue_8001_crlf_text_roundtrip.rs](../../../tests/cases/issue_8001_crlf_text_roundtrip.rs)는
CRLF/CR/LF의 전체 Document Debug·저장 LineSeg·이벤트 동등성과 raw/adapter HWP·HWPX 저장 재열기를 검사한다.
[Studio 단위](../../../rhwp-studio/tests/crlf-insert-history.test.ts)와
[실제 Chrome E2E](../../../rhwp-studio/e2e/crlf-insert-history.test.mjs)는 독립 원문 suffix·scalar caret·Undo/Redo를 보호한다.
Node 모형 검사는 본문·일반/중첩 셀 및 머리말·꼬리말·각주 각 3개 newline 형태를 확인한다.
HF/FN의 직접 코어 API 전체까지 실브라우저에서 검증했다고 확대하지 않는다.

## 기존 표 정책 회귀의 입력 정합성

#7288의 `cell_unit_row_is_atomic`, `declared_frame_overflow_allowance`, `table_page_break_split_policy`
3개 합성 모듈 7검사를 확인했다. 원 CR은 강제 개행이 아니라 무시되는 자동 줄바꿈 표지로 잘못 쓰이고 있었다.
이 표지를 제거해 원래의 자동 줄바꿈 계약을 복구하고, CELL 대조군은 각 행은 한 쪽에 맞되 두 행 전체는
맞지 않게 구성했다. NONE oversized 예외만으로 약한 대조군이 통과하지 않도록 행 소유·첫 행의 원자성·
실제 분할 시 행 내부 이월을 함께 검사했다. 기존 assertion·overflow 허용치를 삭제·완화하지 않았다.
합성 정책 계약의 통과를 독립 한컴 문서와 물리 배치가 일치한다는 증거로 바꾸지 않는다.

## 조판 영향과 시각 범위

조판 값·수용 조건을 바꾸는 변경 없이 같은 정규화 LF를 기존 reflow·저장 경로에 전달한다.
동일 seed의 **기준 devel LF와 후보 LF/CRLF/CR HWP 저장본 4개가 모두 4096바이트,
SHA-256 `409578512489409f92cc6f1cb88a060fbff7c5a0216ac37eef032f838c060812`로 동일**했다.
전체 문서·줄 구성·이벤트 동등성도 정식 검사로 보호했다. renderer 파일 경로만으로 비해당을 판단하지 않고
실제 입력 결과와 소비 경로를 대조했다. 새로운 실물 배치 fixture·px golden·수용 조건은 추가하지 않았다.
이 입력 보존 범위의 Visual Sweep은 비해당이며 독립 Print의 페이지 정확도나 시각 개선을 주장하지 않는다.

## 로컬·통합 검증

정확한 통합 code head는 `3868f40b99090d921bfa8357f2c2cf8453e6b57f`,
기준 devel은 `4296fa045df2743a0eaa1b0cc71b2a3c1dd90bb6`다.
실제 fetch에서 기준 head가 변하지 않아 불필요한 rebase를 하지 않았다.

| 검증 | 실제 결과 |
| --- | --- |
| 전체 nextest release-test | 10,831 PASS / 0 FAIL / 기존 45 SKIP, 실행 981.211초 |
| focused Rust | #7999 3개·#8001 5개·기존 표 정책 7개, 총 15 PASS |
| fmt·Native/WASM Clippy·workspace build·all-target Clippy | 모두 exit 0 |
| manifest·unit-tier | 고정 base SHA 기준 exit 0 |
| Studio 단위·타입·production build | 2,328 PASS / 기존 2 SKIP, 타입·build exit 0 |
| editor 계약 | 전체 36 PASS, 실패 이력과 재확인 조건은 아래에 기록 |
| 실제 Native CLI·stdio MCP | CLI 실행/dry-run 8개 및 session/stateless 보존 PASS |
| 실제 fresh WASM Chrome E2E | 51 assertion PASS, raw API와 실제 operation·Undo/Redo 포함 |
| E2E manifest·문서 | 162파일/162행, 링크·공백 PASS, metadata 기존 16오류 동일·신규 0 |

명령은 `CARGO_BUILD_JOBS=8 cargo nextest run --locked --cargo-profile release-test
--target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast`다.
공용 target을 순차 재사용했다. 제품 보정 `0cdfadcca4d7d858238120939a1a79d18ede1180` 이후
제품 입력 1,887파일의 SHA-256이 같고 후속 변경은 검사·문서뿐임을 확인했다.

초기 editor 고병렬 실행에서는 100ms notifySaved handshake fallback 1검사가 실패했다.
제품·기대값·timeout을 바꾸지 않고 해당 3검사와 전체 36검사를 concurrency 2로 실행해 통과했다.
새 Rust 검사의 초기 Serialize 타입 오류·stale harness에 따른 0개 실행과 초기 브라우저 locator 오류는
보정 후 다시 실행했으며 결함 재현 FAIL 또는 성공 검증으로 세지 않는다.

WASM은 Docker 없는 승인된 호스트 진단 경로에서 `scripts/wasm-pack-locked.sh --target web
--out-dir pkg --no-opt`, jobs 8·release codegen-units 16으로 fresh 빌드했다.
WASM SHA-256은 `96acd20dace8251809029e78974b83a42310bda38e5a49970edc3663a8c706f4`이며
pkg/public 쌍의 바이트 동일성을 확인했다. 실제 Windows·Docker 최적화 배포 검증으로 확대하지 않는다.

## 실제 GitHub CI

통합 [PR #8004](https://github.com/edwardkim/rhwp/pull/8004)의 위 code head에서
[Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39059657983) · [CI](https://github.com/edwardkim/rhwp/actions/runs/39059658110) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39059658256) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39059657994) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39059658020) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39059657909)가 모두 실제 완료·성공했다.
Native Skia·프런트엔드 패키지·lint·Archive A/B/C/D 회귀와 최종 Build & Test도 성공했다.
전체 check 35개 결과는 neutral 1개, skipped 4개, success 30개이며 미완료·실패 0개다.
해당 PR merge ref의 열린 CodeQL alert도 0건이었다. skipped를 실제 검사 실행 성공으로 바꾸어 적지 않았다.

아직 실행 전인 문서 trailing head의 최신 fast-pass·required check·mergeability 확인,
정상 merge·이슈 종료·댓글·정리를 완료로 쓰지 않는다. 사용자는 PR·정상 merge·후속 처리를 승인했다.

## 증적·후속 처리

실행 로그·producer 및 입력 지문·실제 CLI/MCP/브라우저 전후 JSON·CI API 영수증은 검토 중
ignored output에 보관하고, 후속 정리 때 `/home/tsjang/rhwp-pr-evidence/pr-8004/`에 해시 확인해 보존한다.
원본 검사·기존 입력/PDF·contributor fork·공용 target은 보존한다.

## Merge 후 contributor PR comment 계획

정상 merge·devel 포함·duration 결과·관련 이슈 상태를 확인한 뒤 해당 원 PR에 한국어 존댓말로
원 기여 감사, author/원 SHA 보존, 아래 적용 기록의 보정 여부·수정 전후 실행 증거,
실제 CI와 개별 검토 링크·남은 범위를 게시한다. 원 PR을 직접 merge한 것으로 쓰지 않고
통합 PR로 반영한 뒤 close함을 명시한다. UTF-8 body-file로 게시하고 API로 실제 줄바꿈·한글·BOM을 확인한다.
동일 merge·증거 댓글은 중복 게시하지 않는다. contributor fork는 변경·삭제하지 않는다.
