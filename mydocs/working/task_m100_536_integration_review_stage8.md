# seo-rii 통합 검토 8단계 — #7964 반영과 실제 소비 경로 재검증

## 기준과 완료된 선행 검증

Issue: #536. 원 contributor 비 draft PR20개 범위와 source head는 그대로다. draft Qt·packaging은 제외한다.
[7단계](task_m100_536_integration_review_stage7.md)의 source751/baseb7 전체 회귀10,770 PASS,
실제 FFI·Native/fresh WASM 증거는 당시 결과로 유지한다. 새 base의 완료 증거로 바꾸지 않는다.
TSV 본문 삽입·본문 크기 제한 시 SHA 고정 원본 링크 제출 지침은 `92e4f518a`에 기록했다.
사용자가 취소한 `post_merge.md` 수정은 포함하지 않는다.

최신 `upstream/devel`은 #7964 merge `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`이다.
한컴 그림 색조·WMF crop·물리 인쇄 종이·저장 줄과 셀 도형 원점·Direct PDF가 바뀌었다.
사전 merge-tree는 `src/renderer/skia/image_conv.rs`의 콘텐츠 충돌1개를 검출했다.
완료된 이전 gate 결과를 commit한 뒤 실행 중인 Cargo/Rust가 없음을 확인했다.
사용자 dirty `rhwp-studio/public/rhwp.js`가 있는 기본 작업공간은 변경하지 않는다.

## 통합 근거와 계획

1. 같은 검토 branch를 새 devel 위로 rebase한다. 기존 source author와 원 SHA를 보존한다.
   충돌은 원 commit의 일반 동작과 새 upstream 동작을 실제 호출 경로로 대조해 해소한다.
2. 이미지 경로는 `image_resolver::adjusted_picture_bytes`의 한컴 색조 바이트 생산 →
   Skia `draw_image_bytes`의 정규화·decoded-image 캐시 → 색 필터·sampling → 실제 draw로 추적한다.
   이미 bake한 색조에 효과·밝기·대비를 다시 적용하지 않고, 캐시 key의 입력 바이트와 변환 조건,
   source-pixel 이산 효과·nearest sampling·실패 fallback을 함께 확인한다. gradient와 Direct PDF의
   자동 병합도 검토하여 색조 지원과 기존 gradient admission을 둘 다 보존한다.
3. 공용 `target/pr-review`, Cargo 잡8개로 새 base와 후보 Native Skia CLI·fresh host `--no-opt`
   WASM을 순차 빌드한다. Docker 최적화 배포 검증으로 표시하지 않는다. 이전 산출물은 해시 고정한
   역사적 결과로 보존하며 새 producer의 실행을 대신하지 않는다.
4. 같은 원본8개 전쪽 Native/fresh WASM을 비교한다. 최신 정책의 `silhouette_raw_match_percent`로
   양쪽90% 이상·쪽수1:1·누락0·devel 대비0.01%p 이상 하락 없음을 확인하고 대표 PNG를 직접 판독한다.
   분자·괄호/폼 bevel의 기존 차이와 미달 단독 극한 진단은 전체 해결로 표시하지 않는다.
5. 기존 Native 집중·HFT·새 그림 색조/WMF/용지 경계, 공식 Skia·Direct PDF, 실제 ABI feature-on/off,
   fresh Studio/browser, 변경 범위의 Rust lint·workspace build·전체 회귀를 새 source에서 확인한다.
   기능 코드가 실제로 바뀐 통합 head의 검증이며 지침 Markdown 수정 때문에 회귀를 반복하는 것이 아니다.
6. 완료한 실제 결과만 개별 review20개와 구현 기록에 연결한다. 최종 TSV32개·대표 PNG20개를 새
   source로 갱신하고, producer SHA와 asset head를 구분한다. 원격 base/source head 재확인 후 자신의
   통합 branch만 정확한 이전 remote SHA의 lease로 push한다. PR의 실제 CI → 같은 PR의
   review/오늘할일 trailing → trailing CI → 정상 merge·post_merge 순서를 유지한다.

위 내용은 단계 시작 당시 계획이다. 아래 최종 실제 실행 결과가 현재 로컬 판정이며 원격 PR/CI는 이후 실제 상태를 확인한다.
실제 명령·로그·해시는 ignored `output/pr-review/seo-rii-20261017/latest-*`에 기록한다.

## 실제 리베이스 결과

리베이스를 완료했다. 직후 head는 `ec742f167991fd8c5efcd01ab7e8fe752c0a070e`이며164개
commit의 수·순서·제목과 원 contributor author를 보존했다. 독립 merge-tree와 비교한 수동 해소
경로는 `src/renderer/skia/image_conv.rs` 하나다. cache 도입·이산 효과·cache-hit sampling·진단
commit의 같은 파일 충돌을 순서대로 해소했다. 나머지 자동 병합 결과는 독립 tree와 같다.

한컴 색조 보정은 SVG/JPEG 정규화 뒤, decode 및 이산 효과 전의 cache miss에서 수행한다.
보정 PNG는 같은 cache/배치 경로로 중립 효과·밝기0·대비0으로 재생한다. 원본 decode key에는
색조 조건도 포함해, 먼저 중립 그림을 캐시한 뒤 같은 원본을 다른 밝기/대비로 그릴 때 보정을
우회하지 않게 했다. 기존 이산 효과의 source pixel 처리·nearest sampling·실패 fallback과
bounded cache 관측을 유지했다. 지원하지 않는 색조의 필터 합성 순서·linearRGB fallback도
새 upstream과 대조해 보존했다. 실제 실행 결과는 다음 검증이 완료된 뒤 추가한다.

