---
kind: report
status: active
canonical: mydocs/plans/task_m100_6812.md
issue: 6812
last_verified: 2026-09-07
---

# #6812 완료 보고 — 원본 1페이지 해결로 범위 확정

## 1. 메인테이너 판정

2026-09-07 메인테이너는 `edf083614` baseline을 Docker WASM으로 빌드한
rhwp-studio에서 원본 1페이지의 그림과 TAC 표 조판 문제가 해결됐음을 직접 확인했다.
이번 타스크는 여기서 완료하며, 모든 문제를 한꺼번에 해결하지 않고 분할 정복하기로 결정했다.
이 결정이 이전의 미완성 확장 전부를 같은 타스크에서 처리한다는 지시를 대체한다.

## 2. 수용 기준본과 실제 검증

- 제품 코드: `edf083614027a84c1f838cd9fdf370988b48766f`.
- 브랜치: `task_m100_6812_edf083614_baseline`.
- 입력: review-6798 worktree의
  `samples/issue6797/156160455-social-pig-farm-income.hwp`.
- Docker 표준 WASM 빌드: exit 0, 6분 42초, 최적화 포함.
- `pkg/rhwp_bg.wasm`: 10,315,568 bytes,
  SHA-256 `f93e6b4afb95dbccb4ed28a3c18403695aa4ec6ff948cc7395e57e639c782527`.
- 새 WASM의 원본 로드·11쪽 계산·첫 페이지 SVG 생성 성공.
- 첫 페이지 SVG: 338,679 bytes,
  SHA-256 `b4703e335a54c1200f5455c7de5622a384a3edb3ddd7caeb70640ed10904a9ab`.
  기존 `output/6812/implemented-edf083614/156160455-social-pig-farm-income_001.svg`와
  바이트 단위로 일치한다.
- 기존 7700번 Studio가 새 WASM을 HTTP 200으로 제공하며, 응답과 로컬 파일의 바이트 일치 확인.
- 최종 시각 판정: 메인테이너의 원본 1페이지 PASS. 전체 11쪽 PASS로 확대하지 않는다.

## 3. 보존과 한계

- `task_m100_6812`의 후속 확장 이력(`8a032d96b`)과 기존 review worktree·로컬
  output 증적은 보존한다. 이번 기준본에 후속 확장을 다시 병합하지 않았다.
- Stage 1 조사 기준 `7c67bff3a`는 `task_m100_6812_stage1_baseline` 브랜치와
  `baseline/6812-stage1` 태그로 보존한다. 이는 1페이지 수정 전 코드다.
- 수용한 `edf083614`에도 당시 미완성 확장과 셀 시험 실패가 포함되어 있었다.
  보존된 후속 기록의 해당 후보 검사 결과는 20 PASS / 셀 1 FAIL이다. 최초 완료 판정 시에는
  이를 재실행하거나 시험을 삭제·완화하지 않았다. 이후 승인된 시험 분리와 실제 재검증은
  6절에 기록하며, 전체 회귀 통과로 확대하지 않는다.
- Center/Bottom 합성 문서는 메인테이너가 비정상 문서로 판단하여 조판 판정 근거에서
  제외했다. 이 문서로 도출한 기대값을 일반 규칙 또는 후속 구현의 정답으로 사용하지 않는다.
- MCP PDF의 이미지 누락은 메인테이너 관측으로 기록한다. 그 사실만으로 쪽 수·주변
  배치까지 영향을 받았다고 단정한 해석은 철회했다. 원인 및 영향 범위는 미확정이다.
- 7쪽 표 간격과 셀·복합 inline 등 잔여 문제는 이번 완료 범위 밖이다. 후속 작업은
  정상 문서와 독립 검증 근거를 바탕으로 메인테이너가 선택한 범위에서 진행한다.
  자동으로 신규 자식 이슈를 생성하거나 확장 구현을 재개하지 않는다.

## 4. 원격 절차와 구분

이번 판정은 제한된 타스크 범위의 구현·시각 수용 완료다. push, PR 생성, merge,
GitHub #6812 close는 이번 지시로 실행하지 않았다. 향후 통합을 지시받으면 현재
기준본에 남은 코드·시험의 제출 범위와 해당 검증 게이트를 별도로 확인한다.

## 5. 후속 제출 준비 점검 (2026-09-07)

메인테이너의 다음 절차 승인으로 제출 전 범위·통합 점검을 수행했다. 자체 작업 PR 준비로
문서·Git 워크플로와 self-merge, local-validation, visual-fixture 가이드를 적용했다.
이 단계에서 PR 번호를 예측해 review 문서를 만들거나 원격 상태를 변경하지 않았다.

- 검사 head: `6edf4eb3a` (제품·시험·Cargo는 `edf083614`와 동일).
- `git fetch upstream devel`로 확인한 최신 base:
  `1098e7210452a1bfe536729963844d023b499b5f`.
  앞선 `08bf41c69e` 이후 변경은 의존성 통합 #6838과 CI controller #6820 두 커밋이다.
- `git merge-tree --write-tree HEAD upstream/devel`: exit 0, 충돌 없음.
  결과 tree: `8479a197fa5f31fd69e315992e64866dd51b0f9c`.
  merge tree의 `git diff --check`도 통과했다. 실제 merge나 로컬 devel 전환은 하지 않았다.
