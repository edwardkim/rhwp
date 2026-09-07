---
kind: report
status: active
canonical: mydocs/plans/task_m100_6812.md
issue: 6812
last_verified: 2026-09-08
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

## 7. 승인된 최신 devel 통합과 제출 검증 (2026-09-07)

메인테이너가 최신 devel 통합 후 전체 회귀·Native Skia·Docker WASM 검증을 승인했다.

### 7.1 통합 후보와 보존

- 통합 전 HEAD: `3e3a88055`. tracked/untracked 변경 없음.
- `git fetch upstream devel`로 재확인한 원격:
  `1098e7210452a1bfe536729963844d023b499b5f` (5절 이후 추가 변경 없음).
- merge-tree 사전 점검 exit 0 후 `git merge --no-ff upstream/devel` 실행.
  충돌 없이 생성된 merge commit: `5c55848a6ed386960480eafd98924fa5e53dcfa3`.
- 작업 브랜치는 `task_m100_6812_edf083614_baseline`을 유지했다. 로컬 devel을
  별도로 전환·이동하거나 원격에 push하지 않았다.
- 수용한 `edf083614` 대비 제품 소스 차이는 devel에서 유입된 `src/parser/mod.rs`의
  압축 호환 보정과 `src/renderer/skia/renderer.rs`의 Skia 호환 보정이다.
  #6812 줄 배치 구현을 추가 수정하거나 보존된 셀 확장 구현을 다시 넣지 않았다.
- 보존 브랜치 `task_m100_6812@8a032d96b`,
  `task_m100_6812_stage1_baseline@7c67bff3a`는 그대로다.

### 7.2 통합 후보의 실제 게이트

Rust 검증은 기존 review worktree를 `5c55848a6` detached HEAD로 전환하고
기존 고정 target을 재사용했다. 모든 Cargo 명령은 순차 실행했다.
호스트는 WSL2 Linux, Rust 1.93.1, nextest 0.9.137, 논리 CPU 16개다.
전체 nextest와 Skia lib의 테스트 동시성은 가용 메모리 약 26GiB를 확인한 뒤 8로 설정했다.

| 게이트 | 결과 |
| --- | --- |
| suite prepare·manifest check·source-side unit tier check | 모두 PASS, 파생물은 미커밋 |
| fmt 적용·fmt check | PASS |
| native Clippy / WASM lib Clippy | PASS / PASS (59.72초 / 52.77초) |
| workspace build / workspace all-targets Clippy | PASS / PASS (1분 25초 / 1분 21초) |
| release-test 전체 nextest | **9,143 PASS / 0 FAIL / 46 skipped**, 78 binaries. 컴파일 5분 26초, 실행 321.641초 |
| #6812 집중 시험 | 위 전체 실행에 포함된 **20 PASS**. 분리한 셀 시험은 포함하지 않음 |
| Native Skia lib | **4,112 PASS / 0 FAIL / 13 ignored** (root 3,930 + 내부 crate 15/165/2) |
| Native Skia 이미지 누락 대체 표시 | **2 PASS / 0 FAIL**, 비대상 168 skipped |
| Native Skia 직접 PDF export | **4 PASS / 0 FAIL**, 비대상 168 skipped |
| Docker 표준 WASM | **PASS**, 최적화 포함 6분 56초 |

전체 회귀 명령은 `cargo nextest run --locked --cargo-profile release-test
--target-dir /home/edward/mygithub/rhwp-6812-review-target --tests --test-threads 8
--no-fail-fast`다. IR/overflow dump 환경변수를 함께 지정해 같은 실행에서 계측했다.
Skia 3종은 local-validation의 lib 및 두 source wrapper 명령에 같은 target과
`--features native-skia`를 사용했다.

nextest는 권장 버전 0.9.140보다 낮다는 경고와 `report-skipped` 설정 미인식 경고를 냈다.
최소 요구 버전 0.9.91은 충족했고, 실제 전체 실행 및 위 PASS/skip 집계를 확인했다.
검증 도중 도구 버전이나 timeout·skip 정책·시험 기대값을 바꾸지 않았다.
대형 CellBreak #2063 검사는 215.731초에 PASS했다. 이는 이번 실행 시간이지
변경 전후 성능 비교 또는 성능 개선의 증거는 아니다.

### 7.3 새 fixture의 기준선 검사

- 원본 fixture SHA-256은 2절의 입력과 같은
  `1b99b763aac36a14a9f463e35ee894a23eb1083780040eab5e0f02a481c694b8`이다.
- IR field sweep: PASS. 기존 TSV 568행 대비 현재 dump 250행이며 새 행·증가 없음.
  318행 감소를 이번 #6812의 개선으로 귀속하지 않고, 무관한 기준선 일괄 변경도 하지 않았다.
