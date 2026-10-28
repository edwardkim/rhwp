# Task #7280 Stage 29 — R2aa 일반/TAC 표 배치 경로 조정 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2z](task_m100_7280_stage28.md), 시작 head `718eebe66`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2aa 구현·고정 제품 SHA 집중 검증 완료. R2 전체/PR 준비 완료가 아니다.

## 1. 책임과 보존 범위

장식 표를 제외한 일반/TAC 표 하나의 포맷·배치 선택·후행 표 지연 등록을
`controls/flow_table.rs`로 옮긴다. `FlowTableInput`은 기존 문단 입력과 구성/측정 결과를 빌리며,
가변 페이지 상태와 문단 전체에서 공유하는 float lane은 별도 인자로 전달한다.
새 public API·IR·PageItem 스키마·trait·범용 rule engine은 만들지 않는다.

`natural_top_lead`는 strict fit의 선행 높이만 조회하고,
`state::prepare_flow_table_anchor`는 배타 영역을 소비한 뒤 실제 높이를 반환한다.
조정자는 기존 엔진의 포맷/TAC/빈 호스트 float/블록 표 배치 메서드를 호출한다.
엔진 의존을 완전히 제거한 단계가 아니며, 분할 알고리즘과 그 내부 상태 쓰기는 후속 분리 대상이다.
읽기 전용 계산 함수에는 엔진이나 가변 상태를 전달하지 않는다.

### 실행 순서와 의미 보존

- 부모가 일반 표의 host 텍스트 소유권을 표시한 뒤 조정자를 호출한다.
  현재 높이 `< 1.0` 조회 → 기존 포맷 → strict fit 앵커 준비 → 측정 결과/첫·마지막 표 조회 순서다.
- strict fit에서만 signed 세로 오프셋의 `.max(0)`와 바깥 위 여백을 px로 합산한다.
  기존 float 배타 영역 소비 후의 `current_height`가 보정 앵커다. strict fit이 아니면 원래
  `para_start_height`를 반환한다. 새 clamp·수치·좌표계 변환을 추가하지 않는다.
- 유효 TAC 판단 → TAC 배치, 아니면 빈 호스트 float 배치 시도, 실패하면 블록 표 배치 순서다.
  앞 문단의 저장 vpos는 TAC 경로 안에서만 조회한다. 원래 엔진 profile 조회 시점도 유지한다.
- 빈 호스트 float와 지연 후보는 원래 `para_start_height`를 계속 받는다.
  블록 표는 기존처럼 보정 앵커를 배치/예산 인자 두 곳에 전달한다. 두 높이를 합치지 않는다.
- 블록 표 직전 쪽 수 관측 → 블록 배치 → 지연 조회 생성 → 배치 후 쪽 수/현재 항목 관측 →
  기존 후보 조회 → 비어 있지 않을 때만 큐 등록 순서다. R2s/R2t 후보/큐 알고리즘은 바꾸지 않는다.
- 조정자는 큐 등록 여부로 반복 중단 신호를 반환한다. 부모는 현재 표의 각주 수집을 먼저 수행하고
  그 뒤 중단한다. 원래 반복 밖 bool은 true가 되면 같은 회차 끝에서 반드시 break했으므로
  다음 회차로 true가 전달되지 않는다. 회차별 반환값으로 옮겨도 각주와 후행 컨트롤 순서는 같다.
- 장식 표 continue, 비표 컨트롤 배치, 루프 뒤 host 텍스트 발행과 TAC 높이 정산은 그대로 둔다.
- 지역 `FormattedTable`은 조정자 반환 때 해제되어 부모 각주 순회보다 수명이 짧아진다.
  이 결과는 소유 수치/벡터이며 사용자 정의 Drop이 없다. 뒤 각주 순회는 이를 참조하지 않고
  원본 `table.cells`와 상태의 등록 여부를 읽으므로 배치/각주 데이터 소유권은 바뀌지 않는다.

기존 분기·수치의 타당성을 새로 승인하거나 조판 문제를 수정한 것으로 보고하지 않는다.
테스트 source/assertion/ID, baseline/golden/ignore 및 CI는 변경하지 않는다.

## 2. 검증 계획과 한계

