# chrisryugj HWP3 PR 2건 검증 증적

검증일: 2026-11-09 KST. base `b768bee1c4c0023087687f79a0a95f12108743e9`, runtime 보정 `4fabb47ae28b99d371e2aac3560e2610e327c311`. 원 기여 두 head는 merge commit으로 보존했다. 이후 문서와 테스트 보강은 runtime 입력을 바꾸지 않는다. 원격 push·공개 comment·새 PR·merge는 아직 수행하지 않았다.

## 원본과 독립 기준

| key | HWP3 입력 / SHA-256 | 독립 한컴 Print / SHA-256 | 전체 쪽 |
| --- | --- | --- | ---: |
| so-sueop | [samples/SO-SUEOP.hwp](../../../../samples/SO-SUEOP.hwp)<br>`b5e410d4972b988240eb79f462c75ab3f92ce3415af6ed7f411906cc7498396c` | [pdf/SO-SUEOP-hwp-2020.pdf](../../../../pdf/SO-SUEOP-hwp-2020.pdf)<br>`12f7e011dcf1aaf90484a79c60b7ca853dcea860f135263ac41abf7c9ae613a7` | 46 |
| sample16 | [samples/hwp3-sample16.hwp](../../../../samples/hwp3-sample16.hwp)<br>`559fd94860cf836d5800436d56055c23a73e59eedef3da273a5ba365e6ed9e16` | [pdf/hwp3-sample16-current-print-20261017-2020.pdf](../../../../pdf/hwp3-sample16-current-print-20261017-2020.pdf)<br>`8f0bfd27ae567481e244f39de6f7b596b3afa8b227dce895d4ebf9cebc416bd0` | 64 |
| cell-order | [samples/hwp3-table-cell-order.hwp](../../../../samples/hwp3-table-cell-order.hwp)<br>`d9e30ed3a46e20b358c89eb1b26e14cf031f78e5192cea61752fed77f8dd939d` | [pdf/issue7642-hwp3-realign-controls-20261012/hwp3-table-cell-order-hancom-2020.pdf](../../../../pdf/issue7642-hwp3-realign-controls-20261012/hwp3-table-cell-order-hancom-2020.pdf)<br>`eb56a4e7a7638e777919bec8082d187ef98a3b48c6e1fc7d06d258454a4cd2aa` | 1 |

2020 Print는 compat 2022이며 원본 PDF를 변환·교체하지 않았다. Native Skia 직접 PNG와 실제 Chromium Canvas2D의 fresh WASM 직접 PNG를 96 DPI·print profile에서 비교했다. SVG raster를 이 결과로 표현하지 않는다.

## 실제 producer

| 경로 | source / 동등성 | 실제 실행 파일·WASM SHA-256 |
| --- | --- | --- |
| base Native | `2320351d2325a41d900e91f4e07f559d4a924a7d`의 이전 성공 producer. 현재 base와 `src crates Cargo.toml Cargo.lock assets`의 실제 Git diff가 0임을 먼저 확인하고 immutable copy 사용 | `c611a9264dcd52913e4002582ae9b3e703ffe0da2d207cc819eed1e93f3c7c94` |
| corrected Native | `d2137e0ccf7f28a1bcfc294cad669de923e596ac`, runtime source는 위 보정과 동일. 최종 재빌드 309.714초, exit 0 | `1ea5a54d0352f84cf930b06685cd77b312872f0f476910a416bb442e912aeb48` |
| base WASM | `b768bee1c4c0023087687f79a0a95f12108743e9`, compile + wasm-opt 832.142초, exit 0 | `2f972bd5d4eed4f3991d3af91adbafc22b6d33740b99f4a499fa0e35ddaf9af2` |
| corrected WASM | `4fabb47ae28b99d371e2aac3560e2610e327c311`, compile + wasm-opt 772.832초, exit 0 | `e9365e92c5f9b665ac9438ea0af8f04f0f94da5406f503e723f547a3c4ec9561` |