- overflow-cell: 16개 partition dump를 정렬·병합한 결과 기존 12행과 완전히 일치.
- 두 dump 모두 새 `issue6797/` fixture의 비영 발산/overflow 행은 없다.
  따라서 이 샘플을 위해 기준선 행을 억지로 추가하거나 회귀를 허용하지 않았다.

### 7.4 WASM·승인 출력 보존 확인

메인 checkout에서 `docker compose --env-file .env.docker run --rm wasm`을 실행했다.

- 새 WASM: 10,316,029 bytes,
  SHA-256 `10d8823ab1a2cae0d90dcc491ecd6c2c42fe3caea6865d9c2f37f3c282927042`.
- 원본 로드 11쪽, 1페이지 SVG 338,679 bytes.
- SVG SHA-256:
  `b4703e335a54c1200f5455c7de5622a384a3edb3ddd7caeb70640ed10904a9ab`.
  메인테이너가 승인한 `edf083614`의 SVG와 **바이트 단위 일치**.
- 기존 Studio 7700번 HTTP 200. Studio가 제공하는 WASM 응답과 새 로컬 WASM도 바이트 일치.
  서버를 재시작하거나 다른 review 서버를 바꾸지 않았다.
- 이는 수용된 1페이지 출력 보존의 기계적 확인이다. 이번에 메인테이너의 새 시각 판정을
  받은 것으로 기록하지 않으며, 전체 11쪽의 시각 PASS로 확대하지 않는다.

메인 checkout의 `output/6812/integration-5c55848a6/`에 lint·nextest·Skia·Docker 로그,
IR/overflow dump, 검증 스크립트·JSON과 `156160455-social-pig-farm-income_001.svg`를 보관했다.
생성물은 Git에 포함하지 않았다. 검증 종료 후 review worktree 변경 없음 및
`git diff --check upstream/devel...HEAD` PASS를 확인했다.

### 7.5 다음 승인 경계

최신 devel을 통합한 후보의 이번 필수 검증은 통과했다. 다음은 원격 push와 devel 대상
PR 생성이며 별도 승인이 필요하다. push 직전에는 원격 devel을 다시 확인한다.
PR에는 #6812 원본 1페이지 해결, 셀 시험의 승인된 범위 분리, 기존 skip/ignore 수,
7쪽 및 셀 확장 미해결 범위를 숨기지 않고 기록한다. PR CI·self-review·병합·이슈 close는
아직 실행하지 않았다.

## 8. push 승인 후 원격 재확인 — 새 조판 통합으로 제출 일시 보류

