# 최신 통합 source의 전체 회귀·Native Skia 라이브러리 완료

검증 source `ba2ebfd34a4ebf402bce8e002254b672a09ef764`, 정책 비교 base `f529dc92d4d172bf32390d7749743a45d9e41bfc`.
검증 전후 Rust·Cargo·내장 자원 등 4,694개 파일의 고정 snapshot을 대조했다. 이후 문서 커밋은 제품 소스를 바꾸지 않는다.

| 완료 범위 | 실제 최종 결과 |
| --- | --- |
| 전체 integration nextest | **12,583 PASS·0 FAIL·45 SKIP·exit 0**. 실제 테스트 실행 919.391초 |
| Native Skia 라이브러리와 내부 crate | **4,040 PASS·0 FAIL·12 IGNORE·exit 0**. root 3,858개와 내부 crate 15·165·2개 |

회귀 환경은 시스템·사용자·상용 글꼴 경로를 숨기고 공개 자원 5개만 노출한 bwrap 환경이다.
`NotoSansKR-ExtraLight.ttf`, `NotoSansKR-Regular.ttf`, `NotoSansSymbols2-Regular.ttf`,
`RHWPOldHangul-Regular.otf`, `RHWPPublicBatang-Regular.ttf`를 실제 fc-list로 확인했다.
시각 비교의 시스템 글꼴 전체 공급과 구분하며, 설치된 비공개 OTF를 회귀 통과 조건으로 쓰지 않았다.

## 실행과 출처

Cargo는 같은 `/home/tsjang/rhwp/target/pr-review`를 순차 재사용했으며 build jobs 8, nextest threads 12다.
같은 target을 쓰는 다른 Cargo 작업은 겹치지 않았다. 각 명령의 실제 시작·종료는 컴파일 시간을 포함한다.

### full-nextest-minfont

실제 실행 head `dc703ef8180e0c410aa754011c34660e33e68480`. 시작 UTC `2026-11-05T01:47:35.341785+00:00` → 종료 UTC `2026-11-05T02:35:34.595430+00:00`.
로그 SHA-256 `d19e9baa93566191a436de9d0365fcdd5b267e2d71f824d8840b79f30152c459`.

```bash
bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh cargo nextest run --locked --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast
```

### skia-lib

실제 실행 head `0e412ebe8dc6aa726c52dcf359241207d045930d`. 시작 UTC `2026-11-05T02:35:34.690855+00:00` → 종료 UTC `2026-11-05T02:46:41.806814+00:00`.
로그 SHA-256 `5f668d59d7539c3b173ac3f13cc9d60398560f1b96076069dbd096435d409063`.

```bash
bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh cargo test --locked --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia --lib -- --test-threads 12
```

## 남은 공통 검증

필수 실행 41개가 모두 exit 0으로 완료됐다. CI Native 개별 모듈 21개도 모두 통과했고 doctest는 8 PASS·3 IGNORE다.
CI workflow에서 추출한 21개 모듈과 실행 계획의 21개 모듈이 정확히 일치하고 누락·추가가 없음을 확인했다.
보호 대상 c183 WASM 집계도 별도 완료했다. 새 base 실제 제품과의 기능 보존 대조·최종 리뷰 갱신 전에는 PR 준비 완료나 최종 승인으로 판정하지 않는다.
원시 로그·JSON은 로컬 output에 보관하며 제출 자료에는 복사하지 않는다.

## 필수 실행 41개 전체 결과

각 실행 전후 고정 snapshot을 대조했고 마지막 실행의 실제 종료 UTC는 `2026-11-05T03:05:21.823706+00:00`다. 로그·exit 파일과 명령의 대응을 확인했다.