최종 Native는 source 보정 후 올바른 edition 2021 포맷까지 반영해 다시 빌드했다. 초기 진단 binary를 최종 producer로 사용하지 않았다. WASM은 Docker Compose에서 Rust 1.93.1·wasm-pack 0.15.0·release·codegen-units 16·wasm-opt를 완료했다. 모든 Cargo 작업은 기본 작업공간의 물리적 `target/pr-review`를 순차 재사용했다. WASM glue와 package hash는 실제 브라우저 소비 기록에서도 확인했다.

같은 `/app` mount 경로의 mtime 캐시로 변경 source를 다시 컴파일하지 않은 후보 실행 두 건은 중지하고 증거에서 제외했다. 호스트의 실제 source 절대 경로를 컨테이너에서도 사용하고 변경 파일의 mtime을 갱신한 뒤 `Compiling rhwp ...`가 실행된 최종 성공만 수용했다. 최종 build의 `src/parser/hwp3/mod.rs` Git blob은 `73dc66969f9147c5c591586e612408b7c7c0dc2b`다. 제품 source나 검사 기준을 우회한 변경은 없다.

## 전쪽 시각 측정

지표는 색상 경계 보정 전 2px `silhouette_raw_match_percent`다. 3문서의 전체 111쪽씩을 base/후보와 Native/fresh WASM 양쪽에서 대응시켰다. 쪽수 누락이나 대응 수 불일치는 없다.

| 문서 | backend | base 최저 | 후보 최저 | 최대 하락 %p | 후보 90% 미만 쪽 | 전쪽 TSV |
| --- | --- | ---: | ---: | ---: | --- | --- |
| so-sueop | native | 61.69502% | 61.69802% | 0.31024 | 11, 14, 21, 27, 32, 43, 44, 45, 46 | [base](base-so-sueop-native.tsv) · [후보](corrected-so-sueop-native.tsv) |
| sample16 | native | 98.53716% | 99.57001% | 0.00000 | 없음 | [base](base-sample16-native.tsv) · [후보](corrected-sample16-native.tsv) |
| cell-order | native | 100.00000% | 100.00000% | 0.00000 | 없음 | [base](base-cell-order-native.tsv) · [후보](corrected-cell-order-native.tsv) |
| so-sueop | wasm | 61.97765% | 61.97235% | 0.27353 | 11, 14, 27, 43, 44, 45, 46 | [base](base-so-sueop-wasm.tsv) · [후보](corrected-so-sueop-wasm.tsv) |
| sample16 | wasm | 98.73826% | 99.62920% | 0.00000 | 없음 | [base](base-sample16-wasm.tsv) · [후보](corrected-sample16-wasm.tsv) |
| cell-order | wasm | 100.00000% | 100.00000% | 0.00000 | 없음 | [base](base-cell-order-wasm.tsv) · [후보](corrected-cell-order-wasm.tsv) |

sample16·cell-order는 두 경로의 모든 쪽에서 90% 이상이다. SO-SUEOP의 기존 미달과 [Native 표시 문제](so-sueop-visual-hold.md)는 해소·예외 승인되지 않았다. 평균값이나 2%p 이내 하락으로 이를 면제하지 않는다.

## 이웃 탐색 보정과 실제 내용 보존

원 기여의 `take(2048)` 뒤 실제 이웃을 찾지 못한 상태가 없음으로 변환되고, 표 오른쪽/아래쪽 끝까지 병합한 뒤 기존 겹침 해소가 뒤 셀을 버렸다. 원 head를 보존한 source `a90852db333d145ede87c15c9e5064029d20ab20`의 Native binary `bcbebde1c61a67d4072a20311c5e23937961a1f5aeb81f824cfd81679754cdcf`로 실제 parser 누락을 재현했다.

| 구조 | base 셀 수 | 원 기여 셀 수 | 보정 Native/WASM 셀 수 |
| --- | ---: | ---: | ---: |
| 2,048행 × 2열 | 4,096 | 4,096 | 4,096 / 4,096 |
| 2,049행 × 2열 | 4,098 | 4,097 | 4,098 / 4,098 |
| 2,050행 × 2열 | 4,100 | 4,098 | 4,100 / 4,100 |

