# Task #7280 Stage 27 — R2y 표 컨트롤 진입 판단 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2x](task_m100_7280_stage26.md), 시작 head `76ce6a501`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2y 구현·고정 제품 SHA 집중 검증 완료. R2 전체/PR 준비 완료가 아니다.

## 1. 책임과 보존 범위

일반 `typeset_table_paragraph`의 표 컨트롤 진입부를 분리한다.
`controls/table_entry.rs`는 저장 줄 경계의 쪽 이월 필요성과 데코레이션 경로 선택만 조회한다.
`controls::prepare_table_control`은 첫 판단 → 기존 쪽/단 전환 → 두 번째 판단 순서를 소유한다.
`state::table_control_page`는 현재 항목 유무와 단 수만 읽는다. 전환 알고리즘은 수정하지 않는다.

실제 장식 표 항목 발행, 표 측정/포맷, continuation 컷·예약·진단, host 소유 플래그,
일반/TAC 표 배치와 각주 처리는 부모의 기존 본문에 남긴다. 이번 절편은 그 본문 이동이 아니다.
새 public API·IR·PageItem 스키마나 범용 rule engine을 추가하지 않는다.

### 실행 순서와 의미 보존

- 저장 줄 경계 조건의 `order_pos`, 원래 `treat_as_char`, 항목 유무, 저장 태그 검사,
  vpos 0/5000 경계, 표 높이+바깥여백과 줄 높이의 2px 허용치를 그대로 유지한다.
  새로운 줄 소속 알고리즘이나 유효 TAC 속성 판정으로 대체하지 않는다.
- 이월이 필요하면 기존 `advance_column_or_new_page`를 호출하고 나서 단 수와 가용 높이를 읽는다.
  이월 전 상태를 두 번째 판단에 고정하지 않는다. 반면 `host_col_w`는 원래처럼 루프 앞 계산값이다.
- 가용 높이는 closure로 전달한다. 기존처럼 `row_count > 1`에서만 조회하며,
  종이 기준 표 여부를 검사하기 전에 평가하는 순서도 유지한다.
- profile도 closure로 전달한다. 기존 단일 단·비-TAC·비과대 조건을 통과한 경우에만
  `!self.profile.get().hwp5_stored_pagination_layout()`을 읽는다. state profile이나 새 포맷 분류로
  치환하지 않는다. 이름 `original_hwpx`는 기존 helper의 인자 맥락이지 새로운 포맷 판정이 아니다.
- 종이 기준 overlay, 다단의 빈 host/혼합 TAC host, 단 밖의 TopAndBottom 표, 원본 HWPX
  InFront flowWithText 조건과 모든 계산·분기·허용치를 유지한다. 이 조건의 타당성을 새로 승인하거나
  한컴 피델리티를 개선했다고 주장하지 않는다.
- 공유 문단 속성 helper는 다른 소비자 때문에 부모에 남겨 명시적으로 import한다.
  이는 의존 방향 정리의 이행 상태이며 상위 의존 제거 완료는 아니다.

제품 구조 외 테스트 source/assertion/ID, baseline/golden/ignore 및 CI는 변경하지 않는다.

## 2. 검증 계획과 한계

`output/7280/stage27/verify-table-entry.mjs`는 새 조회 함수에서 기존 진입 코드를 복원해
조건·식·상수·순서를 비교한다. 조정자의 전환 순서, 상태 관측값, 지연 callback 인자와
나머지 부모 배치/continuation/각주/테스트·기존 state/controls 본문 불변도 확인한다.
항목 유무·단 수의 단순 읽기는 인자 전달 시 관측하지만 상태나 진단 부작용은 없다.
정적 대조는 실행 검증 또는 전체 시각 일치의 대체물이 아니다.

별도 review worktree에서 파생 suite 준비 → 고정 baseline 대비 manifest/unit-tier → fmt →
native Clippy → 집중 nextest를 순차 수행한다. R2x 265건에 다음 기존 8건을 추가한다.

