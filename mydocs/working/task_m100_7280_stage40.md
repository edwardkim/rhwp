# Task #7280 Stage 40 — R2ak 지연 그림 소유 후보 조회 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2aj](task_m100_7280_stage39.md), 시작 head `57ec65b1d`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2ak 구조 분리와 고정 제품 SHA 집중 검증 완료. R2 전체/PR 준비 완료가 아니다.

## 책임과 보존 계약

`native_hwp5_square_picture_next_page_owner`는 façade로 유지하고 후보 조회를
`controls/deferred_picture::next_page_owner`로 이동한다. 이 후보에만 쓰는 연속 저장 밴드
문단 조회 helper도 같은 모듈로 옮기며 기존 unit test의 helper 호출 경로만 새 모듈로 연결한다.
테스트 입력·이름·assertion 의미는 변경하지 않는다. 신규 cfg(test) support item은 추가하지 않는다.

Query는 `TypesetState`를 받지 않고 필요한 profile/단 수/현재 항목 차용/각주 높이/흐름 높이만
`DeferredPicturePage`로 관측한다. state가 이를 만든다. 단순 scalar/참조 읽기는 진입 시 수행하지만
profile 판정·항목 존재 판정·각주 guard는 원래 순서에 남긴다. 가용 높이는 closure로 전달해
reset_idx==0의 저장 전진량 비교 및 최종 frame 비교 시점에만 조회한다. 진단 호출을 선행하지 않는다.

실제 경로: 구역의 비표 그림 처리 → 후보 Query → `DeferredSquarePictureControl` 큐 등록 후
continue → 기존 `push_new_page`의 anchor 등록/보류 목록 반영 → 그림 발행이다.
Query의 저장 밴드 대상·WrapAnchorRef는 원래 호출자가 그대로 큐에 전달한다. 현재 본문을 다음
쪽으로 옮기거나 현재 쪽에 그림을 즉시 발행하지 않는다. 큐·페이지 수명 코드는 R5에 그대로 남긴다.

높이 경로는 저장 줄 높이/간격 → 필요 시 현재 각주 예산과 비교 → 밴드 대상 제한 →
그림 frame + 캡션 측정 + 캡션 간격 → 최종 가용 높이 비교다. `LayoutEngine` 캡션 측정 의존은
기존 위치/횟수를 보존한 잔여 의존이며 새 공통 측정 정책으로 바꾸지 않는다.
표 컷/이어받기·각주 예약 알고리즘과 본문/미주 인덱스는 비변경이다.

기존 호환 조건·상수·IR/API·테스트 기준값·ignore·CI를 변경하지 않는다.
기존 예외를 유지하는 구조 이동이며 조판 규칙의 타당성을 새로 승인한 것은 아니다.

## 검증 계획과 한계

작업지시자의 테스트 분리 지침을 적용한다. 새 회귀 테스트는 제품 소스가 아니라
`tests/cases/` 원본과 별도 integration suite 경로를 따른다. 이번에는 새 테스트를 추가하지 않고
기존 계약을 재사용한다. 기존 private white-box 테스트의 별도 crate 이전은 내부 경계/API와
함께 설계할 항목이며, 이번 단순 조회 이동에 테스트 전면 이전을 섞지 않는다.

`output/7280/stage40/verify-deferred-picture.mjs`로 Query와 밴드 helper를 원본과 대조하고,
snapshot 필드·façade 인자·나머지 parent/state/controls(호출자·큐 포함) 불변을 확인한다.
기존 unit test의 helper 호출 경로만 정규화하여 나머지 테스트 본문 불변도 확인한다.
이는 분기 커버리지나 직접 시각 판정이 아니다.

집중 선택은 Stage39의 325건에 기존 layout 계약 2건을 더한 327건이다.

- 기존 `native_hwp5_square_picture_uses_the_next_page_wrap_owner`: p155 본문 유지,
  p156 그림 64 소유/좌표/중복 부재와 좁은 본문 비겹침.
- 기존 `native_hwp5_square_picture_figure_56_uses_the_same_next_page_owner_contract`:
  다음 문단이 좁은 밴드로 시작하는 그림 56의 p126/p127 소유 계약.