보정 뒤 2·2,048·2,049·2,050행과 전치 열의 8개 실제 입력에서 모든 고유 텍스트·셀 주소·1×1 소유를 Native parser와 최적화 WASM의 실제 HWPX 저장 XML로 검증했다. 내용 누락·중복·스팬 오염은 0이다. 합성 입력은 공식 HWP3 10.6의 구조 계약이며 한컴 생성본·Print 시각 증거라고 표현하지 않는다. 재현 빌더와 관계 검사는 [정식 테스트](../../../../tests/cases/hwp3_table_grid_left_edges.rs)에 포함했다.

같은 구간 색인을 두 축에서 사용한다. 구성 O(n log n), 질의 O(log² n), 구간 저장 O(n log n)이며 word 셀 수에 의해 tree 깊이는 최대 16이다. 상한을 늘리거나 선형 탐색을 무제한으로 바꾸지 않았다. 1,820개 oracle 대조는 맞닿은 끝점·0폭/0높이·두 축을 포함하고 65,534개 geometry의 실행도 확인했다. 전체 프로세스의 메모리·실행 시간 절대 상한을 새로 보장한다는 뜻은 아니다.

## 원 네 표의 독립 XML 대조

| 입력의 대상 | base 행×열 | 보정/독립 행×열 | 모든 셀 주소·병합 일치 |
| --- | --- | --- | ---: |
| sample16 접수 | 16×5 | 15×5 | 37 / 37 |
| sample16 사업책임자 | 23×15 | 23×17 | 234 / 234 |
| sample16 경력사항 | 15×15 | 15×17 | 69 / 69 |
| cell-order | 28×12 | 29×12 | 64 / 64 |

기대값은 `samples/hwp3-sample16-hwp5.hwpx`와 `pdf/issue7642-hwp3-realign-controls-20261012/hwp3-table-cell-order-hancom-2020.hwpx`의 실제 XML이다. Native·브라우저 WASM의 저장 결과가 모든 셀에서 일치한다. 보정된 원 문서의 Native dump는 원 기여 후보와 바이트 단위로 같아 원 개선이 유지됨을 확인했다.

## 직접 판독과 대표 PNG

- 원 네 표의 전체 쪽과 상단·병합 경계·하단을 같은 배율로 비교했다. 내용·순서·셀 포함·경계가 유지되며 새 겹침·누락·중복은 관측하지 않았다. [sample16 56쪽](sample16-p056-comparison.png), [cell-order 전 1쪽](cell-order-p001-comparison.png). 51·57쪽도 같은 방식으로 읽었고 source/TSV에 포함했다.
- 문장부호는 6·10·13·23·25·29·35·45쪽의 선·따옴표·화살표·중간점·반각 책괄호·긴 줄표와 주변 간격을 확대 판독했다. [화살표](arrow-p13.png), [책괄호](pua-p25.png), [긴 줄표](symbols-p23.png), [기존 본문 흐름](legacy-p45.png). [별도 판독 결과와 보류 이유](so-sueop-visual-hold.md)를 따른다. 긴 줄표의 Native 잉크 겹침은 **기존 독립 한컴 HWPX를 base renderer로 열어도 재현**했으며, #8789의 새 decode 때문에 생긴 renderer 회귀로 분류하지 않는다. [분리 대조](base-independent-dash-control.png).

## 글꼴과 격리

시각 비교에는 사용자 제공 HFT 변환 OTF 디렉터리와 HCR Batang·HCR Dotum TTF, Gulim TTC를 명시했다. Native/Canvas에 동일한 프로그램 공급과 font environment 정책을 적용했다. 실제 cmap에 없는 glyph는 source 문자 보존과 표시의 차이를 구분했다. HMKMM.TTF 추가 대조도 미달을 해소하지 못했다. 프로그램·원시 SVG·생성 JSON·로그는 공개 commit에 넣지 않는다.