- `issue_1152_intra_para_vpos_reset` 1건: 저장 줄 리셋 표의 앞/뒤 페이지 소속.
  후속 TAC 배치 경로에도 리셋 판단이 있어 이 진입 조회만 격리 검증한 것으로 주장하지 않는다.
- `issue_775` 1건: 다단 시험지 표의 y 위치 대조군.
- `issue_1271_hwpx_behind_text_table` 3건: 종이 기준 overlay로 생기는 추가 PartialTable 방지와
  표지/다음 section 소속; 같은 입력의 글꼴·쪽 번호 검사 2건은 부가 대조군이다.
- `issue_5798_offcolumn_float_table_no_band` 1건: 단 밖 표의 흐름 밴드 비예약.
  제목 위치의 넓은 범위 assertion이지 정밀 기하 동일성 증거는 아니다.
- `issue_6366_infront_flow_paginate` 1건: 원본 HWPX 글앞으로 흐름 표의 6쪽 계약.
  쪽수만 검사하므로 세부 배치 일치 증거로 격상하지 않는다.
- `issue_5918_table_tail_fragment_page` 1건: 잔여 표 조각과 후속 표의 페이지 소속.
  flowWithText 데코레이션 제외 조건을 넓히지 않는 반례다.

임계값 양옆·모든 wrap/anchor 조합·모든 프로파일을 새 fixture로 추가하지 않는다.
그 세부 조합은 정적 보존 대조와 기존 집중 범위 이상으로 실행 입증한 것이 아니다.
전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM 및 직접 시각 대조는
구현계획 §7의 통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 승인 범위가 아니다.

## 3. 고정 head 검증

- 제품 SHA: `0630b1bb3cfa58d51331eb4d27af0e35176cec8b`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2y`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
- 정적 대조 통과: `output/7280/stage27/extraction-proof.json`.
- 파생 suite 준비 후 manifest 고정 baseline 비교 통과:
  1,382 sources / 5,965 static attrs / 48 targets.
- unit-tier 고정 baseline 비교 통과:
  4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 / cfg support 28.
- `cargo fmt --all -- --check` 통과.
- native Clippy `-D warnings` 통과(exit 0, 56.61초).
- 로그: `output/7280/stage27/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

위 review worktree에서 다음 순서로 실행했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage27/run-focused.sh
```

집중 명령은 [R2x의 전체 명령](task_m100_7280_stage26.md#3-고정-head-검증)에서 worktree를 R2y로
바꾸고 `--test regression_suite_024 --test regression_suite_026`을 더한 것이다.
기존 `-E` 필터에 §2의 여섯 모듈 각각을 `test(<모듈명>::)`으로 OR 추가했다.
`--locked --cargo-profile release-test --lib --no-fail-fast`와 고정 target은 동일하다.

결과: **273건 통과 / 실패 0건**, 24 binaries, 필터 비선택 7,936건, exit 0.
빌드 6분 42초, 테스트 1.568초. 실행 ID: `ae2dbce6-1ed8-451e-95cd-7f45596ae47e`.
로그: `output/7280/stage27/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행에서 동일 필터로
선택한 273개 PASS 이름과 일치함을 확인했다. 이번에 전체 회귀를 재실행하지 않았으며,
필터 비선택은 기존 ignore 50건과 별개다. 출력 픽셀 동일성이나 전체 시각 일치 판정은 아니다.

nextest 0.9.137(권장 0.9.140) 및 observation profile 설정 경고는 기준 실행과 동일하다.
review worktree의 tracked 변경은 없고 파생 suite/manifest는 커밋하지 않았다.
검증 뒤에는 계획과 결과 기록만 수정했으며 제품 코드 변경은 없다.

## 4. 후속

일반 표 문단에 남은 데코레이션 표의 실제 배치·continuation 및 일반/TAC 표 배치 조정 책임을
이어 확인한다. R3 표 컷/이어받기, 각주, 최종 상태 캡슐화와 전체 통합 검증은 남아 있다.
