# Task #7280 Stage 42 — R3a 표 측정 결과 수용 판단 분리

- Issue: #7280. 이전: [Stage41](task_m100_7280_stage41.md).
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md) R3, §7.1.
- 시작 head: `df7d96038`. 고정 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R3a 측정 결과 수용 Query 분리·집중 검증 완료. R3 전체 또는 제출 게이트 완료가 아니다.

## 이번 경계

`format_table`의 측정 결과 선택을 `table::fit_measured_for_host` Query로 분리한다.
입력은 기존 Paragraph/Table/MeasuredTable 참조, dpi와 읽기 전용 profile 조회다.
전체 TypesetEngine/TypesetState나 가변 문서 접근을 전달하지 않는다.
profile은 기존 조건식에서 같은 순서로 지연 조회하고 측정 helper 호출 순서를 유지한다.
원래 측정값 fallback, host 간격, 각주 수집과 FormattedTable 조립은 호출자에 남긴다.

이 작업의 독립 기준은 고정 baseline의 동작과 기존 정식 회귀 계약이다. 선언 높이 fit,
빈 호스트의 중첩/선언 꼬리 보정 등 기존 호환 조건을 이동하지만 새 조판 규칙으로 승인하지 않는다.
HWP/HWPX IR, 계산 상수·비교 연산·clone 의미와 조건의 단락 평가를 변경하지 않는다.

소비 경로: `format_table`의 원본 측정 조회 → Query의 보정 후보 →
`fitted_visible_mt.as_ref().or(mt)` → FormattedTable의 행/누적/전체 높이다.
`controls/flow_table.rs:74`와 `deferred_placement.rs:38`의 포맷 결과는 실제 배치로 전달된다.
장식 표도 `controls/paragraph_flow.rs:114`의 지연 포맷 호출을 유지한다.
TopAndBottom·문단 기준 float가 아니거나 측정 결과가 없는 경로는 기존처럼 보정 후보가 없다.
`typeset_block_table_inner`는 `ft.effective_height`를 float 밴드 및 전체 fit에 사용하지만,
행 분할에는 **호출자가 별도로 전달한 원본 MeasuredTable**과 LayoutEngine 컷 높이를 사용한다.
즉 이번 추출을 fit/분할/paint의 모든 측정이 이미 단일 결과로 통일됐다는 증거로 삼지 않는다.
기존 분기별 원본/유효 표 선택·높이 보정은 그대로 남긴다.
이번 절편은 시작/끝 컷·누적 예약·빈 물리 밴드·이어받기 종료 알고리즘을 수정하지 않는다.

제품 `17a378136f` 기준 주요 소비 위치는 `typeset.rs:16146`(Query), `:16147`(fallback),
`:16381`(host 포함 총높이), `:19450`(블록 조정), `:21231`/`:21297`(별도 컷/온전한 행 예산),
`:22857`(행 스캔 호출)이다. 공용 height_measurer의 세 fit helper는 원본을 수정하지 않고
clone에 행/누적/전체 높이를 계산한다. helper 본체와 parent의 문단 텍스트 판별은 이번에 이동하지 않았다.

## 검증 계획

원본 추출식과 새 Query를 dpi/profile 표기만 정규화해 대조하고 나머지 parent가 불변인지 검사한다.
기존 표 fit·성장·꼬리·TAC/float 회귀를 별도 integration suite에서 실행한다.
새 source-side test/support와 public API는 추가하지 않는다. baseline·ignore도 변경하지 않는다.
review worktree에서 suite 준비·고정 baseline 정책 검사·fmt·native Clippy·집중 nextest를 순차 실행한다.
정적 대조는 모든 분기 실행이나 한컴 시각 일치의 증거가 아니다.

§7.2의 R3 책임 묶음 전체 회귀·Native/fresh WASM 시각 비교, §7.3의 최종 제출 lint/build
묶음은 후속 게이트로 남긴다. 이번 절편에서 원격 push·PR·댓글은 수행하지 않는다.

## 진행 기록

