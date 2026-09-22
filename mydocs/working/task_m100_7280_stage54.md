# Task #7280 Stage 54 — R4 각주·미주 책임 묶음

- 이전: [Stage53](task_m100_7280_stage53.md).
- 승인 범위: [구현계획 §4.1](../plans/task_m100_7280_impl.md#41-r3r5-책임-묶음-진행으로-전환)의 R4 전체.
- 시작 head: `2613b32bb`, 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: **2026-09-23 R4 책임 묶음 구현·검증 완료**. R5/R6 및 최종 제출 게이트는 남아 있다.
- 검증 제품 SHA: `59256802c433aeb90142b9d419b9cca6ebcec184`.
  원격 push·PR·댓글은 수행하지 않았다.

## 책임과 호출 경계

`notes/footnotes`는 저장 줄 분할/소유 쪽 Query, 내용 측정, 본문 각주 등록을 분리한다.
본문 control 루프의 같은 위치에서 `register_body_footnote`를 호출하며, 기존 control-loop
`continue` 세 곳은 함수 `return`으로 치환한다. 완료 쪽/현재 쪽/다음 쪽 선택과 예약 순서는 같다.
표 fragment 큐는 R3의 `table/footnotes`와 `table/continuation`이 계속 소유한다.

`notes/reservation`은 TypesetState의 각주 예약 Query와 Command를 한 책임 아래 둔다.
projected 높이 조회는 `&self`, 실제 예약과 완료 쪽 갱신은 `&mut self`이며, 구분선·여백·
footer 공간 회수와 본문 예산 갱신을 기존 순서대로 수행한다. 전역 state 쓰기 제한은 R5다.

`notes/endnotes`의 구역 조정자는 원본 참조 순서로 prepare → paragraph → emit을 연결한다.
content/profile/types는 번호·원본 해석과 기존 정책 관측/미주별 carry를 소유한다.
format/measure는 동일 구성 컨텍스트 및 scratch 레이아웃을 쓰며 실제 페이지 상태를 바꾸지 않는다.
metrics의 호출자 로컬 accumulator 갱신과 진단 출력은 보존하므로 모든 함수를 순수 함수로
새로 주장하지 않는다.

미주 문단 루프의 initial-fit → rewind-tail → tail-fit → new-note-fit 계산은 각각
명시적 Input/Result로 분리했다. Input의 TypesetState 참조는 읽기 전용이다. 각 조회 사이의
배치·단 전환·offset/carry 갱신을 원래 위치에 유지해, 이후 조회가 변경된 상태를 읽는 순서도
보존한다. fit은 기존 수용 판정, emit은 확정된 분할/전체 배치를 담당한다.
기존 다수 플래그와 수치 정책은 유지하며 이번 분리를 정책 정당성 검증이나 결함 수정으로
보고하지 않는다. 큰 Query 입력을 더 작은 정책 모델로 바꾸는 설계는 동작 근거 없이 강행하지 않는다.

## 변경 전후 대조와 검증 결과

- `output/7280/r4-batch/structural-proof.json`: 이동한 함수 본문, 네 Query 계산,
  문단 Command 순서, 본문 각주 선택/예약 **98/98 일치**. formatter 차이와 함수 경계의
  continue→return·참조 전달만 정규화했다. 타입/호출 연결 검토 및 실행 검증을 대신하지 않는다.
- 첫 컴파일 및 lib Clippy 통과. 중간 import/가시성·Query 입력 타입 오류는 수정했다.
  `d49cddf4f`의 test build에서 기존 private 각주 helper import 누락을 발견했고,
  `59256802c`에서 기존 테스트 경로를 복원한 뒤 검증을 다시 수행했다.
- 테스트 원본·기대값·baseline·ignore·IR·public API는 변경하지 않는다.

아래 로그·JSON의 공통 위치는 `output/7280/r4-batch/`다. 검증 worktree
`/home/edward/mygithub/rhwp-review-7280-r3k`를 clean 상태 확인 후 재사용했다.
source-side 기존 테스트의 위치·module ID는 보존했고 새 source test는 추가하지 않았다.

| 검사 | 최종 결과와 증거 |
| --- | --- |
| fmt | 통과, `fmt.log` |
| 파생 suite 준비·고정 base manifest | 1,382 sources / 28 suites + 20 exceptions 통과, `prepare.log`, `manifest.log` |
| 고정 base unit-tier | 4,205 tests / 298 modules / cfg support 28, `unit-tier.log` |
| native Clippy `-D warnings` | 통과, `clippy-native.log` |
| 누적 집중 | **395 passed / 0 failed**, `nextest-focused.log`; 비선택 8,449건은 ignore 증가가 아님 |
| 전체 회귀 | **10,096 passed / 0 failed / 기존 제외 50**, `nextest-full.log` |
| 기준본과 회귀 대조 | 통과 이름 10,096개 동일, `regression-comparison.json` |
| 각주·미주·재개 관련 | 전체 실행에서 해당 이름 280건 통과 확인, `note-contracts.json`; 별도 재실행 아님 |
| Native 출력 | 12개 문서 **353쪽**의 dump-pages·RenderTree·SVG 동일, `native-comparison.json` |
| Docker WASM | fresh 빌드 성공, `build-wasm.log`; 9개 문서 **329쪽** RenderTree·SVG 동일 |
| 실제 PNG | 선택 **19쪽**의 전후 및 Native/WASM 간 동일, `visual-{native,wasm}-comparison.json` |
| 좌표·클리핑 후보 목록 | 9문서 × 13종 동일, `ledger-comparison.json` |

미주 측정의 상태 비변경은 기존 `test_measure_endnote_advance_side_effect_free`, 각주 fresh-page
큐는 `issue_5966_queued_table_footnote_completes_on_forced_fresh_page` 등 실제 실행 계약과
연결했다. 측정 함수만 읽기 전용으로 바꾸고 후속 상태 갱신 순서를 바꾼 것이 아니다.
본문 control 루프는 match 뒤에 별도 명령이 없으므로 추출 함수 return 뒤에도 다음 control로
진행한다. 문단 후 spacing/flow 정산은 control 루프 밖에서 원래대로 실행된다.

전체 회귀 실행 시간은 496.509초이며 자원 경합이 있어 성능 변화로 해석하지 않는다.
nextest 0.9.137/권장 0.9.140 및 observation 설정 경고는 기존과 같다.
Chrome launch 실패는 기준 square-body와 변경 night-guard에서 재시도로 해소했고 실패 로그도
보존했다. 이를 제품 결함이나 성공 실행으로 세지 않았다.

### 빌드·캡처 provenance

- `provenance.json` 및 각 `native/{base,head,head-standalone}/provenance.json`에 source,
  입력/PDF, CLI·WASM·JS 해시를 기록했다. baseline CLI/WASM은 기존 고정본 해시를 확인해
  재사용했지만 출력·PNG는 R4 디렉터리에 새로 생성했다.
- 캡처용 Native CLI는 동일 head의 nextest 준비 빌드 결과 `bin/head`다.
  SHA-256 `d0fcdba700deae2c9ffbfc22a68a9f35af9e232c92270b73c84f1316e0755096`.
- 명시적 `cargo build --profile release-test --bin rhwp` 결과는 `bin/head-standalone`에 별도
  보존했다. SHA-256 `60f303d337b3af5c04834dafec2f904c4e9bfb15cc7a43764fd95905b0d49227`.
  두 빌드의 dependency fingerprint가 달라 바이너리 동일성을 주장하지 않는다.
  **같은 353쪽의 pagination·기하·SVG를 추가 대조해 동일**함을 확인했다
  (`cli-build-comparison.json`). 캡처용 파일을 standalone 파일로 바꿔치기하지 않았다.
- fresh WASM SHA-256:
  `87cc6df6964390e6fa29453f29108907414e850aae2781d7b1cc7d6a9c8774b3`.
  `/home/edward/mygithub/rhwp-review-7280-r3k/pkg`에서 실제 Chrome WASM API로 내보냈다.
  Native SVG를 WASM 결과로 사용하지 않았으며 Native는 동일 웹폰트 정책 제공에만 사용했다.
- Host/Docker Rust 1.93.1, Node 24.15.0, Chrome 146.0.7680.31, 96 dpi.
  주 checkout의 Studio 배포용 `pkg`를 덮어쓰지 않았다.

## 시각 확인과 남은 차이

`targets.json`에 정확한 샘플/PDF/선택 쪽을 기록했다. 기준본과 변경본의 같은 페이지·영역에서
번호·본문/표·각주 영역·미주 제목과 그림·다단 이월을 대조했다. 대표 compare/standalone
overlay/review를 직접 열었으며 자동 점수를 시각 판정으로 대신하지 않는다.

| 대상 | 확인 쪽·경계 | 관측 |
| --- | --- | --- |
| `deferred-picture` 연구 보고서 | 75/76, 90/91: 본문·표 각주와 뒤 문단 | 각주 101/102, 표 뒤 각주 142의 다음 쪽 소유 및 후속 본문 보존. PDF와 글꼴·표 외곽 세부 차이는 기존과 동일 |
| `endnote-2022-09` | 9/12: 미주 시작·구분선·다단 하단 | 번호·구분선·그림과 풀이 흐름 보존. 기존 수식/글꼴 모양 차이 잔존 |
| `endnote-between20` | 21/22: 큰 미주 간격·단 이월 | 기존 PDF 대비 줄 소유/세로 위치 차이도 보존. 22쪽 상단 일부 줄의 이전 쪽 이월과 그림·문항 위치 차이를 신규 회귀로 분류하지 않음 |
| `endnote-zero` | 15/16: 0 간격·그림/수식 | 새 중복·누락·페이지 이동 없음. 기존 수식/폰트 차이 잔존 |
| `endnote-no-separator` | 15/16: 구분선 없음·큰 여백 | 기존 간격과 번호 순서 보존. 16쪽 오른쪽 하단의 PDF 대비 줄 이월 차이도 기존 상태이며 개선으로 보고하지 않음 |
| R2/R3 대조군 | square-host/body/table, night-guard 및 TAC 3종 | 앞선 책임 묶음의 전체 Native 출력 동일; PDF 대응 7쪽 PNG도 동일 |

대표 산출 경로는 다음과 같다. 각 디렉터리에 compare·overlay·review 세 종류가 있다.

- `native/head/deferred-picture/deferred-picture/compare/compare_076.png`
- `native/head/deferred-picture/deferred-picture/review/review_090.png`
- `wasm/head/deferred-picture/deferred-picture/overlay/overlay_076.png`
- `native/head/endnote-2022-09/endnote-2022-09/compare/compare_012.png`
- `native/head/endnote-between20/endnote-between20/overlay/overlay_022.png`
- `wasm/head/endnote-zero/endnote-zero/review/review_016.png`
- `wasm/head/endnote-no-separator/endnote-no-separator/review/review_016.png`

**판정: 리팩토링의 동작 보존 충족. 한컴 피델리티 개선·기존 예외의 정당성 승인과 구분한다.**
모든 353쪽의 시각 전수 판독을 했다는 주장은 하지 않는다. 전체 기하/SVG와 후보 목록 대조,
선택 19쪽 PNG 동일성, 위 대표 영향 경계의 직접 확인을 결합한 증거다.
ledger의 한 쪽 문서 SVG 카운터는 비번호 파일명을 0으로 집계하는 기존 한계가 있어 총 쪽수
근거로 쓰지 않았다. 353쪽은 실제 dump-pages 및 export 파일 집합으로 확인했다.

## 재현 명령과 다음 범위

`validate.sh`는 고정 review worktree에서 prepare → fmt → 고정 base 정책 → native Clippy →
누적 집중 → 전체 nextest → standalone CLI build를 순차 수행한다. 주요 명령은 다음과 같다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
CARGO_BUILD_JOBS=4 cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --tests --test-threads 8 --no-fail-fast
docker compose -p rhwp --env-file .env.docker run --rm --no-deps \
  -e CARGO_BUILD_JOBS=4 -e BINARYEN_CORES=4 wasm
# 주 checkout: variant=base/head, mode=ledger/native/wasm
node output/7280/r4-batch/native.mjs head
node output/7280/r4-batch/visual.mjs head wasm
node output/7280/r4-batch/compare-native.mjs
node output/7280/r4-batch/compare-visual.mjs wasm
```

`typeset.rs`는 R4 시작 20,455줄에서 12,035줄로 줄었다. 입출력 타입·연결 코드가 늘었으므로
전체 LOC/CC 감소를 주장하지 않는다. R4의 각주/미주 책임 분리는 완료했으며, **다음은 승인된
R5 상태 소유·구역 조정 전체**다. R6 구조 정본·기여자 관리 지도·재계측과 최종 PR CI의
WASM/workspace Clippy·workspace build·Native Skia 등 제출 게이트는 아직 완료하지 않았다.