`latest-rebase-audit.json`에 이전/현재164개 SHA mapping과 tree 대조를 보존한다.
이전 전체 회귀는 source751/baseb7 결과이며 최신 리베이스 결과의 회귀 통과로 표시하지 않는다.

## 실제 Native lint 오류와 보정

producer `8af3e72bde9e62cfb4daa1e9f7663bb3a244c282`에서 Native 집중288PASS, Skia lib4043PASS/12ignored와 fresh Studio10종을 확인했다. 이후 feature-on FFI Clippy는 `renderer/skia/renderer.rs`의 `needless_option_as_deref` 2곳으로 exit101이었다. 리베이스 자동 병합 뒤 `resolved.as_deref().or(fallback.as_ref())`가 이미 `Option<&ResolvedImagePayload>`를 생산하므로 다음 `as_deref()`는 같은 타입을 다시 역참조한다. 두 호출만 제거하고 동일 `map`/`is_some_and` 소비와 fallback·효과 억제 동작을 유지한다. lint 경고를 허용하지 않는다.

이 변경은 같은 payload 참조를 소비하는 타입상 항등 정리지만 실제 source가 바뀌므로 이전288PASS·4043PASS를 새 commit 실행으로 표시하지 않는다. 완료한 base9415 산출물은 재사용하고, 후보 Native/fresh WASM·시각 증거 및 영향 gate를 새 producer에 연결한 뒤 남은 전체 회귀·lint를 진행한다. 전체 회귀는 아직 이번 base에서 실행 전이었다. 이전 source751/baseb7의10770PASS는 역사적 결과로 유지한다.

## 최종 실제 실행 결과

실행 source `3535106051394672e93ca8401ff9eae8b449ec20`, 기준 `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`. 전체 회귀10779PASS, 실패0, 45skipped. Native 집중288PASS.
Native Skia lib는 4,043PASS /0FAIL /12ignored다.
Rust 필수 lint·workspace build·doc·실제 ABI feature-on/off와 fresh Studio10종은 모두 exit0이다.13개 전쪽 Native/fresh WASM은 raw90% 이상·쪽수1:1·누락0·devel 대비0.01%p 이상 하락 없음이다.24개 대표PNG와52개 최종TSV를 새 producer로 갱신했다. direct reading의 남은 차이·한계는 아래 기록과 판독 원장을 따른다. 원격 exact-head CI는 아직 미실행이며 실제 CI 성공 후 같은 PR의 trailing 기록에 연결한다.

