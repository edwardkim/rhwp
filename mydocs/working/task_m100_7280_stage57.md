# Task #7280 Stage 57 — 최신 devel 통합과 분리 모듈 반영

- Issue: #7280. 이전: [Stage56](task_m100_7280_stage56.md).
- 승인: PR 생성 전에 devel을 merge하고 baseline 이후 typeset 변경을 흡수한다.
- 시작 head: `933bdb6915bd4652600af20ee0853f476c6693b8`.
- 통합 base: `1966af77fa8046c844d654b157b5168baad8a30e` (`upstream/devel`).
- 원래 구조 비교 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`는 보존한다.
- 상태: **로컬 merge 및 통합 head 집중 검증 완료, 최종 제출 gate 대기**.
  원격 push·PR·댓글은 수행하지 않는다.

## 통합 범위와 실제 연결

시작 head 대비 원격 전용 47개 commit을 통합한다. 원래 baseline 이후 원격의
typeset 영역 변경은 root `typeset.rs` 4개 hunk(78행 추가/2행 삭제)다.
Git 충돌은 해당 root 한 파일에서 발생했다. 기존 추출 구조를 보존하고 다음처럼 반영했다.

| 원격 변경 | 통합 위치와 소비 경로 |
| --- | --- |
| #6761 저장 되감김의 쪽 상단/직전 흐름 위치 일치 helper 2개 | root의 기존 저장 위치 helper 옆, 원격 본문 그대로 |
| #6761 채움률 또는 위치 일치 판정 | `paragraph/whole_fit.rs::inspect`; 기존 점유 높이와 평가 순서를 유지 |
| #6761 분할 진입도 동일 되감김 판정 소비 | `WholeFitEvidence` → `WholeFitDecision` → `paragraph/flow.rs` → `place_after_failed_fit` → `split_entry::should_advance` |
| #6656 pagination 예약 높이 유지 설명 | `paragraph/format.rs`; 계산 변경 없이 원격 주석 반영 |

되감김 결과는 Hangul2024 override를 적용한 기존 시점의 bool을 전달하며 분할 진입에서
재계산하지 않는다. overflow 판정은 원격과 같이 `current_height`를, 위치 일치 판정은
`visible_float_exclusions`를 포함한 점유 높이를 사용한다. 신규 상수·예외·공개 API는 없다.
원격의 다른 layout/height_measurer/parser/serializer/Studio/CI/테스트·기준값 변경은 그대로
통합하며 이 작업에서 통과를 목적으로 baseline이나 ignore를 추가 변경하지 않는다.

## 검증 기록

통합 제품 SHA: `6003fe35689ee9be02d2b05737430e648b07361e`.
merge 부모는 시작 head와 위 `upstream/devel`이며 원격 base의 ancestor 포함을 확인했다.
이전 제품 `7947ee45f`의 R5 결과는 통합 head의 결과가 아니다.
주 작업트리의 과거 파생 suite에 삭제된 test 참조가 남아 있어 최초 `cargo fmt --all -- --check`가
완주하지 못했다. 파생 파일을 source 변경으로 커밋하지 않고 별도 review worktree에서
현행 `--prepare` 후 다시 검증한다.

clean review worktree `rhwp-review-7280-r3k`를 통합 SHA에 고정하여 실행했다.
Rust 1.93.1, 고정 target `target/pr-review`, host 16 CPU/31GiB RAM에서 동시 Rust 작업이
없음을 확인하고 build jobs 4, test threads 8로 순차 실행했다. 원래 baseline worktree는 보존했다.

| 검사 | 결과 |
| --- | --- |
| 원격 typeset 변경 4개 hunk 대조 | PASS. helper 본문 일치, whole-fit/split 조건과 주석 보존; DPI receiver/들여쓰기만 정규화 |
| `cargo fmt --all -- --check` | PASS (현행 suite 준비 후) |
| manifest `--check --base-ref 1966af77…` | PASS. 1,399 sources / 28 suites + 20 exceptions |
| unit-tier `--check --base-ref 1966af77…` | PASS. 4,205 tests / 298 modules / cfg support 28 |
| Native `cargo clippy --locked … -- -D warnings` | PASS |
| `cargo nextest run --locked --cargo-profile release-test … --tests -E … --test-threads 8 --no-fail-fast` | **403 passed / 0 failed** |

집중 검사는 기존 395건과 새 #6761·#6656의 8건이다. 기존 suite 번호를 재사용하지 않고
현행 전체 target에 이름 필터를 적용했다. nextest의 9,784 skipped는 집중 필터에서 제외된
범위로, 전체 회귀 통과나 ignore 9,784건을 의미하지 않는다. nextest 0.9.137에 대한 저장소
권고 0.9.140 경고가 있었으나 명령은 exit 0으로 완료했다.

특히 시장구조 문서의 4쪽 본문 경계/5쪽 머리, 같은 쪽 부분 후퇴 반례,
덜 찬 쪽의 경계 승격/흐름 위치 불일치 반례를 원격의 기존 실물 테스트로 검사했다.
신규 결함 수정이나 수정 전 FAIL 재입증을 수행했다는 주장은 하지 않는다.

재현 명령과 로그: `output/7280/stage57-devel-merge/validate.sh`, `prepare.log`, `fmt.log`,
`manifest.log`, `unit-tier.log`, `clippy-native.log`, `nextest-focused.log`.
정적 대조는 `check-absorption.mjs` / `absorption.json`이다. 정적 일치와 판정 전달 코드 검토는
시각 동등성 증거가 아니다. review의 파생 suite/manifest는 미추적 검증 산출물로만 남겼으며
주 작업트리와 review worktree의 tracked 변경은 없음을 확인했다.

최신 upstream 대비 `src/`, `crates/`, `tests/`, Cargo manifest/lock의 차이는 typeset 구조 영역에만
남는다. 원격 변경의 테스트·기준값을 이 통합에서 추가 수정하지 않았다.

전체 제출 gate와 Native/fresh WASM 시각 대조는 통합 후 정확한 head에 대해 별도로 확인해야 한다.
미실행: 전체 release-test 회귀, WASM/workspace Clippy, workspace build, Native Skia 및
새 upstream와 통합 head의 Native/fresh WASM 출력 대조. 이번 단계에서 Studio pkg를 갱신하지 않았다.
이번 merge만으로 제출 준비 완료나 시각 동등성을 선언하지 않는다.