`output/7280/stage29/verify-flow-table.mjs`로 원래 포맷/strict 앵커/경로/호출 인자/후행 큐 코드를
복원해 비교한다. 입력 묶음의 모든 필드 연결, 상태 관측/반영 식, 각주-before-break,
나머지 부모 경로/테스트·기존 controls/state 본문 불변을 확인한다.
정적 대조는 실행 검증이나 전체 시각 일치의 대체물이 아니다.

별도 review worktree에서 파생 suite 준비 → 고정 baseline 대비 manifest/unit-tier → fmt →
native Clippy → 집중 nextest를 순차 수행한다. R2z 275건에 다음 기존 6건을 추가한다.

- `issue_2439` 4건: 지연 float의 새 쪽 앵커/후속 텍스트 보존, 0 오프셋 형제 표의 배타 영역,
  양수 오프셋 빈 host의 후속 흐름과 위 여백 델타, RowBreak 이어받기의 제목 행/후속 문단.
  축소/합성 fixture와 HWP5 provenance·저장 줄을 재설정한 계약이 포함되어 있다.
  정상 한컴 원본을 새로 직접 시각 대조한 증거로 격상하지 않는다.
- `issue_2322_fullpage_form_table_pair` 2건: 실제 HWP 전면 서식 표의 float 배타 영역과
  전면 TAC 대조군. 둘 다 2쪽 계약이며 첫 검사는 두 번째 표의 2쪽 소속도 확인한다.
  모든 표 외곽/내용 줄의 기하 동일성을 보증하는 검사는 아니다.

기존 선택에 지연 배치·TAC/float 혼재·표 각주 계약이 포함되어 있다. 모든 branch/각주 조합을
새 fixture로 추가하지 않으며, 세부 조합의 실행 범위는 기존 테스트 이상으로 주장하지 않는다.
전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM 및 직접 시각 대조는
구현계획 §7의 통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 승인 범위가 아니다.

## 3. 고정 head 검증

- 제품 SHA: `d55263bf54b426562a120d31ecdb3d3d11e9c02d`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2aa`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
- 정적 대조 통과: `output/7280/stage29/extraction-proof.json`.
- 파생 suite 준비 후 manifest 고정 baseline 비교 통과:
  1,382 sources / 5,965 static attrs / 48 targets.
- unit-tier 고정 baseline 비교 통과:
  4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 / cfg support 28.
- `cargo fmt --all -- --check` 통과.
- native Clippy `-D warnings` 통과(exit 0, 56.41초).
- 로그: `output/7280/stage29/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

위 review worktree에서 다음 순서로 실행했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage29/run-focused.sh
```

집중 명령은 [R2z의 명령](task_m100_7280_stage28.md#3-고정-head-검증)에서 worktree를 R2aa로
바꾸고 `-E` 필터에 `test(issue_2439::)`와
`test(issue_2322_fullpage_form_table_pair::)`를 OR 추가했다.
두 모듈의 suite 026/002는 기존 대상에 포함되어 있어 target은 그대로다.
`--locked --cargo-profile release-test --lib --no-fail-fast`와 고정 target도 동일하다.

결과: **281건 통과 / 실패 0건**, 24 binaries, 필터 비선택 7,928건, exit 0.
빌드 6분 43초, 테스트 1.566초. 실행 ID: `d50c1982-f565-4655-84cb-890eed550989`.
로그: `output/7280/stage29/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행에서 동일 필터로
선택한 281개 PASS 이름과 일치함을 확인했다. 이번에 전체 회귀를 재실행하지 않았으며,
필터 비선택은 기존 ignore 50건과 별개다. 출력 픽셀 동일성이나 전체 시각 일치 판정은 아니다.

nextest 0.9.137(권장 0.9.140) 및 observation profile 설정 경고는 기준 실행과 동일하다.
review worktree의 tracked 변경은 없고 파생 suite/manifest는 커밋하지 않았다.
검증 뒤에는 계획과 결과 기록만 수정했으며 제품 코드 변경은 없다.

## 4. 후속

표 문단 진입·루프의 잔여 조정과 각주 연결을 확인한다. 표 포맷/TAC·블록 배치/이어받기 본체,
최종 상태 캡슐화와 전체 통합 검증은 남아 있다.
