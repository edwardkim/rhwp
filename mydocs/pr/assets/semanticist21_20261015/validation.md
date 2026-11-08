# semanticist21 통합 — 로컬 검증 기록

검증 code: `20d93fe1d4226acab640b66875964d09c7d6d59b`. 기준: `upstream/devel`의 `2efefc6a1f9812a188e32c1cb283e9ef7962ea3e`. 검증 당시 렌더링 생산 코드 및 실제31개 입력/PDF 쌍을 이 code commit의 Git blob SHA와 전수 대조했다. 이 기록 뒤 production code가 바뀌면 기존 결과를 새 구현의 통과로 재사용하지 않는다.

Cargo는 jobs8·공용 `/home/tsjang/rhwp/target/pr-review` 한 곳에서 순차 실행했다. 테스트 실행은 전체12스레드, focused8스레드이며 Cargo 빌드 잡과 구분한다. 일반 검증에서 `CARGO_INCREMENTAL=0`을 지정하지 않았다.

## Rust·정책 gate

| 단계 | 종료 코드 | 실제 최종 요약 | 로컬 원시 로그 |
| --- | --- | --- | --- |
| cache-provenance-format-check | 0 | 완료, exit0 | `output/pr-review/semanticist21-20261015/logs/cache-provenance-format-check-20261016.log` |
| cache-provenance-suite-manifest-check | 0 | [RustTestSuite] 확인 완료: 1494 sources / 6354 static test attrs / 28 suites + 20 exceptions = 48/48 integration targets / nextest 최소 6288 cases / weight 1282754..1288243 | `output/pr-review/semanticist21-20261015/logs/cache-provenance-suite-manifest-check-20261016.log` |
| cache-provenance-unit-tier-check | 0 | [RustUnitTier] 확인 완료: 4103 tests / 296 modules / ready 0 / support 85 / white-box 4014 / cfg support items 28 | `output/pr-review/semanticist21-20261015/logs/cache-provenance-unit-tier-check-20261016.log` |
| cache-provenance-native-root-clippy | 0 | Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 34s | `output/pr-review/semanticist21-20261015/logs/cache-provenance-native-root-clippy-20261016.log` |
| cache-provenance-wasm32-lib-clippy | 0 | Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 27s | `output/pr-review/semanticist21-20261015/logs/cache-provenance-wasm32-lib-clippy-20261016.log` |
| cache-provenance-workspace-build | 0 | Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 28s | `output/pr-review/semanticist21-20261015/logs/cache-provenance-workspace-build-20261016.log` |
| cache-provenance-workspace-all-targets-clippy | 0 | Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 38s | `output/pr-review/semanticist21-20261015/logs/cache-provenance-workspace-all-targets-clippy-20261016.log` |
| cache-provenance-final-focused-rendering | 0 | Summary [   0.380s] 47 tests run: 47 passed, 868 skipped | `output/pr-review/semanticist21-20261015/logs/cache-provenance-final-focused-rendering-20261016.log` |
| cache-provenance-skia-lib | 0 | test result: ok. 3859 passed; 0 failed; 12 ignored; 0 measured; 0 filtered out; finished in 35.11s / test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s / test result: ok. 165 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s / test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s | `output/pr-review/semanticist21-20261015/logs/cache-provenance-skia-lib-20261016.log` |
| cache-provenance-skia-picture-placeholder | 0 | Summary [   2.298s] 2 tests run: 2 passed, 227 skipped | `output/pr-review/semanticist21-20261015/logs/cache-provenance-skia-picture-placeholder-20261016.log` |
| cache-provenance-skia-direct-pdf | 0 | Summary [   0.886s] 4 tests run: 4 passed, 221 skipped | `output/pr-review/semanticist21-20261015/logs/cache-provenance-skia-direct-pdf-20261016.log` |
| cache-provenance-doctests | 0 | test result: ok. 8 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.90s / test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s / test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s / test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s | `output/pr-review/semanticist21-20261015/logs/cache-provenance-doctests-20261016.log` |
| cache-provenance-release-build | 0 | Finished `release` profile [optimized] target(s) in 23m 16s | `output/pr-review/semanticist21-20261015/logs/cache-provenance-release-build-20261016.log` |
| cache-provenance-release-lib | 0 | test result: ok. 3802 passed; 0 failed; 12 ignored; 0 measured; 0 filtered out; finished in 33.21s / test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s / test result: ok. 165 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s / test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s | `output/pr-review/semanticist21-20261015/logs/cache-provenance-release-lib-20261016.log` |

실제 명령은 다음과 같다. 앞 명령이 끝난 뒤 다음 Cargo 명령을 실행했고 실패 뒤 단계로 건너뛰지 않았다.

