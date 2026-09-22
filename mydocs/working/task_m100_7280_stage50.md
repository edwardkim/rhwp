# Task #7280 Stage 50 — R3i 일반 행 분할 진입·마지막 주석 행 Query 분리

- Issue: #7280. 이전: [Stage49](task_m100_7280_stage49.md).
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md) R3, §7.1.
- 시작 head: `d644cc6be`. 제품: `a63d366a984b8578303974d2ddc0a925cd16e956`.
- 고정 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R3i 구조 분리·정적 대조·집중 검증 완료. R3 전체·최종 제출 게이트 완료는 아니다.

## 이번 경계

`table/scan/row_entry.rs`에 일반 행의 마지막 주석 행 형상/컷, 분할 진입, 패딩과 native
저장 reset 조건을 분리했다. `RowEntryQuery`는 읽기 전용 행 Query와 현재 행 시작 컷만
받는다. 전체 TypesetState/TypesetEngine과 가변 페이지 상태는 전달하지 않는다.

- `terminal_note_shape`: strict 여부·마지막 행·rowspan·full-width 단일 셀·컨트롤 없는
  가시 문단·비합성 저장 LineSeg 조건을 원래 순서로 조회한다.
- `terminal_note_probe`: 기존 형상 guard 안에서 남은 밴드와 컷을 조회한다. 예산이 0일
  때도 먼저 컷을 조회하는 순서를 보존한다. `TerminalNoteProbe`는 컷을 clone 없이 반환한다.
  양의 예산/완결/양의 소비 높이 수용 조건과 실제 예약·행 전진·높이 override는 부모에 남겼다.
- `split_gate`: native short-parent child 조회를 먼저 실행한 뒤 일반 분할 가능성과 조합한다.
  `can_intra_split`이 거짓이라는 이유로 native 조회를 생략하도록 바꾸지 않는다.
- `padding`: RowBreak의 잔여 표시 패딩과 그 외 행 최대 패딩의 기존 선택을 보존한다.
- `native_reset_tail`: profile·비-TAC·RowBreak·행 위치·시작 컷을 확인한 뒤 내부 hard break를
  조회한다. 패딩 차감 뒤 실행하는 기존 시점을 유지한다.

새 예외·허용치·공개 API·source-side 테스트는 추가하지 않았다. 기존 조건/출력을 보존하는
구조 작업이며, 마지막 주석 행이나 native child 정책의 정확성을 새로 승인한 것이 아니다.

## 실제 호출·컷·높이·배치 연결

제품 `a63d366a9` 기준:

1. `typeset.rs:22197` 일반 스캔과 `:22330` 각주 예약 후 refit이 같은 스캔을 호출한다.
   블록/rowspan 전용 및 일반 행 통째 수용 경로를 지난 행에서 `:17984` Query를 만든다.
   선행 조기 continue 경로와 별도 TAC 배치는 이번 조회에 들어오지 않는다.
2. `:17989` 주석 형상이 맞으면 `:17994` 컷을 조회한다. 부모는 기존 컷 완결 판정 후
   `cs_before + remaining_band`를 예약하고 `end_row_height_override`에 같은 밴드를 넣는다.
   내용 컷과 선언 행의 남은 물리 높이를 같은 수치로 대체하지 않는다.
3. 미수용이면 `:18011` 분할 gate를 조회한다. 불가한 첫 행의 강제 통째 수용과 그 외 행의
   이월, 정지 진단·break는 그대로다. 가능할 때만 `:18032` 패딩 → content budget →
   `:18034` native reset 순서로 실행한 뒤 기존 mixed-nested reserve 컷을 조회한다.
4. 후속 `:18460` painted-height 고아 판정이 native reset/child flags를 소비한다. 일반 컷,
   저장 프레임 확장, 요구 높이·예산 실패 시 재-cut/이월/누적 예약은 이번에 이동하지 않았다.
5. `:22604` PartialTable 발행과 `:22691` continuation 전진으로 기존 컷/점유 결과를
   전달한다. 종료 뒤 캡션·각주·후행 본문과 빈 밴드/패딩 소유는 변경하지 않았다.

`output/7280/stage50/{extract-row-entry.mjs,extraction-proof.json}`에 원본 식·조회 순서와
부모 전체의 허용된 호출 치환 외 변경이 없음을 확인하는 정적 대조를 보존한다.
모든 분기의 동적 커버리지나 새로운 측정/paint 공통화를 주장하는 증거는 아니다.

