# Task #7280 Stage 30 — R2ab 표 배치 뒤 각주 연결 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2aa](task_m100_7280_stage29.md), 시작 head `75b37eb89`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2ab 구현·고정 제품 SHA 집중 검증 완료. R2 전체/PR 준비 완료가 아니다.

## 1. 책임과 보존 범위

일반 표 문단 루프와 지연 표 배치의 중복 각주 연결을 `typeset/notes.rs`로 옮긴다.
이는 R2의 컨트롤 배치 후 연결 경계이며 R4의 각주 높이/예약/쪽 배분 알고리즘 분리가 아니다.
`notes`는 큐 소유권을 한 번 확인하고 직접 셀 각주를 순서대로 등록한다.
`state::has_fragment_queued_table_footnotes`는 기존 집합의 membership만 읽는다.
부모 엔진은 기존 등록 메서드를 callback으로 제공한다. 새로운 trait·public API·IR은 없다.

- 표 배치 완료 → 큐 소유권 확인 → cell/paragraph/control 순서의 직접 각주 등록을 보존한다.
  중첩 표 내부를 새로 재귀 탐색하거나 Endnote를 등록하지 않는다.
- guard는 진입 시 한 번만 평가한다. 각주 하나의 등록이 새 쪽을 만들 수 있으므로
  각주 목록을 미리 계획하거나 높이를 일괄 예약하지 않고 매번 즉시 기존 메서드를 호출한다.
- `FootnoteSource::TableCell`의 문단/표/셀/셀 문단/셀 컨트롤 인덱스는 원본과 같다.
- 일반 경로는 각주 등록 뒤에 `break_after_current_table`을 판단한다.
  지연 경로는 각주 등록 뒤에 원래 문단 앵커를 확정한다.
- 장식 표의 continue, stored TAC 조기 반환, host Body 각주, fragment 큐 등록,
  각주 높이 추정·쪽 배분 본체는 변경하지 않는다.

조건·수치·출력·기존 알고리즘의 타당성을 바꾸거나 새로 승인하지 않는다.
Rust 테스트·assertion·baseline·golden·ignore·CI 변경은 없다.

## 2. 검증 계획과 한계

`output/7280/stage30/verify-table-notes.mjs`에서 새 coordinator를 양쪽 호출자에 다시 펼쳐
시작 head와 비교한다. guard/순회/인덱스/등록 위치와 나머지 부모 코드 불변,
상태 query의 읽기 전용 식을 확인한다. 실행 증거나 시각 판정을 대체하지 않는다.

별도 review worktree에서 suite 준비, 고정 baseline 대비 manifest/unit-tier 검사,
fmt, native Clippy, 집중 nextest를 순차 수행한다.
R2aa 281건을 유지하고 기존 `issue_3738_rowbreak_table_footnote_fragment` 모듈 전체를 선택한다.
이 모듈의 원본 각주 쪽 소유권·분할·표/본문/각주 경계 검사와 입력을 변형한 계약을 구분한다.
통과 결과를 모든 일반/지연 각주 조합의 분기 실행 증명이나 한컴 직접 시각 일치로 격상하지 않는다.

전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM 및 직접 시각 대조는
구현계획 §7의 통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 승인 범위가 아니다.

## 3. 고정 head 검증

- 제품 SHA: `7fd2aa07e18a40bd582779bb4d2f17b7a97234c9`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2ab`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
- 정적 대조 통과: `output/7280/stage30/extraction-proof.json`.
- manifest: 1,382 sources / 5,965 static attrs / 48 targets 통과.
- unit-tier: 4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 /
  cfg support 28 통과. 고정 baseline과 비교했으며 source-side test 변경은 없다.
- fmt 통과, native Clippy `-D warnings` 통과(exit 0, 55.89초).
- 로그: `output/7280/stage30/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

위 review worktree에서 다음 명령을 순차 실행했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage30/run-focused.sh
```

집중 명령은 [R2aa](task_m100_7280_stage29.md#3-고정-head-검증)와 같은 target/옵션에서
worktree를 R2ab로 변경하고, #3738 각주 모듈의 정확한 단일 테스트 필터를
`test(issue_3738_rowbreak_table_footnote_fragment::)`로 확장했다.
이 모듈은 기존 대상 suite에 있어 대상 바이너리는 늘리지 않는다.
기존 1건에서 33건으로 확장하므로 전체 선택은 281 + 32 = 313건이다.

결과: **313건 통과 / 실패 0건**, 24 binaries, 필터 비선택 7,896건, exit 0.
빌드 6분 48초, 테스트 2.824초. 실행 ID: `53d29405-4015-45f7-88c3-fd880fd8c14d`.
로그: `output/7280/stage30/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행에서 같은
필터로 선택한 313개 PASS 이름과 일치함을 확인했다. 전체 회귀를 재실행한 결과가 아니며,
필터 비선택 수는 기존 ignore 50건과 별개다. 출력 픽셀 동일성이나 직접 시각 판정은 아니다.

nextest 0.9.137(권장 0.9.140) 및 observation profile 설정 경고는 기준 실행과 동일하다.
review worktree의 tracked 변경은 없고 파생 suite/manifest는 커밋하지 않았다.
검증 뒤에는 계획과 결과 기록만 수정했으며 제품 코드 변경은 없다.

## 4. 후속

표 문단 루프의 진입/배치/후처리 조정을 하나의 문단 coordinator로 연결하는 경계를 점검한다.
특히 진입 진단·가용 단 너비·배치 전 높이/쪽 수 snapshot과 루프 뒤 TAC 정산 조건의
상태 접근을 기존 시점대로 옮겨, 부모는 조정 진입점으로 남기는 범위를 검토한다.
표 포맷/TAC·블록 배치/이어받기 본체, 각주 등록 알고리즘과 전체 통합 게이트는 남아 있다.