```bash
cargo fmt --all -- --check
node scripts/rust-test-suite-manifest.mjs --check --base-ref 2efefc6a1f9812a188e32c1cb283e9ef7962ea3e
node scripts/rust-unit-test-tiers.mjs --check --base-ref 2efefc6a1f9812a188e32c1cb283e9ef7962ea3e
cargo clippy --locked --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir /home/tsjang/rhwp/target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings
cargo nextest run --locked --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia --test regression_suite_006 --test regression_suite_016 --test regression_suite_019 --test regression_suite_020 -E 'test(/(^|::)(placeholder_tab_keeps_width_after_reopen|auto_number_after_control|body_page_auto_number_display|new_page_column_definition|semanticist21_qualified_print_positions)::/)' --test-threads 8 --no-fail-fast
cargo test --locked --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia --lib
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia
cargo test --locked --doc --target-dir /home/tsjang/rhwp/target/pr-review
cargo build --locked --release --target-dir /home/tsjang/rhwp/target/pr-review
cargo test --locked --release --target-dir /home/tsjang/rhwp/target/pr-review --lib
```

## 전체 회귀 실행과 수정 후 합산 판정

전체 nextest는 부모 code `ed97e78cb1dbe73e6d0102a809074718e26a2941`에서10,404건을 실행해10,402PASS /2FAIL /45SKIP, exit100으로 끝났다. 실패는 `convert_verify_corpus_ratchet::ratchet_partition_13`와 `ratchet_partition_20`이며 원본 HWP3 저장 LineSeg5개/1개가 사라졌다. 기본 upstream/devel에서는 두 입력의 convert --verify 차이0이었다. 원시 로그: `logs/final-all-rust-nextest-20261016.log`.

현재20d93fe1d는 캐시 출처를 구분해 원본 저장 LineSeg를 유지하도록 보정했다. 실패한 두 partition과 관련 탭/캐시4개를 재실행해6/6PASS(exit0), 이후 최종 렌더링47/47PASS(exit0)를 확인했다. 실패를 이관하거나 assertion·baseline·허용치를 완화하지 않았다. 로그: `logs/cache-provenance-focused-six-20261016.log`, `logs/cache-provenance-final-focused-rendering-20261016.log`.

전체 원 실행 명령: `cargo nextest run --locked --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast` (위 부모 code에서 실행).

사용자 지시 “수정된 이슈만 개별적으로 돌려서 통과하면 전체 회귀테스트 다 통과된것으로 간주”에 따라 전체 실행과 수정 후 관련 재실행을 합산해 전체 회귀 통과로 처리한다. 현재 code에서10,404건 전체를 다시 실행한 exit0이라고 보고하지 않는다. 통합 최신 head의 원격 CI는 별도 gate다.

## 좁힌 회귀와 수정 전후

- 독립 Print 기반 px 첫15건:14PASS/1FAIL. 새 쪽 단 구분선 끝에 후행 간격8px가 붙는 실제 결함을 검출했다. 물리 끝/흐름 간격 소비를 분리한 뒤15/15PASS.
- 본문 번호 paint: 원문 모델은 보존되지만 실제 번호가 빠지는 두 정식 검사 FAIL을 확인하고 공통 표시값 소비를 추가했다.
- 번호 소속: 정식 owner 반례 수정 전1PASS/1FAIL(그림 뒤 scalar2, 원본 기대1), 원문 자리표 중복 소비 제거 뒤 PASS.
- 동적 필드/간격 최종 focused24/24PASS. 이후 신규 문서별 px를 확장한 최종29/29PASS. 최초 추가 px27/29의2FAIL은 별도 각주 흐름까지 본문 수량으로 센 helper 오류였고, 소유 흐름을 구분한 뒤29/29PASS. 실제 각주 출력과 독립 기대 좌표는 유지했다.
- ZIP9 entry.name Result/Cow API에 맞춰 두 원 PR 테스트4곳을 보정했고 원 assertion은 유지했다.
- ClipboardEvent observer 반례: 원래 코드가 교체된 새 문서를 삭제해 실제 실패; 삭제 후 알림으로 바꾼 뒤 관련143focusedPASS.
- 탭/채우기 대화상자: 수동 VM의 실제 tab helper 의존성15FAIL을 연결한 뒤 focused33PASS. 생산 단위 helper를 복제하지 않았다.

## Studio와 실제 브라우저

변경된 TypeScript는 e2e28e79e·a2200bf69 이후 동일하다. 해당 TS tree에서 타입 검사·전체 단위2010/2010·production build를 완료했다. 이후 새 Rust/WASM은 실제 fresh package의 초기 응답 SHA를 관찰해 별도로 브라우저에 반영했다.