- 원본 fixture `samples/issue6797/156160455-social-pig-farm-income.hwp`는
  현재 후보와 merge tree에 포함된다. 다른 worktree에만 의존한 PR이 되는 상황은 아니다.
- 기존 `inline-body-validated-head.log`에서 `edf083614`의 20 PASS / 1 FAIL을
  재확인했다. 이번에 새로 테스트를 실행한 결과는 아니다. 실패 이름은
  `issue_6812_cell_inline_table_respects_its_own_picture_exclusion`이며, 원본을
  바깥 셀과 중첩 TAC 구조로 바꾼 별도 합성 시험이다. 이후 생성한 Center/Bottom
  쪽 분할 시험과는 다른 시험이며, 해당 문서들에 대한 판정을 자동 전용하지 않는다.

제출 준비는 아직 완료되지 않았다. 권고는 제품 코드를 수용 baseline에 유지하면서,
이번 범위 밖인 셀 확장 시험 1건과 그 전용 helper를 보존 이력에 남기고 제출 대상에서
분리하는 것이다. 통과로 위장하는 ignore·기대값 완화는 하지 않는다. 시험 분리의
메인테이너 승인을 받은 뒤 새 제출 후보에서 집중 검사와 필수 검증을 수행한다.
최신 devel의 의존성 변경을 포함한 컴파일·회귀는 아직 확인하지 않았으며,
텍스트 병합 무충돌을 그 검증의 대체물로 사용하지 않는다.

## 6. 승인된 셀 시험 분리와 검증

2026-09-07 메인테이너가 5절의 시험 분리를 승인했다.
`tests/cases/issue_6812_square_picture_tac_table.rs`에서
`issue_6812_cell_inline_table_respects_its_own_picture_exclusion` 1건과
그 시험에서만 사용하는 `collect_nested_pair`를 제출 후보에서 제외했다(91줄).
제품 코드는 바꾸지 않았고, 나머지 20건의 시험과 기대값은 유지했다.
제외 전 원문은 `edf083614` 및 현재 브랜치의 부모 이력에 정확히 보존되며,
후속 확장 작업은 별도 보존 브랜치 `task_m100_6812`에 남아 있다.
이는 셀 문제의 해결 또는 시험 PASS가 아니라 승인된 제출 범위 분리다.

### 6.1 실제 검증 결과

- 제출 후보: `0c23e325a230b5ef49999a859afe783527fdbbde`.
- 검증 checkout: `/home/edward/mygithub/rhwp-review-6812` (위 후보의 detached HEAD).
- 고정 target: `/home/edward/mygithub/rhwp-6812-review-target` 재사용.
- `node scripts/rust-test-suite-manifest.mjs --prepare`: PASS. 파생 suite는 검증
  worktree에만 준비했으며 커밋하지 않았다.
- `node scripts/run-rust-test.mjs --cargo-test issue_6812_square_picture_tac_table --
  --target-dir /home/edward/mygithub/rhwp-6812-review-target`: **20 PASS / 0 FAIL /
  0 ignored**, 148 filtered. 컴파일 1분 12초, 실행 1.72초.
- 다음 묶음은 동일 target에서 순차 실행했으며 전체 명령 체인 exit 0을 확인했다.
  - `cargo fmt --all` 및 `cargo fmt --all -- --check`: PASS.
  - native `cargo clippy --locked … -- -D warnings`: PASS (45.37초).
  - `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown … -- -D warnings`:
    PASS (1분 13초).
  - `cargo build --locked --workspace …`: PASS.
  - `cargo clippy --locked --workspace --all-targets … -- -D warnings`: PASS (1분 29초).
  - `node scripts/rust-test-suite-manifest.mjs --check`: PASS
    (1,180 sources / 5,011 static test attrs / 48 integration targets).
  - `node scripts/rust-unit-test-tiers.mjs --check`: PASS (4,205 tests / 298 modules).
  위 Cargo 명령의 `…`는 모두 `--target-dir /home/edward/mygithub/rhwp-6812-review-target`이다.
- 검증 종료 후 review worktree의 tracked/untracked 변경 없음.
  `git diff --exit-code edf083614 -- src crates Cargo.toml Cargo.lock`도 exit 0이다.
  Studio에 제공 중인 승인 baseline WASM은 변경하지 않았다.

### 6.2 남은 제출 게이트

이번 결과는 범위 분리 후 집중 검사·lint 통과이며 **PR 준비 완료가 아니다**.
release-test 전체 nextest, Native Skia 3종은 이번 실행에 포함하지 않았다.
최신 `upstream/devel` 의존성 변경의 실제 병합도 아직 하지 않았다.
다음 권고 순서는 최신 원격 재확인 → 작업 브랜치에 devel 통합 → 통합 후보의
필수 lint·전체 회귀·Native Skia·Docker WASM 및 원본 시각 재확인이다.
의존성 통합 전후에 긴 전체 회귀를 중복 실행하지 않도록 통합 승인을 먼저 받는다.
기존 baseline/확장 보존 브랜치는 그대로 유지하며, push·PR 생성·merge·issue close는
별도 승인 전에는 수행하지 않는다.
