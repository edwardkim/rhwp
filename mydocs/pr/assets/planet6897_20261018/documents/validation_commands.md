# 최종 후보의 실행 명령과 로컬 검증

- 전체 Rust 검증 head: `3cb5a4c290663343123074be1e2fc0dda18d3db4`; 추가 완료 페이지 검사·lint head: `e25dbd0f5d47bc32686016c08054d2bf718be6a7`; 실제 runtime: `3cb5a4c290663343123074be1e2fc0dda18d3db4`; base: `a6e9d18fc1061318055b4eb348daec201427ea03`.
- 아래 16개 Cargo·정책 단계는 고정 `target/pr-review`를 공유하며 순차 실행했습니다. Cargo jobs 8, 전체 nextest test threads 12이며 일반 검증에서 incremental을 강제로 끄지 않았습니다.
- 실제 runtime 소스 해시를 다시 대조했습니다. 추가 회귀는 테스트·증적·계획 문서만 바꿨으며 runtime 소스 2,744개의 바이트를 유지했습니다. GitHub CI·merge는 해당 원격 head에서 별도로 확인합니다.

## Rust 전체 검증

| 단계 | 실제 명령 | 결과 |
| --- | --- | --- |
| manifest-prepare | `node scripts/rust-test-suite-manifest.mjs --prepare` | exit 0 |
| release-build | `cargo build --locked --release --target-dir target/pr-review` | exit 0 |
| release-lib | `cargo test --locked --release --target-dir target/pr-review --lib -- --test-threads=12` | exit 0 |
| ↳ release-lib | `test result: ok. 3802 passed; 0 failed; 12 ignored; 0 measured; 0 filtered out; finished in 41.58s` | 실제 종료 summary |
| ↳ release-lib | `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` | 실제 종료 summary |
| ↳ release-lib | `test result: ok. 165 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s` | 실제 종료 summary |
| ↳ release-lib | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | 실제 종료 summary |
| format-check | `cargo fmt --all -- --check` | exit 0 |
| native-clippy | `cargo clippy --locked --target-dir target/pr-review -- -D warnings` | exit 0 |
| wasm-clippy | `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings` | exit 0 |
| workspace-build | `cargo build --locked --workspace --target-dir target/pr-review` | exit 0 |
| workspace-clippy | `cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings` | exit 0 |
| manifest-check | `node scripts/rust-test-suite-manifest.mjs --check --base-ref a6e9d18fc1061318055b4eb348daec201427ea03` | exit 0 |
| unit-policy | `node scripts/rust-unit-test-tiers.mjs --check --base-ref a6e9d18fc1061318055b4eb348daec201427ea03` | exit 0 |
| diff-check | `git diff --check` | exit 0 |
| doc-tests | `cargo test --locked --doc --target-dir target/pr-review` | exit 0 |
| ↳ doc-tests | `test result: ok. 8 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.89s` | 실제 종료 summary |
| ↳ doc-tests | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | 실제 종료 summary |
| ↳ doc-tests | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | 실제 종료 summary |
| ↳ doc-tests | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | 실제 종료 summary |
| full-regression | `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | exit 0 |
| ↳ 전체 nextest | `Summary [1350.689s] 11353 tests run: 11353 passed (54 slow), 45 skipped` | 실제 최종 summary |
| skia-lib | `cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib` | exit 0 |
| ↳ skia-lib | `test result: ok. 3859 passed; 0 failed; 12 ignored; 0 measured; 0 filtered out; finished in 49.57s` | 실제 종료 summary |
| ↳ skia-lib | `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s` | 실제 종료 summary |
| ↳ skia-lib | `test result: ok. 165 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` | 실제 종료 summary |
| ↳ skia-lib | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | 실제 종료 summary |
| skia-placeholder | `node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir target/pr-review --features native-skia` | exit 0 |
| skia-pdf-export | `node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir target/pr-review --features native-skia` | exit 0 |

## 완료 페이지 회귀 5건의 정식 자동 수집과 lint

- 검증 checkout: `/home/tsjang/rhwp/output/pr-review/planet6897-20261018/completed-equation-formal-20261022-source`; 공용 빌드 target은 계속 `target/pr-review`입니다.
- 새로운 `tests/cases/equation_completed_document_boundaries.rs`를 자동 수집 경로에서 실행해 5 PASS·0 FAIL을 확인했습니다. 이 결과는 앞의 전체 11,353건 실행에 추가한 5건의 focused 결과이며 전체 11,358건을 한 번에 실행했다고 표현하지 않습니다.
- 별도 golden·snapshot이나 기대값 완화는 추가하지 않았습니다. 독립 Print 원점·소유·순서·내용 보존과 양쪽 raw 98% 이상인 완료 페이지를 보호합니다.
- [문서별 raw·Print 원점·수정 전후 실행·직접 확대](../../planet6897_completed_equation_pages_20261022/README.md)를 확인했습니다. 원본마다 후보 frozen Native 실행과 정식 자동 수집 실행을 구분합니다.

| 단계 | 실제 명령 | 결과 |
| --- | --- | --- |
| manifest-prepare | `node scripts/rust-test-suite-manifest.mjs --prepare` | exit 0 |
| format-check | `cargo fmt --all -- --check` | exit 0 |
| native-clippy | `cargo clippy --locked --target-dir target/pr-review -- -D warnings` | exit 0 |
| wasm-clippy | `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings` | exit 0 |
| workspace-build | `cargo build --locked --workspace --target-dir target/pr-review` | exit 0 |
| workspace-clippy | `cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings` | exit 0 |
| manifest-check | `node scripts/rust-test-suite-manifest.mjs --check --base-ref a6e9d18fc1061318055b4eb348daec201427ea03` | exit 0 |
| focused-new-five | `node scripts/run-rust-test.mjs equation_completed_document_boundaries -- --cargo-profile release-test --target-dir target/pr-review --test-threads 2` | exit 0 |
| diff-check | `git diff --check` | exit 0 |

## Studio·fresh WASM·실제 소비 경로

- 최적화 fresh WASM은 `scripts/wasm-pack-locked.sh --target web --out-dir pkg`로 실제 빌드했으며 codegen-units 16을 사용했습니다. Native/WASM 바이너리 해시·바이트와 실제 source는 [producer 원장](../producer_integrity.tsv)에 있습니다.
- Studio는 저장소 test script의 소스 패턴 전체를 Node test-concurrency 4로 실행해 2,356 PASS·실패 0이며 TypeScript와 production build도 exit 0입니다.
- 실제 Chromium 세 검사는 새 package의 JS/WASM 다운로드 바이트와 실제 실행을 확인했습니다. HTTP 200만으로 통과 판정하지 않았습니다.
- Native 소비 검사는 38 PASS와 12개 실측, WASM은 4개 원본 대조입니다. 새 base #8135의 영역 계획·외부 Surface·캐시 및 원래 출력 보존을 실제 호출 경로에서 확인했습니다.
- Native FFI 빌드와 Qt smoke는 25 PASS입니다. 실제 offscreen 렌더·합성 IME·clipboard·save/reopen 범위이며 실제 OS IME·하드웨어 GPU 검증으로 확대하지 않습니다.

## 원본·독립 기준과 렌더링 검사

- [전체 72개 입력과 기준 PDF](inputs_and_print_pdfs.md): HWP/HWPX 67개는 한컴 Print, PDF 입력 5개는 원본 PDF를 기준으로 비교했습니다.
- 기준/후보 × Native/fresh WASM 288개 TSV 원본, base 쪽수 불일치 진단 16개 추가 TSV, 주 자료 PNG 115개와 완료 수식 15개·원 PR별 대표 자료 62개의 바이트·SHA를 대조했습니다. Native/fresh WASM 각각 전체 1,529쪽이며 최저 raw 92.49647/91.66667%·현재 쪽수 불일치/90% 미만 0입니다.
- 0.1%p 이상 하락 0입니다. base가 쪽수 불일치인 8개 입력은 같은 쪽 번호를 같은 내용으로 가정하지 않고 별도 진단으로 보존했습니다.
- [변경 경계·수식·미세 간격 직접 판독](current_visual_reading.md), [80168 표제의 원 소유 검사](merged-80168.md), [HWPX 완료 서식](handbook-hwpx.md), [문단 테두리와 기존 px 검사 유지](paragraph-border-7967.md)를 함께 확인했습니다.
- [원 PR별 추가 대표 판독](../../planet6897_source27_representatives_20261022/README.md)은 12개 원 PR의 15쪽과 Native/WASM review·overlay를 연결합니다. #8026의 저장 HWP 출력과 원 PR의 기존 HWPX 재조판 비적용 범위를 구분하며 #7993은 닫지 않습니다. HWPX의 해당 TSV도 그대로 보존했습니다.
- 기존 #7114 저장·재열기 계약 5건과 정상 경계 대조 72건의 77건 통과를 전체 회귀와 구분합니다. #7114의 115쪽 기대값과 검사 모듈은 변경하지 않았습니다. 소유·순서·보존·포함·겹침 관계를 유지했고 독립 Print·양쪽 raw 98% 이상·확대 판독을 충족한 완료 페이지에만 ±2px 좌표를 사용했습니다. 375쪽은 양쪽 100%를 우선 선정했습니다.

## #7114의 기존 저장 계약 보존

- 원 해결 #7404와 통합 #7568의 기존 검사 5건을 유지했습니다. 이전 후보의 전체 nextest에서 실패한 3건은 `3cb5a4c290`의 공통 프레임 인증 보정으로 복구했습니다. 115쪽 기대값이나 검사 모듈을 바꾸지 않았습니다.
- 재조판한 표의 원본 되감김은 현재 프레임의 종료를 증명하지 못하므로 `table_text_reflowed` 상태로 원본 종료 빈 줄 인증을 기각합니다. 일반 빈 줄의 독립적인 줄 상자 소유와 미편집 원본 프레임은 유지합니다.
- 최적화 fresh WASM에서 기준/후보 각각 0·1·55·56·61·200자 편집을 실행했습니다. 12회 모두 저장 전후 쪽별 내용 차이가 0이고, 후보의 6개 HWP 저장 결과도 기준과 SHA-256이 같습니다. 55/56/61자는 115쪽과 독립 Print의 1·2쪽 경계를 유지합니다. 200자는 기존과 같은 116쪽이며 저장 전후 구성이 같습니다.
- 이는 기존 저장·재열기 기능의 보존입니다. 이미 이관된 거대 셀의 전쪽 시각 개선이나 남은 중첩 host 편집까지 해결했다고 주장하지 않으며 #7114/#7813은 닫지 않습니다.

## 제출과 후속 기록

이 문서는 실제 로컬 실행 결과입니다. 통합 PR 생성 뒤 정확한 code head CI 성공을 확인하고 원 PR 27개의 archive review·오늘할일을 같은 PR의 trailing 문서 commit으로 반영합니다. 이후 새 trailing head CI·merge와 후속 처리를 확인합니다.