회귀·Native 공식 검사는 bwrap으로 시스템 font 디렉터리와 사용자/개인 글꼴 경로를 숨긴 최소 글꼴 환경에서 실행한다. 실제 Fontconfig 노출은 저장소의 NotoSansKR Regular/ExtraLight, NotoSansSymbols2, RHWPPublicBatang, RHWPOldHangul 5개이며 개인 HFT/Windows/HWP 글꼴 경로는 접근할 수 없음을 확인했다. 라이선스 확인된 bundle/개별 fixture의 명시 공급은 별도다. 시각 비교 글꼴 환경과 혼동하지 않는다.

## 편람과 필수 회귀

Native 편람 `--handbook-gate` 383쪽은 passed, 최저 99.96%, 기준 대비 regression 0, 450.565초다. [전쪽 TSV](handbook-native.tsv). KoPub webfont와 기존 #8153 기준을 사용했고 기준값을 갱신하지 않았다.

필수 로컬 검증은 모두 exit 0으로 완료했다. 최종 테스트 source는 `581d22855f60af2de5fed3208949f7dd81258a0a`이며 runtime source는 위 producer와 같다. 테스트 보강 중 발견한 ZIP 9 `name()`의 `Result` 반환형과 `Box<Table>` 복사 타입은 이 commit에서 고쳤다. 자동 수집 suite를 다시 생성한 뒤 최종 all-target Clippy·suite 정책·집중·전체·Native 검사를 실행했다. 생성 harness·manifest는 제출 변경에 포함하지 않는다.

아래 build·Clippy·test·nextest 명령에는 공통 `--locked`와 `--target-dir /home/edward/mygithub/rhwp/target/pr-review`를 사용했다. 집중 실행의 wrapper도 같은 target을 사용했다. 집중·전체·Native 실행은 위 최소 글꼴 환경에서 수행했다.

| 검증 | 실제 실행 범위 | 결과 |
| --- | --- | --- |
| format | `cargo fmt --all -- --check` | 통과 |
| 기본 Clippy | `cargo clippy -- -D warnings` | 통과 |
| WASM32 Clippy | `cargo clippy -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` | 통과 |
| workspace build | `cargo build --workspace` | 통과 |
| all-target Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | 통과 |
| suite 정책 | `node scripts/rust-test-suite-manifest.mjs --check --base-ref b768bee1c4c0023087687f79a0a95f12108743e9` | 통과 |
| 집중 격자 | `node scripts/run-rust-test.mjs hwp3_table_grid_left_edges -- --cargo-profile release-test --test-threads 8 --no-fail-fast` | 5 PASS |
| 집중 문장부호 | `node scripts/run-rust-test.mjs hwp3_so_sueop_private_punctuation -- --cargo-profile release-test --test-threads 8 --no-fail-fast` | 3 PASS |
| 전체 회귀 | `cargo nextest run --cargo-profile release-test --tests --test-threads 8 --no-fail-fast` | 13,013 PASS, 기존 45 skip, 실패 0 |
| Native lib | `cargo test --profile release-test --features native-skia --lib -- --test-threads 8` | 4,040 PASS, 기존 12 ignored |
| Native 그림 누락 | `node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --features native-skia` | 2 PASS |
| Native 직접 PDF | `node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --features native-skia` | 35 PASS |

Native 공식 3종 합계는 4,077 PASS다. 집중 실행에서 다른 테스트가 필터로 제외된 수는 위 전체 회귀의 기존 skip과 구분한다. 개인 글꼴 유무로 새 검사를 건너뛰지 않았다. 전체 회귀는 컴파일 포함 1,141.170초, 실제 실행 679.271초이며 Native 3종은 각각 353.910·293.942·12.234초였다. 이 수치는 검증 실행 기록이며 일반 사용자 성능 보장을 뜻하지 않는다.

2026-11-09 KST 최종 읽기 전용 API 점검에서 devel과 두 원 PR head는 접수 SHA 그대로였고 본인 계정의 열린 PR은 0건이었다. 새 통합 head의 GitHub CI는 아직 실행하지 않았다. #8791은 **메인터너 보정 후 수용 가능**, #8789는 **기존 시각 차이의 수용 결정 전 머지 보류**다. [원격 처리안](remote-action-plan.md)은 승인 요청용이며 완료 기록이 아니다.