| 실행 | 결과 | raw 로그 |
| --- | --- | --- |
| Studio TypeScript | PASS | `studio-typecheck-final-20261015.log` |
| 전체 Studio unit | 2010/2010PASS | `studio-full-unit-after-compat-20261015.log` |
| Studio production build | PASS | `studio-production-build-20261016.log` |
| 실제 topmost Cut | PASS | `studio-topmost-cut-e2e-20261016.log` |
| 외부 clipboard 우선순위 | PASS | `studio-clipboard-priority-e2e-20261016.log` |
| 최신 WASM 실제150pt/default50pt 대화상자·HWP/HWPX 저장 재열기 | PASS, 생성 줄 캐시 제외 확인 | `studio-tab-units-cache-provenance-20261016.log` |
| 최신 fresh WASM의 실제 PageRenderer C2D/CanvasKit/SVG | 5입력9쪽씩 실제 실행·직접 대조 | `cache-provenance-studio-backend-probe-20261016.log` |

브라우저 관찰기의 최초2회 실패는 Chrome inspector의 streaming WASM response cache eviction이었다. 제품 실패나 성공으로 집계하지 않았다. 초기 fetch response를 clone해 실제 초기화 바이트의 SHA를 관찰하는 방식으로 바꿔 완료했다. [실제 backend PNG와 진단](studio-backends/README.md)에 연결한다.

## fresh WASM·시각·입력

- fresh WASM build exit0, codegen-units16/jobs8. Docker 없는 호스트의 문서화된 `--no-opt` fallback이며 Docker 최적화 build라고 보고하지 않는다. root pkg와 Studio public JS/WASM SHA 일치를 확인했다.
- Native31문서44쪽 및 fresh WASM31문서44쪽 전쪽90% 이상, 쪽수/누락/측정 불가0, base 대비0.01%p 이상 하락0. PR 관련29문서42쪽은 양쪽100%와 직접 확대 판독 완료.
- sourcefingerprint를 고정한 새 CLI·WASM은 현재20d93fe1d 코드와 일치했다. 현재 fresh 파이프라인은 build·Native31문서·WASM31문서 전 단계 exit0으로 완료했다. 앞선143 중단 실행은 현재 완료 실행과 구분한다. 새 PNG88개는 이전 직접 판독 캡처와 픽셀 단위로 동일했다. 새 대표 시트와 실제 backend5시트도 다시 직접 대조했다.
- HWP3 표의 기존 날짜 누락과 다른 HWP3 원형 기호 차이는 신규 px 자격 또는 완전 개선으로 승격하지 않았다. 현재 PR 관련29문서의 개선 범위와 추가 기존 대조군2문서를 구분한다.
- `rhwp threat-scan <실제 fixture> --json`으로 신규29개 입력을 직접 전수 스캔했다: clean29/29, findingCount0, exit0. `RHWP_SECURITY_SWEEP_SAMPLES_JSON`에 tests/fixtures를 전달해 샘플 전용 수집기를 우회하거나0건 실행을 전수 검사로 세지 않았다.

[입력/PDF SHA·전체 페이지 값·직접 PNG·재현 명령](README.md)이 원장이다. TSV/로그/중간 JSON은 ignored output에 두며 PR 본문에 baseline/candidate×Native/fresh WASM124개 TSV 원문을 기록한다. 모든 기준 PDF는50MiB 미만이고 code commit에 포함했다. 신규/변조 폰트 바이너리를 배포하지 않는다.

## 원격 경계

이 문서는 로컬 검증 결과다. 통합 PR의 정확한 최신 head CI·mergeability를 확인한 뒤에만 merge한다. 원 PR7건의 원 author 및 체리픽 출처를 보존하고, merge SHA에서 추적 asset을 확인해 contributor comment·supersede close·devel sync·post_merge 정리를 이어간다. 별도 기록 PR은 만들지 않는다.

## 통합 code candidate CI 확인

통합 [PR #7935](https://github.com/edwardkim/rhwp/pull/7935)의 code candidate head `2ead4954547804215c9c4f4e2c145700467ddbb5`에서 [CI](https://github.com/edwardkim/rhwp/actions/runs/38799924267) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38799924232) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38799924058) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/38799924281) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/38799924242) 및 정책 status 성공을 확인했다. 실패·대기 없음. 검증한 제품 code는 `20d93fe1d4226acab640b66875964d09c7d6d59b`이며 그 뒤 제품/테스트/Studio 변경은 없다. 이 CI 기록만 추가하는 문서 후행 head의 required checks와 mergeability는 별도로 확인한 뒤 merge한다.