## 검증 계약과 한계

기존 389건에 **기존 계약 1건**을 추가한다. 테스트 원본·기대값·ignore는 변경하지 않는다.

| 계약 | 실제 검사 및 한계 |
| --- | --- |
| 기존 #2097 band-fill 1건 | 6개 문서의 쪽수 핀. 마지막 주석 행 관련 `21217935`는 8쪽. 전체 내용/기하·0 예산 분기 실행 여부는 검사하지 않음 |
| 추가 #2308 short-child owner 1건 | `76076` 81쪽에 `…등의 사고`가 한 줄로 존재하고 82쪽에 `고를 예방…`이 없는지 검사. 문서 전체의 중복/누락 전수 검사는 아님 |
| 기존 #3820·#3738 등 | 표 조각/rowspan·각주와 본문 경계 등 앞 절편의 기존 계약을 함께 유지 |

주석 행 계약의 연계는 [#3820 Stage232](archives/task_m100_3820_stage232_terminal_note_row_band_owner.md)의
원인·입력 기록과 현행 `tests/issue_2097_band_fill.rs`를 대조했다. 이 기존 핀은 쪽수만 검사하므로
주석 셀의 정확한 외곽이나 내용 소유를 별도로 증명하지 않는다.
#2308의 별도 wrap-point 계약은 #5193으로 기존 ignore 상태이며 이번 선택에 넣거나 완화하지 않았다.
81/82쪽 owner 계약의 PASS를 그 wrap-point 일치로 보고하지 않는다. 나머지 389건의 한계는
[Stage49](task_m100_7280_stage49.md)와 연결된 앞 절편을 따른다.

review worktree: `/home/edward/mygithub/rhwp-review-7280-r3i`.
공유 `target/pr-review`를 보존하고 jobs 4 / test threads 8로 순차 실행한다.
이번에는 실행 전에 `--prepare`한 실제 suite 013과 baseline의 해당 PASS 이름을 대조하고
runner를 고정했다. 실행 중인 스크립트를 수정하지 않는다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage50/run-focused.sh
```

## 고정 제품 검증 결과

- manifest 통과: 1,382 sources / 5,965 static attrs / 48 integration targets.
- source-side 정책 통과: 4,205 tests / 298 modules / cfg support 28 유지.
- fmt·native Clippy 통과(exit 0). Clippy 57.84초.
- 집중 nextest **390 passed / 0 failed**, 27 binaries, 필터 비선택 8,454건, exit 0.
  비선택은 전체 회귀의 기존 ignore와 구분한다. 빌드 7분 13초, 테스트 11.302초.
  run ID: `588e0622-7bab-4a35-b68e-0f4b94495562`.
- `output/7280/stage50/{compare-focused.mjs,regression-comparison.json}`에서 고정 baseline의
  동일 390개 PASS 이름과 일치함을 확인했다. 선택 밖 전체 회귀는 재실행하지 않았다.
- 실행 로그: `output/7280/stage50/{prepare,manifest,unit-tier,fmt,clippy-native,nextest-focused}.log`.
  이번 경로와 연결한 입력 2개의 SHA-256을 `fixture.sha256`에 기록하고 실행 뒤 재확인했다.
  nextest 0.9.137/권장 0.9.140 및 observation 설정 경고는 이전과 동일하다.

검증 후 제품 변경 없음, review worktree tracked 변경 없음, 문서 상대 링크와
`git diff --check` 통과를 확인했다. 파생 suite/manifest·진단 산출물은 stage하지 않는다.
이번 절편의 구조 보존·기존 집중 계약은 충족했지만 R3 전체 회귀·Native/fresh Docker WASM
시각 비교·최종 제출 게이트는 미실행이다. 기존 계약 재실행이며 결함 수정 전 FAIL/후 PASS나
한컴 출력과의 신규 시각 일치를 주장하지 않는다. 원격 작업은 하지 않았다.

## 다음 절편

일반 행의 저장 프레임 꼬리 확장 조건/컷 선택을 분리한다. 후속 예산 재판정·스캔 상태 소유·
continuation 본체와 R3 통합 검증도 남아 있다. R3 전체 회귀·Native/fresh Docker WASM
시각 비교 및 최종 제출 게이트는 아직 미실행이다. 원격 작업은 승인 범위가 아니다.