- 기존 typeset band unit: 빈 guide를 포함하는 연속 범위 및 첫 범위 밖 문단에서 종료.
- 추가 선택 `issue_3821_page_tail_square_picture_wrap_reaches_visible_text_after_guides`:
  실물 p156의 guide 뒤 visible 문단과 그림 사이 간격.
- 추가 선택 `issue_3738_picture_caption_float_clears_caption_before_next_body_text`:
  p182의 그림/캡션을 포함하는 표 하단과 후속 본문 상단 간격 대조군이다.
  그림 캡션 자체의 모든 기하나 이번 후보 guard 전체를 검사한다는 뜻은 아니다.

위 원본 fixture를 review worktree에서 확인하고 동일 baseline PASS 이름과 대조한다.
비Picture/다단/각주 없음 등 모든 거절 조합과 closure 호출 횟수의 동적 계측은 미검증이다.
새 테스트·기준값 없이 suite 준비 → manifest/unit-tier baseline 비교 → fmt → native Clippy →
집중 nextest를 순차 실행한다. 주 checkout의 파생 suite는 건드리지 않는다.
전체 회귀·WASM/workspace lint/build·Native Skia·fresh Docker WASM·직접 출력 비교는
구현계획 §7의 책임 묶음/제출 전 게이트에 남긴다. 원격 push·PR·댓글은 범위 밖이다.

## 검증 진행 기록

최초 제품 `4483a67c8c36ada1783fb4708e7b1b4428e89591`에서 test-only import가 신규
cfg support로 검출되어 unit-tier가 실패했다. `output/7280/stage40/unit-tier-initial-failed.log`에
보존했다. 기준선을 늘리지 않고 해당 import를 제거하고 기존 test의 호출 경로만 한정했다.
수정 제품 `99d144e3f346eed2e0b92771dfd54034e662153f`로 review worktree를 전환한 뒤
prepare부터 다시 실행했다. 최초 실패 이후 이전 검증 결과를 새 head에 재사용하지 않았다.

- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2ak`.
- 고정 검증 제품 SHA: `99d144e3f346eed2e0b92771dfd54034e662153f`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
  host 16 logical CPUs / RAM 31 GiB, 시작 시 가용 약 19 GiB. 다른 Cargo 없음 확인.
- 정적 대조 통과: `output/7280/stage40/{verify-deferred-picture.mjs,extraction-proof.json}`.
- manifest: 1,382 sources / 5,965 static attrs / 48 targets 통과.
- unit-tier: 4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 /
  cfg support 28 통과. 새로운 테스트/support 항목 없음.
- fmt 통과, native Clippy `-D warnings` 통과(exit 0, 55.46초).
- 로그: `output/7280/stage40/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.
- 실물 fixture 존재/크기 확인 및 SHA-256: `output/7280/stage40/fixture.sha256`.

고정 review worktree의 명령은 다음과 같다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage40/run-focused.sh
```

집중 nextest 결과: **327 passed / 0 failed**, 24 binaries, 필터 비선택 7,882건, exit 0.
빌드 6분 44초, 테스트 2.871초. run ID: `3eeafa93-8fd1-4a32-b0c7-9e254682774c`.
정확한 필터·명령은 `output/7280/stage40/run-focused.sh`, 실행 결과는 `nextest-focused.log`,
baseline 이름 대조는 `compare-focused.mjs`와 `regression-comparison.json`에 보존했다.
선택한 기존 327건의 PASS 이름이 baseline 전수 로그의 동일 선택과 일치한다.
비선택 건수는 기존 ignore 50건과 별개다. 전체 회귀 또는 직접 시각 일치의 증거는 아니다.
nextest 0.9.137 권장 버전 및 observation 설정 경고는 이전과 동일하다.

검증 후 제품 코드를 변경하지 않았다. review worktree tracked 변경은 없고 파생 suite/manifest는
커밋하지 않았다. 두 Markdown의 로컬 파일 링크 존재 및 `git diff --check`를 확인했다.
원격 push·PR·댓글은 실행하지 않았다.

## 후속

R2 진입/소유 경계의 잔여 목록을 재점검하고 책임 묶음 통합 검증으로 이어간다.
R3 표 분할/이어받기, R4 각주/미주 본체, R5 구역 수명/상태 접근 제어는 별도 책임으로 유지한다.
