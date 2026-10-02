# Task #7280 — 조판 책임 분리 최종 로컬 결과보고

- 일자: 2026-09-23. [이슈 #7280](https://github.com/edwardkim/rhwp/issues/7280).
- 상태: **승인 범위 R1–R6 구현·devel 통합·로컬 검증 완료. 원격 제출·CI·review·merge는 미실행**.
- 수행 근거: [수행계획](../plans/task_m100_7280.md), [구현계획](../plans/task_m100_7280_impl.md).
- 기여자용 정본: [조판 코드의 책임 경계와 변경 지도](../tech/typesetting_architecture.md).

## 결과와 범위

문단을 출발점으로 section → paragraph/controls/table/notes → state의 책임을 분리했다.
HWP/HWPX의 공통 IR, 기존 공개 API, 조건·상수·평가 순서와 조판 결과는 보존 대상이다.
기여자가 추가·수정·삭제할 위치와 실제 소비 경로를 찾는 구조가 목적이며 개별 조판 결함을
고치거나 기존 예외를 올바른 일반 규칙으로 재승인한 작업이 아니다.

- Query 결과와 흐름 조정, 상태 Command의 책임을 구분했다. `TypesetState`는 private data와
  불변 Deref만 제공하며 가변 접근자 없이 좁은 명령으로 갱신한다.
- 표의 준비·컷 스캔·조각 방출·이어받기와 Native/WASM 재개 수명을 분리했다.
  행 스캔 진행 상태와 구역 순회 변수는 각각의 소유자에 남긴다.
- SOLID/CQRS 지침을 적용하되 동적 rule engine, 새 IR 복사본, 별도 정책 서비스를 만들지 않았다.
- 구조 정본에 대표 의미 ID 8개, 입력/결과/소비 경로/검사, 잔여 의존과 기여 절차를 연결했다.
  기존 private 테스트는 유지했고 새 source-side 테스트나 파생 suite를 제출하지 않는다.

## 기준과 devel 변경 흡수

| 구분 | SHA |
| --- | --- |
| 최초 devel baseline | `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db` |
| 최종 통합·정책 비교 devel | `1966af77fa8046c844d654b157b5168baad8a30e` |
| 제품 merge | `6003fe35689ee9be02d2b05737430e648b07361e` |
| 실제 빌드·전체 검증 head | `29130d5397da3dc3cd3d1d9416b153584e1a4f41` |
| Stage58 검증 기록 commit | `9023492de84521c9eef5c6778053b7425bb8dc28` |

검증 head 이후에는 문서와 시각 증적만 추가한다. 제출 SHA와 실행 SHA를 같다고 표현하지 않는다.
원격 제출 직전에는 base/head를 재확인하며 새 제품 변경에는 기존 결과를 그대로 적용하지 않는다.

[Stage57](../working/task_m100_7280_stage57.md)에서 원격 47개 커밋을 병합했다.
typeset 충돌 4개 hunk를 분리된 책임 위치로 흡수했으며 #6656 예약 높이 설명과 #6761 저장 vpos
경계 판단을 보존했다. `WholeFitEvidence → WholeFitDecision → paragraph/flow →
place_after_failed_fit → split_entry::should_advance`로 확정 판단을 전달한다.
Hangul 2024 override 뒤의 값과 occupied/current height의 구별을 유지한다.
제품 diff는 typeset 영역 109개 파일에 한정되며 `crates/`, `tests/`, Cargo 설정·lock 변경은 없다.

## 검증 결과

실행 명령·재시도 이력·원시 증적 위치는 [Stage58](../working/task_m100_7280_stage58.md)에 있다.
별도 clean review worktree에서 고정 target `target/pr-review`, 빌드 동시성 4, 테스트 동시성 8로 실행했다.

| 게이트 | 결과 |
| --- | --- |
| prepare, fmt, Native/WASM/workspace-all-target Clippy, workspace build | 모두 PASS |
| manifest / unit-tier 정책 검사, `--base-ref 1966af77…` | PASS: 1,399 sources / 48 targets; unit 4,205 / 298 modules |
| 통합 후 집중 검사 | 403 PASS |
| 전체 release-test nextest | **10,137 PASS / 0 FAIL / 기존 제외 50** |
| Native Skia lib | 4,112 PASS / 0 FAIL / 기존 ignored 13 |
| Native Skia placeholder / direct PDF | 2 PASS / 4 PASS |
| Native base/head 출력 | 14문서 772쪽의 소유·컷, render tree, SVG 동일 |
| fresh Docker WASM base/head 출력 | 11문서 748쪽의 render tree, SVG 동일 |
| 선택 23쪽 PNG | Native 전후·WASM 전후·통합 Native/WASM 모두 byte 동일 |

전체 회귀와 Skia는 중복되는 별도 feature 검사다. focused 이름 필터를 기존 제외 50에 더하지 않는다.
nextest 0.9.137은 권고 0.9.140과 다르고 `report-skipped` 설정 경고가 있었다. 실행 범위나 기준값은
완화하지 않았다. 최초 workspace build의 디스크 부족은 환경 실패로 구분하고 승인된 증분 캐시만
정리한 뒤 재시도했다. 이 결과는 원격 PR CI 통과나 모든 feature의 완전 검증을 뜻하지 않는다.

### 재현 명령

아래는 기존 실행의 인자이며 source checkout에서 파생 suite를 생성하지 않는다.
review worktree에서 검증 SHA를 고정하고 `CARGO_BUILD_JOBS=4`를 사용했다.

```bash
rhwp_review_base_sha=1966af77fa8046c844d654b157b5168baad8a30e
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir /home/edward/mygithub/rhwp/target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check --base-ref "$rhwp_review_base_sha"
node scripts/rust-unit-test-tiers.mjs --check --base-ref "$rhwp_review_base_sha"
cargo nextest run --locked --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --tests --test-threads 8 --no-fail-fast
cargo test --locked --profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --lib -- --test-threads 8
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia
docker compose -p rhwp --env-file .env.docker run --rm --no-deps -e CARGO_BUILD_JOBS=4 -e BINARYEN_CORES=4 wasm
```

시각 산출은 `output/7280/stage58-integration/`의 `native.mjs`, `compare-native.mjs`,
`compare-visual.mjs`와 `targets.json`으로 입력·PDF·쪽을 고정했다.
주 작업트리의 Studio pkg는 교체하지 않았다. Studio 코드·npm/editor·Undo/Redo 변경은 없어
해당 frontend 변경 게이트는 비해당이며 fresh WASM 출력 비교와 혼동하지 않는다.

## 시각 증거와 판정 경계

[보존 증적](../pr/assets/issue_7280_typeset_refactor/README.md)에 Native/fresh WASM의
대표 review·standalone overlay와 실제 입력/PDF를 연결했다. 14개 원본과 11개 PDF는 기존
base와 검증 head에 모두 있으며 현재 파일의 바이트와 Git blob(또는 LFS oid)이 일치함을 재확인했다.

표·문단 흐름, 글꼴·줄바꿈·표 외곽을 직접 판독한 범위와 전체 자동 동일성 검사를 구분한다.
**전후 동일성은 충족**, 다음 한컴 피델리티 차이의 해소는 **비해당/미해결**이다.

- square-host 1쪽: 표/본문 흐름은 유사하나 글꼴 굵기·자형 차이가 남는다.
- chemical-rewind 13·14쪽: 글머리표·본문 위치, 표가 PDF보다 위에 놓이는 차이가 남는다.
- market-rewind 5쪽: PDF의 `제5절 소결`이 rhwp에서는 다음 쪽에 있다.
- endnote-between20 22쪽: 문단 시작·수식·그림 위치 차이가 남는다.

모두 고정 devel과 통합본의 출력이 같지만, 그것을 올바른 조판의 근거로 승격하지 않는다.
나머지 선택 쪽의 자동 동일성을 사람의 전수 시각 통과로 보고하지 않는다. 모든 문서·편집 재조판
조합과 네 줄 조회 helper의 모든 입력 경계는 전수 검증하지 않았다. 이번 주장은 구조 이동의
동작 보존이지 신규 결함의 수정 전 FAIL/수정 후 PASS 주장이 아니다.

## 구조 효과와 한계

[Stage56 계측](../working/task_m100_7280_stage56.md)은 최초 baseline과 **통합 전 제품
`7947ee45f`**의 비교다. 아래 CC 수치를 통합 후 head의 새 계측으로 표시하지 않는다.

| 범위 | 이전 → 리팩토링 후 |
| --- | --- |
| root typeset.rs 줄 수 | 31,937 → 7,902 (devel 통합 후 실제 7,952) |
| typeset 영역 파일 수 / 총 줄 수 | 2 → 109 / 32,037 → 38,891 |
| 영역 최대 CC / CC > 25 함수 | 186 → 88 / 9 → 7 |
| 영역 CC > 15 함수 / 1,200줄 초과 파일 | 15 → 21 / 1 → 4 |

입출력 타입과 책임 경계가 늘어 총 줄 수와 일부 중간 복잡도 항목은 증가했다.
남은 큰 함수·layout/document_core 의존·root helper/private 테스트가 있으며 완전한 단방향
의존이나 facade-only root는 아니다. 기존 호환 예외의 사양 재입증도 완료했다고 주장하지 않는다.
CPU·메모리 성능 비교는 **미측정**이며 CC 감소를 성능 개선으로 보고하지 않는다.

## 제출과 종료

[PR 준비 계획](../plans/task_m100_7280_pr.md)에 본문과 생성 절차를 준비했다.
별도 승인 후 base/head 확인 → 필요한 재검증 → push → devel 대상 Open PR → 원격 CI와
review → merge·이슈 종료 순서다. 제출 전까지 #7280은 OPEN을 유지한다.
