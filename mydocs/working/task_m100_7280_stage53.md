# Task #7280 Stage 53 — R3 책임 묶음 구현·검증

- 이전: [Stage52](task_m100_7280_stage52.md).
- 승인 계획: [구현계획 §4.1](../plans/task_m100_7280_impl.md#41-r3r5-책임-묶음-진행으로-전환).
- 시작 head: `fad75d8e8`, 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: **2026-09-23 R3 책임 묶음 구현·검증 완료**. 작은 helper별 승인·문서를 반복하지 않고
  R3 기록을 여기에 누적했다. R4/R5 완료·최종 제출 게이트 통과를 뜻하지 않는다.

## 책임 분리

- `table/block`: 전체 표 배치 진입 → 분할 준비 → 재개 실행의 조정자.
  `entry`는 기존 조기 배치·반환 순서, `whole_fit`은 상태 변경 없는 전체 수용 조회,
  `prepare`는 분할용 행 기하·각주 예약·호스트 선행 배치와 context 준비를 담당한다.
  `HostPlacementConstraint`는 전체/분할 배치가 공유하던 점유 밴드 제약을 읽기만 한다.
- `table/scan`: 원본 표와 컷용 행 도메인을 유지한다. `runner`가 물리 꼬리 밴드 및
  행 반복을 소유하고, rowspan 블록/일반 행 step은 읽기 전용 페이지 관측과 기존 Query를
  소비해 컷·누적 높이·다음 행을 반환한다. `BlockTableRowScan`과 `BlockRowScanVars`도 이 도메인에 둔다.
- `table/continuation`: 커서·준비 상태·shadow flow의 수명과 기존 crate-visible 재개 API를 소유한다.
  `fragment`는 시작 컷을 한 번 복제한 뒤 예산 준비 → 읽기 전용 scan/refit → emit을 호출한다.
  `emit`이 기존 순서로 항목·각주·커서·다음 쪽을 확정하며, native drain과 WASM job은
  동일한 fragment 구현을 호출한다. facade 재수출로 기존 API와 private 테스트 경로를 유지한다.
- `table/footnotes`: 표 조각에 붙은 각주 큐의 연결을 표 도메인으로 이동했다.
  일반 각주/미주 알고리즘의 전체 책임 정리는 R4이고, TypesetState 외부 직접 쓰기의 최종
  캡슐화는 R5다. 이행 중인 직접 쓰기를 최종 상태 분리 완료로 표시하지 않는다.

기존 조건·숫자·예외·판정 순서·측정 API·테스트 기대값은 변경하지 않는다.
기존 예외를 올바른 조판 규칙으로 새로 승인하거나 결함을 고쳤다는 주장은 하지 않는다.

## 호출·상태 경계 점검

`typeset_block_table_inner → prepare_block_table_entry → prepare_block_table_continuation`
순서다. 조기 배치된 표는 준비/재개로 진입하지 않는다. whole-fit의 저장 프레임 조회 이후에만
기존 흐름 위치를 갱신한다. 준비 완료 후 placeholder로 페이지 상태를 옮기는 시점과
`suspend_before_drain` 반환 시점은 유지한다.

재개 경로는 `context.step → step_block_table_fragment → prepare_table_fragment_budget →
scan_table_fragment → emit_table_fragment`다. 시작/끝 컷과 행 도메인을 재추측하지 않고
기존 scanner 결과를 그대로 받는다. 각주 refit도 동일 scanner를 호출하며, 예산을 변경한
재계산과 실제 예약·배치를 서로 다른 단계로 둔다. 물리 tail 밴드·패딩·반복 제목행·캡션·
각주 예약의 기존 계산 및 마지막 유닛 소비 후 종료 순서를 보존한다.

## 중간 정적·컴파일 검사

- `output/7280/r3-batch/prove.mjs`, `structural-proof.json`: 18개 경계 대조 통과.
  진입/whole-fit/분할 준비, budget/scan/refit/emit, rowspan/일반 행 step,
  native drain, 기존 재개 API 및 표 각주 함수의 계산·분기·명령 본문을 비교했다.
  주석·공백·포맷용 쉼표와 명시된 함수 연결 치환을 제외한다. 새 연결/불변 alias의 수명과
  반복 dispatch는 별도 코드 검토·실행 대상이며 정적 비교가 전체 동등성 증명은 아니다.
- 중간 `cargo check --locked --lib --target-dir target/pr-review` 통과.
  분리 중 누락된 closure 호출 표기와 `fmt` 입력은 컴파일 오류로 검출해 수정했다.
- source-side 테스트·integration 원본·기준값·ignore 변경 없음.
- 구현 커밋 `614f639e5c`, lint 정리 `f70c5dfc5`.
  아래 고정 제품 head의 검증 결과를 따르며, 중간 컴파일만으로 종료하지 않는다.

기존 clean review worktree `rhwp-review-7280-r3k`와 공유 target을 재사용한다.
이전 로그/산출물은 보존하며 별도 대용량 worktree를 늘리지 않는다. 원격 push·PR·댓글은 하지 않는다.

## 고정 head 검사

제품 head는 `f70c5dfc5a9e288e5d37574b227d8831d9044e8e`다.
검증 target은 `/home/edward/mygithub/rhwp/target/pr-review`이고 모든 Host Cargo 실행은 순차로 진행했다.
증적 루트는 `output/7280/r3-batch/`다.

| 검사 | 결과 / 증적 |
| --- | --- |
| fmt | 통과, `fmt.log`, `fmt-final.log` |
| integration manifest, 고정 baseline 비교 | 통과: 1,382 sources / 5,965 static attrs / 28 suites + 20 exceptions = 48 targets; `manifest-final.log` |
| source-side tier, 고정 baseline 비교 | 통과: 4,205 tests / 298 modules / cfg support 28; `unit-tier-final.log` |
| native Clippy `-D warnings` | 통과, `clippy-native-final.log`; 중간 doc blank line·needless return 오류는 최종 head에서 수정 |
| 누적 focused | 395건 통과, `nextest-focused.log`; Stage52 PASS 이름과 일치 |
| 재개·편집 경계 추가 검사 | `wasm_api::tests::issue2424_` 6건 통과, `nextest-resumable.log` |
| 전체 nextest | **10,096건 통과 / 실패 0건 / 기존 제외 50건**, `nextest-full.log` |

추가 6건은 HWP/HWPX의 shadow page-count 보존, 편집으로 인한 이전 job 무효화,
마지막 fragment 전 비공개·완료 후 publish, delete 재개를 검사한다. Native test에서 WASM API
계약을 실행한 것이며 실제 Studio UI의 편집 시나리오 전수를 의미하지 않는다.

정책 base는 기존 동작 baseline `722fb38af…`로 고정했다. 최신 원격 PR base를 고정하는
제출 전 정책 검사·WASM/workspace Clippy·workspace build·Native Skia 전체 게이트는 별도다.
이번 검증에서 기존 테스트 원본·Cargo·baseline·ignore는 변경하지 않았다.

전체 회귀 exit 0: 빌드 6분 32초, 검사 457.785초, 78 binaries,
run ID `49a0f990-9239-4f00-99a6-3f796358f218`.
`compare-regression.mjs` / `regression-comparison.json`에서 Stage1 baseline의
10,096개 PASS 이름과 정확히 일치했다. `issue_2063` 대형 표 계약은 280.289초에 통과했으며,
baseline의 303.645초 실행에도 SLOW 경고가 있었다. 시간 차이를 성능 개선으로 주장하지 않는다.

## 출력 보존 검사

`build-provenance.json`에 source SHA와 실제 실행 CLI/JS/WASM hash를 보존했다.
baseline은 Stage41에서 빌드한 CLI와 baseline worktree의 WASM을 source/hash 대조 후 재사용하되,
이번 비교 출력은 새로 생성했다. head는 고정 review worktree에서 Native CLI와 Docker WASM을
실제 컴파일했다. Native 2분 44초, Docker WASM/wasm-opt 8분 31초, 양쪽 exit 0이다.
주 checkout의 `pkg`는 변경하지 않았다.
Sweep manifest의 Git HEAD는 캡처 도구 checkout을 가리킨다. baseline/head renderer의 실제
source SHA는 binary/package hash와 고정 worktree를 연결한 `build-provenance.json`으로 구분한다.

- baseline CLI: `7191580d6d147d0e470a21d68bb6a85cf7d734828c97ee88be065933c587bfbe`.
- head CLI: `70633e755e7555a1a15703fe9ce5e9ec9d480a374bff71d78c13ed96d6d38231`.
- baseline WASM: `70f6cda1825458bc5246f360cc635259154613ddb7a91fc4f64c2967a0fb585e`.
- head WASM: `f62fa3ff5a70059ff28b5ee9c0d05f03da54bb8414a4afd08703c7c9e264e92f`.

`targets.json`의 8개 입력/262쪽에서 baseline/head의 `dump-pages` 전체 JSON(소유·컷 포함),
RenderTree 전체 JSON 및 SVG 원문이 일치했다(`native-comparison.json`). PDF가 있는 5개 쌍은
양쪽에 전수 text/layout ledger를 만들었다. 원본/PDF hash는 `native/{base,head}/provenance.json`이다.
`ledger-comparison.json`에서는 각 입력의 후보 TSV와 page-count ledger 13개씩, 총 65개 파일이
원문 일치한다. 새 검출 후보가 없다는 보조 증거이며 기존 후보들이 정상 조판이라는 뜻은 아니다.

Native 선택 16쪽의 compare·standalone overlay·review도 새로 생성했으며 PNG 원문이 baseline과
일치한다(`visual-native-comparison.json`). 직접 판독 기록과 fresh WASM 대조는 아래에 누적한다.
WASM은 실제 Chrome의 `renderPageSvg`·`getPageRenderTree`로 내보내며 Native 트리를 대신 쓰지 않는다.

### Native 직접 판독

선택 16쪽의 head review를 직접 열고 표 외곽·내용·후속 문단·각주·그림 경계를 확인했다.
표 조각/각주가 연결되는 90→91쪽은 이번에 새로 만든 baseline review도 직접 대조했다.
아래 PDF 차이는 baseline에서도 동일하게 남아 있는 차이이며, 이번 구조 변경의 개선/회귀나
한컴과의 전체 피델리티 통과로 바꾸어 보고하지 않는다.

| 입력 key / 실제 쪽 | 확인 경계 | PDF 보조 일치율(%) / 보존된 차이 |
| --- | --- | --- |
| square-host / 1 | 좁은 본문→전폭 복귀 | 9.77 / 글꼴·굵기 |
| square-body / 1 | 그림 왼쪽 문단과 뒤 본문 | 46.43 / 제목·글꼴; 축소 이미지 fixture |
| square-table / 5,6,7,8 | 표 앞/옆/아래 본문과 다음 쪽 꼬리 줄 | 30.78,10.40,19.19,12.09 / 글꼴, 특히 7쪽 세로 간격 |
| night-guard / 1 | float 표 뒤 본문 표 하단 | 11.45 / PDF의 하단 붙임 문구가 양쪽 rhwp에서 보이지 않음 |
| deferred-picture / 75,76 | 그림/본문/각주, 표의 첫 조각과 하단 각주 | 10.87,25.36 / 그림 크기·표 행 기하·글꼴의 기존 차이 |
| deferred-picture / 90,91 | 표27의 분할·끝행, 각주141→142 및 뒤 본문 | 7.81,7.63 / 본문 줄바꿈·글꼴·일부 세로 위치 |
| deferred-picture / 126,127,155,156,182 | 표·각주 보존, 그림56/64/67·캡션과 후속 내용 | 9.59,19.48,8.89,14.15,6.83 / 글머리표·그림 크기·182쪽 세로 위치 |

개별 경로는 `output/7280/r3-batch/native/head/<key>/<key>/` 아래
`compare/compare_NNN.png`, `overlay/overlay_NNN.png`, `review/review_NNN.png`다.
WASM은 `native` 대신 `wasm` 아래 동일 구조다. `visual-*-comparison.json`에 각 절대 경로가 있다.
대표 표 연속 경계는 `deferred-picture/deferred-picture/review/review_090.png`와
`review_091.png`다. 보조 일치율은 픽셀 도구의 값이지 사람 판정 정확도가 아니다.

PDF 출처/원문 대응은 Stage41에서 확인한 #4090 MCP PrintToPDFEx, #6309/#6314·#6572 검토,
#3820 Stage9 한컴 2020 기록을 계승하며 동일 hash를 확인했다. 이는 리팩토링의 동작 보존
검증이고 기존 PDF 차이 해결의 시각 승인 요청은 아니다. 미선택 페이지의 직접 판독,
모든 분기 조합, Studio Canvas·편집 후 재조판 전수를 입증하지 않는다.

Chrome launch SIGTRAP이 Native의 마지막 입력과 WASM 캡처 중 발생했다. 실패 로그를 남기고
동일 SHA/입력/패키지 provenance를 확인하는 `--resume`으로 이번 실행의 완료 checkpoint만
이어받았다. 이전 단계 이미지를 재사용하지 않는다. Native 재시도는 완료했고, WASM은 자식 프로세스
CPU affinity를 `0-3`으로 제한한 재시도에서 exit 0으로 완료했다. 제한이 Chrome 오류의 근본 원인을
해결했다고 단정하지 않는다. 제품 조판 실패나 제품 수정으로 분류하지 않는다.

### Fresh WASM 대조

`visual-wasm-head-resume2.log`와 `visual-wasm-comparison.json`: 5개 입력/238쪽 raw SVG와
WASM RenderTree가 baseline/head 사이에서 일치한다. 선택 16쪽 PNG도 baseline/head 및
head Native/WASM 사이에서 일치했다. head의 square-host/body·night-guard 개별 review,
square-table/deferred-picture contact sheet와 표 연속 경계인 90·91쪽 개별 review를 직접 열었다.
Native에서 확인한 소유·외곽·뒤 문단·각주 경계와 동일하며, 기존 PDF 차이도 그대로다.
Chrome 실패를 제품 회귀로 세지 않았고, 실패 실행을 성공 결과로 재사용하지 않았다.

### 재현 명령

```bash
# 고정 review worktree: 정책 base는 위 baseline SHA
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --lib \
  -E 'test(wasm_api::tests::issue2424_)' --test-threads 4 --no-fail-fast
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --tests --test-threads 8 --no-fail-fast
docker compose -p rhwp --env-file .env.docker run --rm --no-deps \
  -e CARGO_BUILD_JOBS=4 -e BINARYEN_CORES=4 wasm

# 주 checkout: 캡처에는 bin/{base,head}와 각 고정 worktree의 pkg 사용
node output/7280/r3-batch/prove.mjs
node output/7280/r3-batch/native.mjs base
node output/7280/r3-batch/native.mjs head
node output/7280/r3-batch/compare-native.mjs
# variant=base/head, mode=ledger/native/wasm 각각 실행
node output/7280/r3-batch/visual.mjs head native
node output/7280/r3-batch/compare-visual.mjs native
node output/7280/r3-batch/compare-visual.mjs wasm
```

Host/Docker Rust 1.93.1, Node 24.15.0, Chrome 146.0.7680.31, 96 dpi, layer SVG와 동일 webfont
정책이다. nextest 0.9.137의 권장 버전 0.9.140 및 observation 설정 경고는 baseline과 같다.
작업별 테스트 시간은 자원 경합이 달라 성능 변화로 해석하지 않는다.

## 남은 범위

R3의 기존 회귀 계약·대표 Native/fresh WASM 출력 보존은 **충족**으로 판정한다.
`typeset.rs`는 R3 시작의 25,933줄에서 20,455줄로 줄었지만, 명시적 입력/결과 타입과
연결 코드는 추가됐으므로 전체 LOC 또는 CC 감소를 주장하지 않는다. 핵심 산출은 표 진입·
행 스캔·재개 상태·조각 예산/scan/emit의 책임 경계이며, 전역 상태 쓰기 제한은 아직 R5다.
검증 뒤 제품 코드는 변경하지 않고 이 결과 문서와 계획의 상태만 커밋한다.

다음 R4는 각주/미주 예약·준비·측정·배치 전체, R5는 상태 소유·구역 조정 전체다.
이번 기록에서는 R4/R5를 구현하지 않았으며, read-only 위치 조사만 했다.
R6 구조 정본·기여자 관리 지도·재계측 및 최종 제출 게이트도 남는다.
작은 helper별 승인을 다시 요청하지 않으며, 기존 규칙의 의미 변경이 필요할 때만 별도 판단한다.
