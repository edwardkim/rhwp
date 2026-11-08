# PR #8000 검토 — 겹치지 않는 원문 검색·치환 범위

## 최종 판정

**승인** — [#7999](https://github.com/edwardkim/rhwp/issues/7999) 해결 범위의
로컬 검증과 명시한 통합 code head CI를 완료했다. 원 검색 알고리즘은 보정 없이 수용한다.
통합 범위의 CRLF Studio 보정은 [#8002 검토](pr_8002_review.md)로 구분한다.
남은 원격 조건은 같은 PR의 문서 trailing CI·mergeability 확인 및 승인된 정상 merge·후속 처리다.

## 접수와 범위

- 기존 기여자 `donggyun112`, 원 [PR #8000](https://github.com/edwardkim/rhwp/pull/8000), base devel.
- 원 head `01d775800d0473205899dd24f3adcd39b57c9707`를 author와 `-x` SHA를 보존해
  `5ea1b0d24755060b3bd7981c6c9e8ff4977b6cc0`에 체리픽했다. 정확한 적용 SHA는 적용 기록에도 연결한다.
- reviewer `jangster77`을 local fetch·체리픽 전에 지정했고 PR 제출 전 API로 재확인했다.
  접수 당시 상세 metadata 조회가 배정에 앞섰던 점은 이 기록에 남긴다. 보완한 지침은
  최소 분류 뒤 지정·API 확인을 먼저 완료하도록 한다.
- 이슈 댓글·review·inline comment·GraphQL thread는 모두 0건이었다. 원 head와 신규 댓글 여부를 제출 전 재확인했다.
- API 인자·응답 schema는 유지한다. `AAAA`에서 `AA` 검색 주소가 0·1·2에서 **0·2**가 되는 것은
  원문 비중복 범위 계약의 의도된 동작 변경이다. Unicode full casefold 추가로 주장하지 않는다.

## 구현 주장·독립 기대값·검사

`find_in_text`가 왼쪽부터 선택한 겹치지 않는 원문 scalar 범위를 `find_matches`가 각 컨테이너에서 수집하고,
search·grep·n번째·전체 replace가 동일 주소를 소비한다. 소문자화 확장에서는 변환 후 길이가 아니라
원문 시작·끝으로 다음 허용 범위를 결정한다. 본문·셀·글상자·수식의 범위는 서로 독립적이다.

| 독립 기대값 | 수정 전 관측 | 수정 후 검사 |
| --- | --- | --- |
| `검토......완료 원문 끝.`의 `...`→`…`는 2건, 뒤 원문 보존 | 실제 기준 CLI가 성공·4건을 보고하고 저장본은 `검토…원문 끝.` | 2건·`검토……완료 원문 끝.`, dry-run·실제 MCP session/save도 일치 |
| `AAAA끝`의 `AA` 주소는 0·2, 길이 증가 치환 후 suffix 유지 | 겹치는 범위가 후속 텍스트를 침범 | search·grep·n번째·전체 치환·저장 재열기 PASS |
| `İ`의 소문자화 확장과 중간 확장 검색도 원문 범위를 공유 | 원문 길이와 변환 길이를 혼동할 수 있는 경계 | 완전/부분 확장 및 일반 대소문자 경계 PASS |
| 컨테이너별 주소·수식 내용 독립 | 서로 다른 문단·컨테이너를 한 범위로 합치면 안 됨 | 10건 치환, 본문/셀/글상자 suffix 4개·수식 suffix 보존 PASS |

정식 검사 [issue_7999_nonoverlapping_replacement.rs](../../../tests/cases/issue_7999_nonoverlapping_replacement.rs)의
3개 테스트와 실제 CLI·stdio MCP·fresh WASM 검색/저장 대조군을 확인했다.
IR 성공 응답·verify만으로 끝내지 않고 원문에서 독립적으로 정한 뒤 단어·emoji·순서를 검사했다.

## 조판 원칙 판정

검사 대상은 원문 범위·내용 보존 API 계약이다. 조판 좌표·메트릭·저장 LineSeg 수용 조건·renderer/
composer/pagination 계산과 golden을 바꾸지 않는다. 공통 입력 정규화의 실제 LF 소비 동등성 근거는
[#8002 검토](pr_8002_review.md#조판-영향과-시각-범위)에 있다. 합성 컨테이너 입력의 물리 배치·독립 한컴 Print
일치를 주장하지 않고 새 물리 배치 fixture·px golden을 추가하지 않았다. 이 제한된 범위의 Visual Sweep은 비해당이다.

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