- 제품 SHA: `17a378136f2d692d8e9ebe56eb8e35f5de48975a`.
- 정적 증거: `output/7280/stage42/{verify-table-fit.mjs,extraction-proof.json}`.
  Query 본문은 dpi/profile 접근 표기·포맷만 정규화하면 원본과 동일하다.
  3개 profile 지연 호출 위치 및 호출자 나머지 전체(간격·각주·분할·테스트 포함) 불변을 확인했다.
- 주 checkout의 첫 `cargo fmt --all`은 기존 파생 suite의 삭제된 source 참조
  `issue_7090_sibling_table_occupancy.rs` 때문에 중단했다. 제품 컴파일 실패가 아니다.
  주 checkout의 파생 파일을 수정하지 않고 전용 review worktree에서 `--prepare` 후
  `cargo fmt --all -- --check`를 다시 수행해 통과했다.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r3a`.
  공유 `target/pr-review`를 유지하고 Cargo는 순차 실행한다.
  host 16 logical CPUs / RAM 31 GiB, 가용 약 19 GiB를 확인했다.
  빌드 jobs 4 / focused test threads 8을 사용한다.
- manifest: 1,382 sources / 5,965 static attrs / 48 targets 통과.
- source-side 정책: 4,205 tests / 298 modules / cfg support 28 유지, 고정 baseline 대비 통과.
- 집중 선택: Stage40의 기존 327건에 height_measurer 계약과 #5879(본문 하한·쪽 경계),
  #5748(TAC 행 내용 하한·clip 내부 baseline) 계약을 추가한다.
  #5906의 빈 host 마지막 행/외곽·쪽수 계약은 기존 선택에 포함돼 있다.
  실제 suite 목록·필터는 `output/7280/stage42/run-focused.sh`에 보존한다.
  #5748은 TAC 대조군이지 TopAndBottom 전용 Query guard의 실행 증거는 아니다.
  모든 guard 조합·profile 호출 횟수의 동적 검증 및 편집 UI 검증을 주장하지 않는다.

## 고정 head 검증 결과

review worktree에서 다음 명령을 순차 실행했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage42/run-focused.sh
```

- fmt·native Clippy `-D warnings` 통과(exit 0, Clippy 59.15초).
- 집중 nextest **352 passed / 0 failed**, 24 binaries, 필터 비선택 7,857건, exit 0.
  이 비선택 건수는 전체 회귀의 기존 ignore 50건과 구분한다.
- 빌드 6분 46초, 테스트 3.562초.
  run ID: `8c269bd2-13e9-4bd7-aaa0-a32cbd1f60d5`.
- `output/7280/stage42/{prepare,manifest,unit-tier,fmt,clippy-native,nextest-focused}.log`에 기록.
  `compare-focused.mjs` / `regression-comparison.json`에서 고정 baseline의 같은 352개
  PASS 이름과 일치함을 확인했다. 새 결함 수정이 아니므로 수정 전 FAIL을 주장하지 않는다.
- 주요 추가/기존 실물 입력 SHA-256은 `fixture.sha256`에 보존했다.
- nextest 0.9.137 권장 버전·observation 설정 경고는 기존과 동일하다.
  원격 CI와 도구 환경이 완전히 같다는 뜻은 아니다.

검증 이후 제품 코드는 변경하지 않았다. review worktree의 tracked 변경은 없으며,
파생 suite/manifest·진단 산출물은 커밋하지 않는다. 문서 로컬 링크와 `git diff --check`도 확인했다.
이번 절편의 정적 보존·기존 계약은 충족했으나, R3 전체 시각/통합 검증과 최종 제출 게이트는 남아 있다.

## 다음 절편

`format_table`에 남은 host 간격 계산과 결과 조립을 분리한다. 특히 실제 흐름용 `after`와
fit용 `after_for_fit`, strict 후속 문단 fit의 구분을 보존한다. 각주 수집 순서는 유지하고,
그 뒤 행 스캔·블록 분할·continuation 책임 분리로 이어간다. 원격 push·PR·댓글은 수행하지 않았다.