| 실행 | 명령 | exit | 원시 로그 SHA-256 |
| --- | --- | --- | --- |
| manifest-prepare | `node scripts/rust-test-suite-manifest.mjs --prepare --base-ref f529dc92d4d172bf32390d7749743a45d9e41bfc` | 0 | `719fb4a7666f6ad6cfa2594a324a65fd13e96ca8e954156bd142319960d3d38c` |
| fmt | `cargo fmt --all -- --check` | 0 | `9be43f6943facc92153063e97f41e9fb37584d1a0f234674aa3333a63a232c6d` |
| manifest-check | `node scripts/rust-test-suite-manifest.mjs --check --base-ref f529dc92d4d172bf32390d7749743a45d9e41bfc` | 0 | `aed2152dbdf8c34035e66448b5b7da15f335e79228a7968c3a4ee9448c76147c` |
| unit-tiers-check | `node scripts/rust-unit-test-tiers.mjs --check --base-ref f529dc92d4d172bf32390d7749743a45d9e41bfc` | 0 | `078ce1de99f8801b969b024005a86ac085e6be00af2ce45416b2feb6c6557534` |
| native-root-clippy | `cargo clippy --locked --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings` | 0 | `36722d54d54e8cb69c023cb300cdd2cc4a8a07ced4a772c49ff6e76bf3b070c7` |
| wasm-clippy | `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings` | 0 | `980122691fb4534b1ef40d225d5855d193c26d5ad74ba31cd698adf183898f62` |
| workspace-build | `cargo build --locked --workspace --target-dir /home/tsjang/rhwp/target/pr-review` | 0 | `74549a3bc656fd71bbbb389e27c09ace3d7b2174a3ae22e2d7292882b5abb5f9` |
| workspace-clippy | `cargo clippy --locked --workspace --all-targets --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings` | 0 | `3cf90981f1cee84feaee82dc0a8f15d02ec4deb4fb4d95907ecd151eb0a77fa7` |
| focused-issue_8470_vertical_cell_content_axis | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_8470_vertical_cell_content_axis -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `9e2270d04e13a05685c78ebd7319b326978c108b69409cedbd35ad2202dfbb81` |
| focused-issue_8470_vertical_cell_table_no_split | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_8470_vertical_cell_table_no_split -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `4b5e7b1a21acece6f6669307f5ae35ad79a6a553e95baf73db57aad14d4bd660` |
| focused-issue_6029_vertical_cell_line_budget | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_6029_vertical_cell_line_budget -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `b0a37295b30cd2f6173b55ba1224d9ccd2b0f0a901d6bc14de4bd41ebdef6cf9` |
| focused-issue_8425_default_outline_head_distance | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_8425_default_outline_head_distance -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `7b258fe3765070ede431843346d23d5b79d37c9f6445e125bc442ec2b0ad9620` |
| focused-issue_8532_rowspan_hard_break | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_8532_rowspan_hard_break -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `5a60e060673ae9c72e3595e501f61d738a839acd85a07ef431bc0f211a9d4a55` |
| native-build | `cargo build --locked --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia --bin rhwp` | 0 | `a27c9cfefd95a74999c0936698cd909a08015c4d22095a4f346f8a7eb423d836` |
| pin-native | `python3 /home/tsjang/rhwp/output/pr-review/planet6897-20261103/pin_rebase8609_product.py native` | 0 | `a42ba105dbc29eec60282928b30d4642ecf40348ebcaa2c635c0f0fefb943408` |
| wasm-build | `sh scripts/wasm-pack-locked.sh --target web --out-dir /home/tsjang/rhwp/output/pr-review/planet6897-20261103/producer-packages/rebase8609-dda217dba0` | 0 | `67e5728744a96a901e878d49fc6d85a559c9f8ae93f5150ab1cba68e73592494` |
| pin-wasm | `python3 /home/tsjang/rhwp/output/pr-review/planet6897-20261103/pin_rebase8609_product.py wasm` | 0 | `bb7baa464653da23872472af9141c2a402640b06c6fa8a6b7921f732dcaa0d0b` |
| full-nextest-minfont | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh cargo nextest run --locked --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast` | 0 | `d19e9baa93566191a436de9d0365fcdd5b267e2d71f824d8840b79f30152c459` |
| skia-lib | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh cargo test --locked --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia --lib -- --test-threads 12` | 0 | `5f668d59d7539c3b173ac3f13cc9d60398560f1b96076069dbd096435d409063` |
| native-cli_exit_codes_native | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test cli_exit_codes_native -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `31d5e9c608c18c21cb077efe0e8159f9d119e497a6b91a0a538ff1b9241d89fc` |
| native-issue_1144_native | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_1144_native -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `603256d27763e2b92a70ed366743c4eedc0269f0eefbeaf4204d2d55af226c70` |
| native-issue_2225_missing_picture_placeholder | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_2225_missing_picture_placeholder -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `7eaacf2c44c08ffb9a7102f9056f58c9d1ac965b8a3d2fc40b9c82763101b14f` |
| native-render_p37_direct_pdf_export | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test render_p37_direct_pdf_export -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `71303ff6afbf574b9d548f94c8a609ba867ec99e1a103d33b561a92f62e9cd1d` |
| native-issue_7950_pdf_paint | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_7950_pdf_paint -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `361c98fcf8637c7ef80ae99c123cf9d34ca1fcb095b37704a6f2e9c7bbaf715a` |
| native-issue_2083_hide_fill_page_background | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_2083_hide_fill_page_background -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `1b380fd8804e550d82b2a62468d6e89353fa1293e1fa644ac9c0621b50c03e5b` |
| native-issue_2292_chart_png_clip | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_2292_chart_png_clip -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `4079369bf5ccdb88b202223ff6beab36b778f63e729287c32d0c68d6ad8c6788` |
| native-issue_2293_chart_png_text | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_2293_chart_png_text -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `9f29dc39dd7de020919de7235aa2d7e4d148a8f6b5e6ed3d86857ab41a85bffb` |
| native-issue_7820_sixel_contract | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_7820_sixel_contract -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `2c3e4f5acce052d1386c388655603a21e4d077dc2507d07f3dab8c210392bf39` |
| native-issue_8150_kitty_contract | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_8150_kitty_contract -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `a71cbcb68a5116b34f60bd0a32d369a68a2b2a2d845a83dc5945edaf1d7f152b` |
| native-issue_4961_font_decision_trace | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_4961_font_decision_trace -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `af3ec8725a916182def077ca7fbe4bef890bb2798ced3d1304c2f8237bd47e6e` |
| native-issue_7902_original_frame_and_content | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_7902_original_frame_and_content -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `1aba5c13d26d6ec71c464b00cb6670a309faa61e80dfabd7081f5fab98b8d7c0` |
| native-issue_8160_hwp2022_paragraph_relations | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_8160_hwp2022_paragraph_relations -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `7b3703fe61c59d8c8b644b604a17fb380077103cbbda48160ba59fccbab31be5` |
| native-native_fractional_glyph_origin | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test native_fractional_glyph_origin -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `d10bd02a86d90bf03976f5435b3f6e1834efda130953ac987d50e96116ff09fb` |
| native-skia_native_glyph_replay | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test skia_native_glyph_replay -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `ee599b69f7c69894008fca47bd43794a637a39b2d15eae4eac8b471b925734c7` |
| native-issue_8153_opaque_mixed_axis_image_sampling | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_8153_opaque_mixed_axis_image_sampling -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `b892e97ec7542c4dacba4a0f926c0db1d3bb44d884f73eca941bc959a8c16de4` |
| native-issue_8153_print_glyph_geometry | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_8153_print_glyph_geometry -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `c0a89ab21f1d146306af9aecef5d20fcf1bd40fed69de2248412de99a072d08e` |
| native-issue_8153_symbol_fallback | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_8153_symbol_fallback -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `0a066567567e983590e11e8c776cd141e5c42349b8c9724ff139295870180691` |
| native-issue_536_native_region_cache | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test issue_536_native_region_cache -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `165c8b34ae1d179225dbb6951db18afa8253bd2ba49c8dfde4707e83b31e2fc3` |
| native-skia_region_plan | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test skia_region_plan -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `434842847f4d8310de7aaf651f9e5cc96df47f834d6afe3133bbd83447b7cc51` |
| native-skia_host_surface | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node scripts/run-rust-test.mjs --cargo-test skia_host_surface -- --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 0 | `8d4715a0226526d5fd282f614c262eaf2f3af7be1bac1b507127891132e05b3c` |
| doc-tests | `bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh cargo test --locked --doc --target-dir /home/tsjang/rhwp/target/pr-review -- --test-threads 1` | 0 | `60ddbd8a029ac41db0bf403eddd1a1805a960c1169a552afd56009143fbca208` |

## 측정 주석 보완 이후의 실행 코드 보존

전체 41개가 종료된 뒤 `issue_8470_vertical_cell_content_axis.rs`의 첫머리 측정 주석만 최신 실제 양쪽 전 5쪽 100.00%와 직접 판독 결과로 갱신했다.
주석 갱신 시각과 Git에서 확인한 직전 파일 변경 시각·SHA를 구분했다. 테스트·helper 본문은 바이트 단위로 같고 production 코드는 바꾸지 않았다.
독립 `rustfmt --check`와 unit-tier 정책을 확인했다. 주석 바이트 수가 sharding 가중치에 포함되어 파생 harness가 달라졌으므로 정식 `--prepare` 뒤 manifest `--check --base-ref`를 통과시켰다.
파생 harness의 source module 다중집합이 이전과 같음을 확인했으며 생성 파일은 제출하지 않는다. 주석만 바뀌었으므로 전체 회귀·렌더·시각 측정은 반복하지 않았다.