| gate | 실제 명령 | 전체 소요 초 |
| --- | --- | ---: |
| latest-manifest-prepare | `node scripts/rust-test-suite-manifest.mjs --prepare` | 3.1 |
| latest-manifest-policy | `node scripts/rust-test-suite-manifest.mjs --check --base-ref 9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 1.4 |
| latest-unit-tier-policy | `node scripts/rust-unit-test-tiers.mjs --check --base-ref 9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 37.9 |
| latest-focused-native | `cargo nextest run --locked --cargo-profile release-test --features native-skia --target-dir /home/tsjang/rhwp/target/pr-review --test issue_4969_shaping_glyph_lowering --test regression_suite_001 --test regression_suite_002 --test regression_suite_003 --test regression_suite_004 --test regression_suite_007 --test regression_suite_008 --test regression_suite_009 --test regression_suite_011 --test regression_suite_012 --test regression_suite_013 --test regression_suite_015 --test regression_suite_016 --test regression_suite_018 --test regression_suite_019 --test regression_suite_022 --test regression_suite_023 --test regression_suite_024 --test regression_suite_025 --test regression_suite_026 --test regression_suite_027 --test regression_suite_028 -E 'test(/(^|::)(issue_4969_shaping_atomic_activation|issue_536_direct_pdf_linear_gradient|issue_536_glyph_effect_contract|issue_536_glyph_run_wire_size|issue_536_glyph_visual_bounds|issue_536_native_accessibility|issue_536_native_clipboard|issue_536_native_commands|issue_536_native_editor|issue_536_native_font_selection|issue_536_native_fonts|issue_536_native_interaction|issue_536_native_runtime|issue_536_native_save|issue_536_native_session|issue_536_native_view|issue_536_native_viewport|issue_536_portable_glyph_geometry|issue_536_raster_diagnostics_contract|issue_536_shaped_measurement_report|issue_536_strict_export_rejection|issue_536_svg_image_comment_admission|issue_536_text_v2_export|issue_536_text_v2_slots|issue_6520_svg_layer_contract|skia_native_glyph_replay|issue_536_equation_limit_name|hft_outline_font_contract|issue_7751_picture_tone_source|issue_7751_print_paper_frames|issue_7751_raster_picture_tones|issue_7751_wmf_picture_payload)::/) or binary(/^issue_4969_shaping_glyph_lowering$/)' --test-threads 12 --no-fail-fast` | 492.1 |
| latest-skia-lib | `cargo test --locked --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia --lib -- --test-threads 12` | 398.9 |
| latest-skia-issue_2225_missing_picture_placeholder | `node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 5.0 |
| latest-skia-render_p37_direct_pdf_export | `node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 4.2 |
| latest-skia-issue_7950_pdf_paint | `node scripts/run-rust-test.mjs issue_7950_pdf_paint -- --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 4.2 |
| latest-ffi-enabled-clippy | `cargo clippy --locked -p rhwp-native-ffi --features native-skia --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings` | 1.4 |
| latest-ffi-enabled-build | `cargo build --locked -p rhwp-native-ffi --profile release-test --features native-skia --target-dir /home/tsjang/rhwp/target/pr-review` | 283.8 |
| latest-pdf-direct | `python3 bindings/Native/tests/pdf_direct_contract.py --library /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-librhwp_native_ffi.so --native-skia enabled --sample samples/lseg-01-basic.hwp` | 1.4 |
| latest-session | `python3 bindings/Native/tests/session_contract.py /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-librhwp_native_ffi.so --native-skia enabled` | 0.2 |
| latest-native_host_smoke | `python3 bindings/Native/tests/native_host_smoke.py --library /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-librhwp_native_ffi.so` | 1.1 |
| latest-native_commands_contract | `python3 bindings/Native/tests/native_commands_contract.py --library /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-librhwp_native_ffi.so` | 0.3 |
| latest-native_file_store_smoke | `python3 bindings/Native/tests/native_file_store_smoke.py --library /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-librhwp_native_ffi.so` | 0.3 |
| latest-native_accessibility_smoke | `python3 bindings/Native/tests/native_accessibility_smoke.py --library /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-librhwp_native_ffi.so` | 0.2 |
| latest-native_view_contract | `python3 bindings/Native/tests/native_view_contract.py /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-librhwp_native_ffi.so` | 0.2 |
| latest-native_worker_contract | `python3 bindings/Native/tests/native_worker_contract.py /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-librhwp_native_ffi.so` | 0.8 |
| latest-native_clipboard_contract | `python3 bindings/Native/tests/native_clipboard_contract.py /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-librhwp_native_ffi.so` | 0.2 |
| latest-full-nextest | `cargo nextest run --locked --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --tests -p rhwp -p rhwp-contracts -p rhwp-native-ffi -p rhwp-ooxml-chart -p rhwp-password-crypto --test-threads 12 --no-fail-fast` | 1240.5 |
| latest-fmt | `cargo fmt --all -- --check` | 40.1 |
| latest-native-clippy | `cargo clippy --locked --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings` | 142.6 |
| latest-wasm-clippy | `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings` | 130.5 |
| latest-workspace-build | `cargo build --locked --workspace --target-dir /home/tsjang/rhwp/target/pr-review` | 203.5 |
| latest-workspace-clippy | `cargo clippy --locked --workspace --all-targets --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings` | 236.8 |
| latest-doc-tests | `cargo test --locked --doc --target-dir /home/tsjang/rhwp/target/pr-review` | 274.7 |
| latest-ffi-disabled-build | `cargo build --locked -p rhwp-native-ffi --target-dir /home/tsjang/rhwp/target/pr-review` | 246.1 |
| latest-ffi-disabled-pdf | `python3 bindings/Native/tests/pdf_direct_contract.py --library /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-disabled-librhwp_native_ffi.so --native-skia disabled --sample samples/lseg-01-basic.hwp` | 0.2 |
| latest-ffi-disabled-session | `python3 bindings/Native/tests/session_contract.py /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-disabled-librhwp_native_ffi.so --native-skia disabled` | 0.3 |
| latest-diff-check | `git diff --check` | 0.6 |
| latest-typescript | `npx tsc --noEmit` | 3.4 |
| latest-frontend-unit | `npm test` | 30.4 |
| latest-frontend-build | `npm run build` | 11.8 |
| latest-renderer-contract | `npm run e2e:renderer-contract` | 3.7 |
| latest-canvaskit-font-coverage | `node e2e/canvaskit-font-coverage.test.mjs` | 1.7 |
| latest-shaping-replay | `node e2e/issue-4969-shaping-replay.test.mjs` | 1.0 |
| latest-browser-portable-geometry | `node e2e/run-with-vite.mjs -- node e2e/glyph-outline-portable-geometry.test.mjs` | 4.9 |
| latest-browser-font-preparation | `node e2e/run-with-vite.mjs -- node e2e/canvaskit-font-preparation-issue7688.test.mjs --mode=headless` | 44.2 |
| latest-browser-picture-tones | `node --test ../scripts/tests/picture-tones.browser.mjs` | 12.5 |
| latest-browser-wmf-crop | `node --test ../scripts/tests/wmf-picture-crop.browser.mjs` | 8.3 |

### 전쪽 raw 실루엣 대조

| 입력 | devel Native | 후보 Native | devel WASM | 후보 WASM |
| --- | ---: | ---: | ---: | ---: |
| lseg-01-basic | 99.74540 | 99.74540 | 99.93175 | 99.93175 |
| lseg-02-mixed | 99.80580 | 99.80580 | 99.84053 | 99.84053 |
| pic-crop-01 | 99.98500 | 99.98500 | 99.74013 | 99.74013 |
| shape-group-02 | 90.03440 | 100.00000 | 100.00000 | 100.00000 |
| equation-lim-print2020 | 88.42676 | 90.53685 | 89.77376 | 91.90909 |
| form-hwpx | 99.88426 | 99.88426 | 99.87500 | 99.87500 |
| form-hwp-manual-crossformat | 99.88426 | 99.88426 | 99.87500 | 99.87500 |
| native-ime | 100.00000 | 100.00000 | 100.00000 | 100.00000 |
| hancom-color-probe-2024 | 100.00000 | 100.00000 | 100.00000 | 100.00000 |
| hancom-color-probe-cross-2024 | 100.00000 | 100.00000 | 100.00000 | 100.00000 |
| hancom-color-probe-boundary-2024 | 100.00000 | 100.00000 | 100.00000 | 100.00000 |
| hancom-blackwhite-zero-2024 | 100.00000 | 100.00000 | 100.00000 | 100.00000 |
| hancom-blackwhite-axis-probe-2024 | 100.00000 | 100.00000 | 100.00000 | 100.00000 |

### 직접 확대 판독과 남은 차이

- `lseg-01-basic`: 문단·줄 순서와 줄바꿈 유지. 한컴 대비 기존 glyph raster 가장자리 차이 남음.
- `lseg-02-mixed`: 혼합 문단·줄 소속과 간격 대응. 한컴 대비 기존 raster 차이 남음.
- `pic-crop-01`: 두 그림의 위치·crop 보존. 기존 얇은 테두리·raster 차이는 전체 해결로 주장하지 않음.
- `shape-group-02`: 경로 외곽·gradient 방향·그림자 대응. raw100은 RGB100과 구분.
- `equation-lim-print2020`: ASCII lim 이름의 face/em 크기 개선. 분자·괄호 간격은 Print와 차이가 남음. 동일 좌표 4배 확대 판독과 base→후보 분수 영역 변경 픽셀0을 확인. 이름 보정의 제한된 범위만 해결로 기록.
- `form-hwpx`: 5개 form control의 상태·문자열·배치 보존. Edit 상단/좌측 bevel 차이 남음.
- `form-hwp-manual-crossformat`: 기존 HWPX 수동 Print와 대응하는 HWP 출력. 새 HWP 자체 Print로 주장하지 않음. Edit bevel 차이 남음.
- `native-ime`: CLI/실제 C ABI의 한글 각과 내용·위치를 확인했다. 독립 Print보다 ink bbox 상단2px·하단1px 및 AA 차이는 기준 Native부터 동일하다. 실제 API794×1123은 CLI/Print794×1122보다 흰 행1개가 더 있으며 공통794×1122영역은 기준/후보 CLI와 픽셀이 모두 같다. raw100을 엄격 RGB100으로 표시하지 않는다.
- `hancom-color-probe-2024`: 한컴 저장 색조 합성 대조군의 행·patch 위치·흑백/색상 패턴 대응. raw100을 RGB100으로 일반화하지 않으며 기존 RGB 계약 검증을 별도로 실행.
- `hancom-color-probe-cross-2024`: 한컴 저장 색조 합성 대조군의 행·patch 위치·흑백/색상 패턴 대응. raw100을 RGB100으로 일반화하지 않으며 기존 RGB 계약 검증을 별도로 실행.
- `hancom-color-probe-boundary-2024`: 한컴 저장 색조 합성 대조군의 행·patch 위치·흑백/색상 패턴 대응. raw100을 RGB100으로 일반화하지 않으며 기존 RGB 계약 검증을 별도로 실행.
- `hancom-blackwhite-zero-2024`: 한컴 저장 색조 합성 대조군의 행·patch 위치·흑백/색상 패턴 대응. raw100을 RGB100으로 일반화하지 않으며 기존 RGB 계약 검증을 별도로 실행.
- `hancom-blackwhite-axis-probe-2024`: 한컴 저장 색조 합성 대조군의 행·patch 위치·흑백/색상 패턴 대응. raw100을 RGB100으로 일반화하지 않으며 기존 RGB 계약 검증을 별도로 실행.

대표 review/overlay는 Native·fresh WASM 각5개 입력과 실제 C ABI screen·Direct PDF의12쌍이다. 전쪽 fresh 캡처26개는 직전 직접 판독본과 원 페이지 PNG·review·overlay 모두 바이트 동일임을 확인해 기존 판독을 재사용했다. 실제 API의 새 review·overlay2쌍은 직접 열었다. 수식은 동일 좌표4배 확대했고 기존 분수 영역의 base→후보 변경 픽셀은 양쪽0이었다. 원점·분모·첨자와 이름 크기를 구분했으며 기존 분자/괄호 간격은 미해결로 남겼다. raw100은 엄격 RGB 일치율100을 뜻하지 않는다. 원격 CI는 이후 실제 통합 PR에서 확인한다.

Docker가 없어 fresh WASM은 host `--no-opt` 경로다. 실제 Apple SDK XCFramework·C# runtime은 실행하지 않았다. 한컴 저장 색조5개는 원래 원본8개와 구분한 합성 RGB 계약이며383쪽 편람 전체 검증의 대체가 아니다. HWP 폼은 기존 HWPX 수동 Print와의 cross-format 대응이며 새 HWP 직접 Print라고 표시하지 않는다.

### 최종 리베이스 증거

164개 commit의 수·순서·제목과 author/email 보존 및 독립 merge-tree 대조를 확인했다. 실제 source의 `-x` trailer에서 원 commit→현재 SHA를 다시 추출해 각 구현 기록에 추가했다. 수동 해소는 `src/renderer/skia/image_conv.rs` 한 경로이며 나머지 자동 tree와 동일했다. source producer와 최종 문서·asset commit SHA는 구분한다.

### 검증 입력의 commit 포함 확인

모든13개 입력과 대응 Print는 실제 producer commit에 포함된 바이트와 대조했다. HWP 폼의 cross-format 참조·API 생성 IME·독립 한컴 저장 색조의 출처는 앞의 제한을 따른다.

| 입력 key | 파일 역할 | 저장소 경로 | SHA-256 |
| --- | --- | --- | --- |
| lseg-01-basic | hwp | `samples/lseg-01-basic.hwp` | `86e607233a2218928fabe5969155116a46d4ab4400db1402fe45b0082149496c` |
| lseg-01-basic | pdf | `pdf/lseg-01-basic-hwp-2020.pdf` | `7c9ebbf14830dc1d519cb42c4be7e55c7e210d5457e616f4d94ead6bdb6ee093` |
| lseg-02-mixed | hwp | `samples/lseg-02-mixed.hwp` | `2d22a57b6d1c26b5593bfeec2dc25f3339abd3ab788cf79825330951e7f46bb0` |
| lseg-02-mixed | pdf | `pdf/lseg-02-mixed-hwp-2020.pdf` | `b85af84c1cc0cdd6d6abb6fff542f881dca272bc813094a48347e2a50b8845bb` |
| pic-crop-01 | hwp | `samples/pic-crop-01.hwp` | `3b5afbe4cdb18faa49452a9cbed770d5caa27aed2f83b708e9ab8516c65ebf4c` |
| pic-crop-01 | pdf | `pdf/pic-crop-01-2022.pdf` | `4ae5f6f8bc308bedf16fbca139de448c42d29b42bbba22221f4a1acc9694c4d3` |
| shape-group-02 | hwp | `samples/shape-group-02.hwp` | `12a4b4962a57b4d292d23a1809255ddb7e8400af4dc7884c5bcb7b4767a6628f` |
| shape-group-02 | pdf | `pdf/shape-group-02-hwp-2020.pdf` | `ab759c9c2031f4a6fe64c75381a2c5dbf215777a98b1b0b7cd0243f801662dc7` |
| equation-lim-print2020 | hwp | `samples/equation-lim.hwp` | `ba168d388df614e7b9390c4685d368cf3019595c5fecbce0e681dc8fb9bea36d` |
| equation-lim-print2020 | pdf | `pdf/seo-rii-20261017/equation-lim-2020.pdf` | `62b6b47da9c9dde5fb44ed4e025bbb871a048f01e9c7bf51f5e93be4a1a51aae` |
| form-hwpx | hwp | `samples/hwpx/form-01.hwpx` | `3bbd207b88fe61e802706de3ccf98abdb8b450493164eec657c9ee88a5aba87e` |
| form-hwpx | pdf | `samples/hwpx/form-01-2024.pdf` | `d1117657d92c789295d73af1bebd11b241eb352254b328d95c07f0efb87b18ee` |
| form-hwp-manual-crossformat | hwp | `samples/form-01.hwp` | `75b6b8bb9121c6aeebf567aaa158ac7ec95caae597fd8a5ec7d4c84a25b927ac` |
| form-hwp-manual-crossformat | pdf | `samples/hwpx/form-01-2024.pdf` | `d1117657d92c789295d73af1bebd11b241eb352254b328d95c07f0efb87b18ee` |
| native-ime | hwp | `mydocs/pr/assets/issue_536_seo_rii_20261017/native-ime.hwpx` | `dea591e15e91d9c23557a1a7630da0f80391ff9ea051fe3600b14ec5cb42adf3` |
| native-ime | pdf | `pdf/seo-rii-20261017/native-ime-2020.pdf` | `b58896a37a4cfb6ba73ea0e741b5aad20abe6a932f034f10db2cbcd8ce1e9c37` |
| hancom-color-probe-2024 | hwp | `tests/fixtures/issue_7751_color_controls/hancom-color-probe-2024.hwp` | `e85d3a4b22ac9ce0515a2bf5702aafbc2259a3562e086fd523b7a05b005dd015` |
| hancom-color-probe-2024 | pdf | `pdf/issue7751/hancom-color-probe-2024.pdf` | `8b4420d8caa04c822f47e67fc9edff4b3e447d64c15cac487b4bbd26e1ea6269` |
| hancom-color-probe-cross-2024 | hwp | `tests/fixtures/issue_7751_color_controls/hancom-color-probe-cross-2024.hwp` | `89a16e13e967b8ad9145cf99367f79af8c5014d4e3e09e9689f19c6ffba5655c` |
| hancom-color-probe-cross-2024 | pdf | `pdf/issue7751/hancom-color-probe-cross-2024.pdf` | `c74563caae874002ceb0976db9af136f00bcbce8ba38a295113ce9d6d5f2b967` |
| hancom-color-probe-boundary-2024 | hwp | `tests/fixtures/issue_7751_color_controls/hancom-color-probe-boundary-2024.hwp` | `e40cfacea9220e44e1281e3a3841648f92d6b6098d20040cfce4497c3f369375` |
| hancom-color-probe-boundary-2024 | pdf | `pdf/issue7751/hancom-color-probe-boundary-2024.pdf` | `2ead7978b7f7210d8050d737f7ae318d9a74897b0395a4d162c5413e21c4af3e` |
| hancom-blackwhite-zero-2024 | hwp | `tests/fixtures/issue_7751_color_controls/hancom-blackwhite-zero-2024.hwp` | `58c10239a1185abe20b3655c661b13f73977ede222bef918ac33c4ce331e166d` |
| hancom-blackwhite-zero-2024 | pdf | `pdf/issue7751/hancom-blackwhite-zero-2024.pdf` | `73d64173e0d8e106929adfc0fd9d3c8ab393ceea981872eed6329e63c513c69b` |
| hancom-blackwhite-axis-probe-2024 | hwp | `tests/fixtures/issue_7751_color_controls/hancom-blackwhite-axis-probe-2024.hwp` | `e45bf1d4e57802a39496f3b993495bbb1400ac965df19250237d2c943b6ee464` |
| hancom-blackwhite-axis-probe-2024 | pdf | `pdf/issue7751/hancom-blackwhite-axis-probe-2024.pdf` | `9c84d7ca1f199d213f71cb74420bd6017d1970e7fe769c27f5d71880c3c60a08` |

극한 PDF 실행 경로는 처음 MCP가 저장한 ignored `output/pr-review/seo-rii-20261017/mcp-print-pdfs/equation-lim-2020.pdf`였고, 위 저장소 `pdf/seo-rii-20261017/equation-lim-2020.pdf`와 바이트/SHA가 같음을 재확인했다. 이 경로 차이를 새 Print나 다른 입력으로 표시하지 않는다.

### 실제 producer 산출물

| 산출물 | source SHA | 파일 SHA-256 |
| --- | --- | --- |
| latest-base-native-build | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | `2dbcc1d896eb9def0069c4c0046580d0dd8ab48fff50dd388d475ddb0414b005` |
| latest-base-wasm-build / rhwp_bg.wasm | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | `32c750752521b18acd82f6b0f22a51ffb96e49008cceb32d79d51e6c849f1b37` |
| latest-base-wasm-build / rhwp.js | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | `894c403edf1c7746a29680cfb79e3ec9233739903194d4c0ef66362ae3212eb5` |
| latest-candidate-native-build | `3535106051394672e93ca8401ff9eae8b449ec20` | `94a0efaed8f661eabeaf6d7717ea557eb20cefd1642acb057cad66bc27a7799b` |
| latest-candidate-wasm-build / rhwp_bg.wasm | `3535106051394672e93ca8401ff9eae8b449ec20` | `2de4e28ac158955f7079952b79be5e5b5ad1b7ee9361fb7ba7c9932a2971a30c` |
| latest-candidate-wasm-build / rhwp.js | `3535106051394672e93ca8401ff9eae8b449ec20` | `779a6d6287048fa2fde92f82c460d8f737fbeb31aebe8c8ce16654788244ecf4` |
| latest-ffi-enabled-artifact.json | `3535106051394672e93ca8401ff9eae8b449ec20` | `851786bd50cc8d3dcce51832b94807a6a4f19d1a3c4f364c861d58106c3a0de5` |
| latest-ffi-disabled-artifact.json | `3535106051394672e93ca8401ff9eae8b449ec20` | `3ab21de8cb7ef020b3eef384cfae2c0ca963ac93374889c2bf19f66e19c95960` |

### 실제 시각 검증·빌드 명령

공통 환경은 `CARGO_BUILD_JOBS=8`, 공용 `CARGO_TARGET_DIR=/home/tsjang/rhwp/target/pr-review`, `RHWP_FONT_PATH=/opt/hnc/hoffice11/Shared/TTF/Install:/opt/hnc/hoffice11/Shared/TTF/Hwp:/opt/hnc/hoffice11/Shared/TTF/All`이다. WASM 빌드는 `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16`과 host `--no-opt`를 사용했다. WASM 캡처는 `VISUAL_SWEEP_CHROME=/home/tsjang/.cache/puppeteer/chrome/linux-154.0.8037.57/chrome-linux64/chrome`을 사용했다. 아래 output 경로는 당시 실행 경로이며 최종 TSV·대표 PNG는 위 Git asset 경로에 보존한다. 같은 입력/Print의 source·실행 파일 해시를 위 표와 함께 확인한다.

<details><summary>실제4개 빌드와52개 전쪽 TSV 명령</summary>

- latest-base-native-build / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`: `cargo build --locked --profile release-test --features native-skia --bin rhwp --target-dir /home/tsjang/rhwp/target/pr-review`
- latest-base-wasm-build / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`: `scripts/wasm-pack-locked.sh --target web --out-dir /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-pkg --no-opt`
- latest-candidate-native-build / producer `3535106051394672e93ca8401ff9eae8b449ec20`: `cargo build --locked --profile release-test --features native-skia --bin rhwp --target-dir /home/tsjang/rhwp/target/pr-review`
- latest-candidate-wasm-build / producer `3535106051394672e93ca8401ff9eae8b449ec20`: `scripts/wasm-pack-locked.sh --target web --out-dir /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-pkg --no-opt`
- latest-base-native-hancom-color-probe-2024 / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-color-probe-2024.hwp --pdf pdf/issue7751/hancom-color-probe-2024.pdf --key hancom-color-probe-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-native-scores`
- latest-base-native-hancom-color-probe-cross-2024 / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-color-probe-cross-2024.hwp --pdf pdf/issue7751/hancom-color-probe-cross-2024.pdf --key hancom-color-probe-cross-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-native-scores`
- latest-base-native-hancom-color-probe-boundary-2024 / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-color-probe-boundary-2024.hwp --pdf pdf/issue7751/hancom-color-probe-boundary-2024.pdf --key hancom-color-probe-boundary-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-native-scores`
- latest-base-native-hancom-blackwhite-zero-2024 / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-blackwhite-zero-2024.hwp --pdf pdf/issue7751/hancom-blackwhite-zero-2024.pdf --key hancom-blackwhite-zero-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-native-scores`
- latest-base-native-hancom-blackwhite-axis-probe-2024 / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-blackwhite-axis-probe-2024.hwp --pdf pdf/issue7751/hancom-blackwhite-axis-probe-2024.pdf --key hancom-blackwhite-axis-probe-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-native-scores`
- latest-base-native-lseg-02-mixed / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/lseg-02-mixed.hwp --pdf pdf/lseg-02-mixed-hwp-2020.pdf --key lseg-02-mixed --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-native-scores`
- latest-base-native-lseg-01-basic / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/lseg-01-basic.hwp --pdf pdf/lseg-01-basic-hwp-2020.pdf --key lseg-01-basic --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-native-scores`
- latest-base-native-shape-group-02 / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/shape-group-02.hwp --pdf pdf/shape-group-02-hwp-2020.pdf --key shape-group-02 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-native-scores`
- latest-base-native-pic-crop-01 / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/pic-crop-01.hwp --pdf pdf/pic-crop-01-2022.pdf --key pic-crop-01 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-native-scores`
- latest-base-native-equation-lim-print2020 / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit1: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/equation-lim.hwp --pdf /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/mcp-print-pdfs/equation-lim-2020.pdf --key equation-lim-print2020 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-native-scores`
- latest-base-native-form-hwpx / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/hwpx/form-01.hwpx --pdf samples/hwpx/form-01-2024.pdf --key form-hwpx --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-native-scores`
- latest-base-native-form-hwp-manual-crossformat / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/form-01.hwp --pdf samples/hwpx/form-01-2024.pdf --key form-hwp-manual-crossformat --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-native-scores`
- latest-base-native-native-ime / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp mydocs/pr/assets/issue_536_seo_rii_20261017/native-ime.hwpx --pdf pdf/seo-rii-20261017/native-ime-2020.pdf --key native-ime --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-native-scores`
- latest-base-wasm-lseg-02-mixed / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/lseg-02-mixed.hwp --pdf pdf/lseg-02-mixed-hwp-2020.pdf --key lseg-02-mixed --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-pkg`
- latest-base-wasm-pic-crop-01 / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/pic-crop-01.hwp --pdf pdf/pic-crop-01-2022.pdf --key pic-crop-01 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-pkg`
- latest-base-wasm-shape-group-02 / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/shape-group-02.hwp --pdf pdf/shape-group-02-hwp-2020.pdf --key shape-group-02 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-pkg`
- latest-base-wasm-lseg-01-basic / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/lseg-01-basic.hwp --pdf pdf/lseg-01-basic-hwp-2020.pdf --key lseg-01-basic --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-pkg`
- latest-base-wasm-equation-lim-print2020 / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit1: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/equation-lim.hwp --pdf /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/mcp-print-pdfs/equation-lim-2020.pdf --key equation-lim-print2020 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-pkg`
- latest-base-wasm-form-hwpx / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/hwpx/form-01.hwpx --pdf samples/hwpx/form-01-2024.pdf --key form-hwpx --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-pkg`
- latest-base-wasm-form-hwp-manual-crossformat / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/form-01.hwp --pdf samples/hwpx/form-01-2024.pdf --key form-hwp-manual-crossformat --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-pkg`
- latest-base-wasm-native-ime / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp mydocs/pr/assets/issue_536_seo_rii_20261017/native-ime.hwpx --pdf pdf/seo-rii-20261017/native-ime-2020.pdf --key native-ime --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-pkg`
- latest-base-wasm-hancom-color-probe-2024 / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-color-probe-2024.hwp --pdf pdf/issue7751/hancom-color-probe-2024.pdf --key hancom-color-probe-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-pkg`
- latest-base-wasm-hancom-color-probe-cross-2024 / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-color-probe-cross-2024.hwp --pdf pdf/issue7751/hancom-color-probe-cross-2024.pdf --key hancom-color-probe-cross-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-pkg`
- latest-base-wasm-hancom-color-probe-boundary-2024 / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-color-probe-boundary-2024.hwp --pdf pdf/issue7751/hancom-color-probe-boundary-2024.pdf --key hancom-color-probe-boundary-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-pkg`
- latest-base-wasm-hancom-blackwhite-zero-2024 / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-blackwhite-zero-2024.hwp --pdf pdf/issue7751/hancom-blackwhite-zero-2024.pdf --key hancom-blackwhite-zero-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-pkg`
- latest-base-wasm-hancom-blackwhite-axis-probe-2024 / producer `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-blackwhite-axis-probe-2024.hwp --pdf pdf/issue7751/hancom-blackwhite-axis-probe-2024.pdf --key hancom-blackwhite-axis-probe-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-base-wasm-pkg`
- latest-candidate-native-lseg-01-basic / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/lseg-01-basic.hwp --pdf pdf/lseg-01-basic-hwp-2020.pdf --key lseg-01-basic --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-native-scores`
- latest-candidate-native-lseg-02-mixed / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/lseg-02-mixed.hwp --pdf pdf/lseg-02-mixed-hwp-2020.pdf --key lseg-02-mixed --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-native-scores`
- latest-candidate-native-pic-crop-01 / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/pic-crop-01.hwp --pdf pdf/pic-crop-01-2022.pdf --key pic-crop-01 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-native-scores`
- latest-candidate-native-shape-group-02 / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/shape-group-02.hwp --pdf pdf/shape-group-02-hwp-2020.pdf --key shape-group-02 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-native-scores`
- latest-candidate-native-equation-lim-print2020 / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/equation-lim.hwp --pdf /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/mcp-print-pdfs/equation-lim-2020.pdf --key equation-lim-print2020 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-native-scores`
- latest-candidate-native-form-hwpx / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/hwpx/form-01.hwpx --pdf samples/hwpx/form-01-2024.pdf --key form-hwpx --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-native-scores`
- latest-candidate-native-form-hwp-manual-crossformat / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/form-01.hwp --pdf samples/hwpx/form-01-2024.pdf --key form-hwp-manual-crossformat --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-native-scores`
- latest-candidate-native-native-ime / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp mydocs/pr/assets/issue_536_seo_rii_20261017/native-ime.hwpx --pdf pdf/seo-rii-20261017/native-ime-2020.pdf --key native-ime --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-native-scores`
- latest-candidate-native-hancom-color-probe-2024 / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-color-probe-2024.hwp --pdf pdf/issue7751/hancom-color-probe-2024.pdf --key hancom-color-probe-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-native-scores`
- latest-candidate-native-hancom-color-probe-cross-2024 / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-color-probe-cross-2024.hwp --pdf pdf/issue7751/hancom-color-probe-cross-2024.pdf --key hancom-color-probe-cross-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-native-scores`
- latest-candidate-native-hancom-color-probe-boundary-2024 / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-color-probe-boundary-2024.hwp --pdf pdf/issue7751/hancom-color-probe-boundary-2024.pdf --key hancom-color-probe-boundary-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-native-scores`
- latest-candidate-native-hancom-blackwhite-zero-2024 / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-blackwhite-zero-2024.hwp --pdf pdf/issue7751/hancom-blackwhite-zero-2024.pdf --key hancom-blackwhite-zero-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-native-scores`
- latest-candidate-native-hancom-blackwhite-axis-probe-2024 / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-blackwhite-axis-probe-2024.hwp --pdf pdf/issue7751/hancom-blackwhite-axis-probe-2024.pdf --key hancom-blackwhite-axis-probe-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-native-scores`
- latest-candidate-wasm-lseg-01-basic / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/lseg-01-basic.hwp --pdf pdf/lseg-01-basic-hwp-2020.pdf --key lseg-01-basic --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-pkg`
- latest-candidate-wasm-lseg-02-mixed / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/lseg-02-mixed.hwp --pdf pdf/lseg-02-mixed-hwp-2020.pdf --key lseg-02-mixed --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-pkg`
- latest-candidate-wasm-pic-crop-01 / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/pic-crop-01.hwp --pdf pdf/pic-crop-01-2022.pdf --key pic-crop-01 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-pkg`
- latest-candidate-wasm-shape-group-02 / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/shape-group-02.hwp --pdf pdf/shape-group-02-hwp-2020.pdf --key shape-group-02 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-pkg`
- latest-candidate-wasm-equation-lim-print2020 / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/equation-lim.hwp --pdf /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/mcp-print-pdfs/equation-lim-2020.pdf --key equation-lim-print2020 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-pkg`
- latest-candidate-wasm-form-hwpx / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/hwpx/form-01.hwpx --pdf samples/hwpx/form-01-2024.pdf --key form-hwpx --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-pkg`
- latest-candidate-wasm-form-hwp-manual-crossformat / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp samples/form-01.hwp --pdf samples/hwpx/form-01-2024.pdf --key form-hwp-manual-crossformat --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-pkg`
- latest-candidate-wasm-native-ime / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp mydocs/pr/assets/issue_536_seo_rii_20261017/native-ime.hwpx --pdf pdf/seo-rii-20261017/native-ime-2020.pdf --key native-ime --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2022 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-pkg`
- latest-candidate-wasm-hancom-color-probe-2024 / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-color-probe-2024.hwp --pdf pdf/issue7751/hancom-color-probe-2024.pdf --key hancom-color-probe-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-pkg`
- latest-candidate-wasm-hancom-color-probe-cross-2024 / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-color-probe-cross-2024.hwp --pdf pdf/issue7751/hancom-color-probe-cross-2024.pdf --key hancom-color-probe-cross-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-pkg`
- latest-candidate-wasm-hancom-color-probe-boundary-2024 / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-color-probe-boundary-2024.hwp --pdf pdf/issue7751/hancom-color-probe-boundary-2024.pdf --key hancom-color-probe-boundary-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-pkg`
- latest-candidate-wasm-hancom-blackwhite-zero-2024 / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-blackwhite-zero-2024.hwp --pdf pdf/issue7751/hancom-blackwhite-zero-2024.pdf --key hancom-blackwhite-zero-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-pkg`
- latest-candidate-wasm-hancom-blackwhite-axis-probe-2024 / producer `3535106051394672e93ca8401ff9eae8b449ec20` / 실제 exit0: `python3 scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/issue_7751_color_controls/hancom-blackwhite-axis-probe-2024.hwp --pdf pdf/issue7751/hancom-blackwhite-axis-probe-2024.pdf --key hancom-blackwhite-axis-probe-2024 --rhwp-bin /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-rhwp --compat 2024 --dpi 96 --embed-fonts full --page-workers 4 --out /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-scores --wasm-pkg /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/latest-candidate-wasm-pkg`

기준9415의 극한은 Native88.42676%, WASM89.77376%로 exit1이며 원 미달 증거로 보존했다. 후보353의90% 통과·무하락 판정과 구분한다. 동일 기준 산출물은 재실행하지 않고 binary·입력·PNG·TSV 해시를 대조해 재사용했다. `--silhouette-only`의 `not_evaluated`를 승인으로 간주하지 않았으며 전쪽/대표 판독과 구조·실제 API 검사를 별도로 수행했다.

</details>

## 통합 PR #7969 code head CI 완료

검증한 통합 code head `c5b99e2c2825f1c56e9750134fc5722f916a8a8b`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/39007700375) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39007700497) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39007700494) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39007700528) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39007700524) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39008935219)가 실제 성공했다. CodeQL의 JavaScript/TypeScript·Python·Rust·Actions Analyze와 GHAS CodeQL check도 성공했다. Full CI의 Native·lint·frontend, archive A/B/C/D builder·worker가 실제 성공했으며, 별도 WASM Build job의 skip과 frontend fresh WASM 성공을 구분한다.

로컬 검증 producer `3535106051394672e93ca8401ff9eae8b449ec20` 뒤의 code/asset head `c5b99e2c2825f1c56e9750134fc5722f916a8a8b`는 문서·증거만 추가했다. 전쪽 raw·실제 ABI·브라우저 결과와 source의 동일성을 확인했다. 기존 로컬 실행을 다시 돌리지 않는다.

원 PR20개의 review·적용 기록과 오늘할일을 같은 PR의 single-parent trailing commit으로 반영한다. 최신 devel의 오늘 기록은 merge-tree와 링크·기록 보존 검사로 확인한다. 아직 실행 전인 trailing CI·merge·후속 처리는 완료로 기록하지 않는다.