2026-09-07 메인테이너가 원격 push·PR 생성을 승인했다. 실제 push 전에 fetch한 결과,
원격 devel은 기존 검증 base `1098e72104`에서
`a3a30d99d4aefb15ed0dbed96645eb1ca3595099`로 전진했다.
[PR #6847](https://github.com/edwardkim/rhwp/pull/6847)은 planet6897 렌더링 PR 10건과
메인테이너 보정을 통합했으며, GitHub 기록상 2026-09-07 21:42:10 KST에 병합됐다.

- 신규 원격 변경: 105개 파일, +5,760/-69줄 및 fixture/PDF/시각 증적.
- #6812와 수정 파일이 겹치는 핵심 경로는 `float_placement.rs`, `layout.rs`,
  `layout/paragraph_layout.rs`, `typeset.rs`다. 단순 문서·의존성 변경이 아니다.
- #6797/#6798의 같은 원본 문서 7쪽 처리도 이 통합에 포함된다. 그 문제를 이번
  #6812에서 새로 해결했다고 주장하거나 이미 들어온 기여자 수정을 다시 복제하지 않는다.
- 원본 fixture는 두 head에서 Git blob `86d355d678dcbeaae2f10fb11b5cacf61ec580c1`로
  일치한다. 샘플의 add/add 내용 충돌은 없다.
- 현재 `f8bfcc4ea`와 새 devel의 `git merge-tree --write-tree`는 exit 0,
  결과 tree `3602fb884c7d61f3ae9569281e6db3e7c321b8aa`다. 이는 텍스트 병합
  가능성만 확인한 것이며 조판 규칙 간 상호작용 검증이 아니다.
- 기존 7절 검증은 `5c55848a6`에 대해서 유효하다. 새 devel과의 통합 후보에 대한
  통과 증거로 전용하지 않는다. 같은 head의 원격 PR은 아직 없으며 push·PR 생성도
  실행하지 않았다. 기존 Studio WASM과 수용 기준본은 유지했다.

권고는 새 devel을 작업 브랜치에 통합하고 #6812와 유입된 인접 회귀를 먼저 검사한 뒤,
필수 전체 게이트 및 원본 1페이지 출력 보존을 다시 확인하는 것이다. 새로운 조판 변경과의
통합·재검증 승인을 요청한다. 실패가 발견되면 원인을 분리해 보고하며, 완료 범위 밖 셀
확장이나 7쪽 구현을 임의로 재개하지 않는다.

## 9. #6847 통합 후 재검증 완료 (2026-09-07)

메인테이너가 새 devel 통합 → 집중 검사 → 전체 게이트·1페이지 보존 확인 → push·PR
생성을 승인했다. 시작 시 원격은 `a3a30d99d4`로 유지되어 있었고, 충돌 없이 생성한
통합 후보는 `51ed17d3b9c1d7088566557b79eef5850da59231`이다.
조판 소스나 시험 기대값을 추가 보정하지 않았다. 원본 fixture는 이미 devel에 동일
바이트로 포함되어 이번 PR의 신규 HWP 파일이 아니다.

### 9.1 실제 검증

기존 review worktree를 위 후보로 전환하고 suite를 준비했다. 고정 Cargo target은
유지했고, 각 target의 Cargo 명령은 순차 실행했다. 마지막 Native Skia와 Docker WASM만
서로 다른 checkout·target에서 병행했다. 전체 nextest와 Skia lib의 동시성은 8이다.

| 검증 | 결과 |
| --- | --- |
| #6812 + #6778/#6782/#6787/#6790/#6793/#6795/#6797/#6800/#6803/#6837 집중 검사 | **68 PASS / 0 FAIL**, 0.649초. 9,169건은 집중 필터 비대상/기존 skip |
| fmt 적용·check, native·WASM32·workspace all-targets Clippy, workspace build | 모두 PASS |
| manifest / source-side unit tier check | PASS / PASS (1,191 sources, 48 integration targets / 4,205 tests) |
| release-test 전체 nextest | **9,191 PASS / 0 FAIL / 기존 46 skipped**, 실행 345.793초 |
| Native Skia lib | **4,112 PASS / 0 FAIL / 기존 13 ignored** |
| Native Skia 이미지 누락 대체 표시 | **2 PASS / 0 FAIL**, 비대상 175 skipped |
| Native Skia 직접 PDF export | **4 PASS / 0 FAIL**, 비대상 191 skipped |
| Docker WASM | **PASS**, 최적화 포함 7분 55초 |

전체 nextest 및 Skia 명령은 7.2절과 동일하며 새 후보에서 다시 실행한 결과다.
집중 검사는 전체 nextest 명령에
`-E 'test(/issue_(6812|6778|6782|6787|6790|6793|6795|6797|6800|6803|6837)_/)'`를
추가했다. 최종 전체 실행에는 필터를 사용하지 않았다. 기존 nextest 권장 버전 경고는
동일하며 도구 버전·skip·timeout 정책을 바꾸지 않았다.

IR dump는 새 devel 원장 571행 대비 253행이며 새 행·증가 없이 기존 318행 감소만 남았다.
overflow dump는 기존 12행과 일치한다. 원장 파일을 갱신하거나 무관한 감소를 이번 개선으로
귀속하지 않았다. #6812 원본의 신규 비영 IR/overflow 행은 없다.

### 9.2 승인된 출력 보존과 Studio

- 새 WASM: 10,334,418 bytes,
  SHA-256 `66d053824d13a5c9f40f179ea984509f0017dcb5b257bf703271d04a49851bce`.
- 원본은 기존 SHA-256과 동일하며 11쪽으로 로드됐다.
- 1페이지 SVG: 338,679 bytes,
  SHA-256 `b4703e335a54c1200f5455c7de5622a384a3edb3ddd7caeb70640ed10904a9ab`.
  승인된 `edf083614` 출력과 **바이트 단위 동일**하다.
- 7700번 Studio와 WASM 응답은 HTTP 200이고, 제공 WASM은 로컬 새 산출물과 동일하다.
- 검증 로그·dump·SVG·JSON·재현 스크립트는 메인 checkout의
  `output/6812/integration-51ed17d3b/`에 보관했다. 생성물은 커밋하지 않는다.

8절의 새 devel 통합 미검증 상태는 위 결과로 해소했다. 원본 1페이지에 대한 기존
메인테이너 시각 수용을 유지하며, 전체 11쪽 시각 승인이나 셀 확장 완료를 주장하지 않는다.
원격 push·PR 생성은 이번 승인 범위에 포함되지만 CI 완료·최종 self-review·병합·이슈 종료는
아직 미수행이다. PR 채번 후 별도 번호 기반 review 기록과 오늘할일을 같은 PR에 포함한다.

## 10. #6848 추가 통합 및 최종 제출 검증 (2026-09-08)

9절 종료 직전 원격 devel에 #6848이 추가 병합되어
`ac8c9fa2c9bfcaadb74f3b46a8ec2a879c3a8099`로 전진했다. 에이전트가 fetch 결과를
판정하기 전에 후속 push를 실행한 순서 오류로, `262f7c38f`가 원격 작업 브랜치에 먼저
올라갔다. PR은 만들지 않았고 이 오류를 메인테이너에게 즉시 보고했다.
이는 devel 직접 push나 병합이 아니며, 이전 검증 결과를 새 devel의 통과로 주장하지 않았다.

승인된 최신 통합·검증·push·PR 생성 순서를 계속 적용했다. 추가 통합에서는
오늘할일 문서 한 곳의 add/add 내용 충돌을 양쪽 기록 모두 보존하여 해소했다.
제품 소스는 자동 병합됐으며 별도의 조판 보정은 하지 않았다.
최종 검증 코드 commit은 **`380f838300db555d511a7267421fd65d317dc111`**이다.

### 10.1 실제 결과

| 검증 | 결과 |
| --- | --- |
| #6812·인접 조판·새 object 속성/수식/gradient 집중 nextest | **92 PASS / 0 FAIL**, 실행 0.953초. 필터 비대상/기존 skip 9,172건 |
| fmt / native·WASM32·workspace all-target Clippy / workspace build | 모두 PASS |
| manifest / source-side unit tier | PASS / PASS (1,195 sources / 4,205 unit tests) |
| release-test 전체 nextest | **9,218 PASS / 0 FAIL / 기존 46 skipped**, 실행 427.848초 |
| Native Skia lib | **4,112 PASS / 0 FAIL / 기존 13 ignored** |
| Native Skia 이미지 대체 표시 / 직접 PDF export | **2 PASS / 4 PASS**, 비대상 skip 각각 179 / 191 |
| Studio npm test / TypeScript noEmit | **1,493 PASS / 0 FAIL / 기존 2 skipped** / PASS |
| Docker WASM | **PASS**, 최적화 포함 8분 31초 |

Cargo는 기존 review worktree·고정 target에서 순차 실행했다. Docker는 메인 checkout의
별도 target에서 병행했다. Studio 검사는 새 devel에서 유입된 그림 Undo 코드를 추가로
확인한 것이며 이번 PR의 Studio 변경이라고 분류하지 않는다. 위 실행 시간은 병행 작업과
호스트 부하를 포함한 검증 소요 시간이지 통제된 성능 비교가 아니다.

새 local-validation §4.3.1을 읽고 네 코퍼스 dump를 같은 전체 nextest에서 수집했다.
IR 571→253행(318행 감소), overflow 12→12행, off-canvas 83→70행,
text-overlap 162→152행이며 신규 문서·수치 증가가 없다. 원장이나 기대값은 바꾸지 않았다.
전체 회귀에 포함된 oracle page count와, 외부 controlset이 필요한 clipping 검사를 구분한다.
clipping 별도 실행은 하지 않았고 해당 gate를 통과했다고 주장하지 않는다.
최신 base 대비 신규/변경 HWP·HWPX·HML·PDF가 없으므로 신규 fixture 보안 검사 입력 대상은 없다.

### 10.2 수용 출력 보존

- WASM 10,352,999 bytes,
  SHA-256 `c22fa7685f3d0b6b66b8efbc69323bec1ae152164bf95684aab7952224af558f`.
- 동일 원본 11쪽 로드, 1쪽 SVG 338,679 bytes.
- SVG SHA-256 `b4703e335a54c1200f5455c7de5622a384a3edb3ddd7caeb70640ed10904a9ab`로
  메인테이너가 승인한 `edf083614` 출력과 바이트 단위 동일.
- Studio 7700번 WASM HTTP 200 및 새 로컬 파일과 일치 확인.
- 메인 checkout의 `output/6812/integration-380f83830/`에 전체 로그, 네 dump,
  SVG, WASM 검증 JSON·스크립트를 보관했다. Git 생성물은 추가하지 않았다.

이전 head의 push 이후 새 통합 후보 검증을 완료했으며, 검증 코드는 변경하지 않고 보고서만
추가해 같은 원격 작업 브랜치를 fast-forward 갱신한다. 원격 devel SHA 일치와 ancestry를
push의 명시적 선행 조건으로 확인한다. PR 생성 뒤 번호 기반 접수 기록을 추가하되,
최종 self-review·CI 통과·병합·이슈 종료를 미리 완료로 기록하지 않는다.
