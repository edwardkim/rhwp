# 최종 source의 실제 로컬 Rust 게이트

- 실행 source: `4deadfa9638baffc3b4dc7e9d56a3edff9f83aeb`
- 실행 cwd: `/home/edward/mygithub/rhwp-worktrees/review-integration-9076-johndoekim-fields`
- 공용 target: `/home/edward/mygithub/rhwp/target/pr-review`; Cargo 호출은 순차 실행했다.
- 전체·집중·Native 검사는 아래 격리 prefix를 사용했다. 사설 글꼴·Windows 경로를 노출하지 않았고 Fontconfig의 실제 공개 글꼴 5개를 확인했다.
- 원시 로그·중간 JSON은 Git 제외 경로에 두고 이 표에 실제 명령·종료·시간을 기록한다.

## 최소 글꼴 실행 prefix

```bash
rhwp_minimal=( bwrap --ro-bind /home/edward/.nvm/versions/node/v24.15.0 /home/edward/.nvm/versions/node/v24.15.0 --ro-bind /usr /usr --symlink usr/bin /bin --symlink usr/lib /lib --symlink usr/lib64 /lib64 --ro-bind /etc /etc --proc /proc --dev /dev --ro-bind /sys /sys --bind /tmp /tmp --dir /home/edward --bind /home/edward/mygithub/rhwp /home/edward/mygithub/rhwp --bind /home/edward/mygithub/rhwp-worktrees/review-integration-9076-johndoekim-fields /home/edward/mygithub/rhwp-worktrees/review-integration-9076-johndoekim-fields --bind /home/edward/.cargo /home/edward/.cargo --ro-bind /home/edward/.rustup /home/edward/.rustup --tmpfs /usr/share/fonts --tmpfs /usr/local/share/fonts --setenv FONTCONFIG_FILE /home/edward/mygithub/rhwp/output/pr-review/johndoekim-20261116/minimal-font/fonts.conf --unsetenv RHWP_FONT_PATH --chdir /home/edward/mygithub/rhwp-worktrees/review-integration-9076-johndoekim-fields sh -c 'fc-list -f "%{file}\n"; exec "$@"' sh )
```

아래 격리된 명령은 이 prefix 뒤에서 실행했다. `suite-prepare`·format·lint·workspace build는 같은 cwd의 일반 실행이며 격리 열에 구분했다.

| 실행 | 격리 | 종료 | 초 | 실제 Cargo/Node 명령 |
| --- | --- | ---: | ---: | --- |
| suite-prepare | 일반 | 0 | 2.2 | `/home/edward/.nvm/versions/node/v24.15.0/bin/node scripts/rust-test-suite-manifest.mjs --prepare` |
| focused-issue_9076_textbox_unfilled_field_rows | 최소 글꼴 | 0 | 311.2 | `/home/edward/.nvm/versions/node/v24.15.0/bin/node scripts/run-rust-test.mjs issue_9076_textbox_unfilled_field_rows -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --test-threads 8` |
| focused-issue_9082_hwp5_filled_field_end_before_trailing_controls | 최소 글꼴 | 0 | 1.5 | `/home/edward/.nvm/versions/node/v24.15.0/bin/node scripts/run-rust-test.mjs issue_9082_hwp5_filled_field_end_before_trailing_controls -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --test-threads 8` |
| focused-issue_7732_split_span_frame_alignment | 최소 글꼴 | 0 | 14.4 | `/home/edward/.nvm/versions/node/v24.15.0/bin/node scripts/run-rust-test.mjs issue_7732_split_span_frame_alignment -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --test-threads 8` |
| focused-issue_7732_table_outer_margin_ownership | 최소 글꼴 | 0 | 13.1 | `/home/edward/.nvm/versions/node/v24.15.0/bin/node scripts/run-rust-test.mjs issue_7732_table_outer_margin_ownership -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --test-threads 8` |
| focused-issue_7732_print_ttf_em_grid | 최소 글꼴 | 0 | 16.4 | `/home/edward/.nvm/versions/node/v24.15.0/bin/node scripts/run-rust-test.mjs issue_7732_print_ttf_em_grid -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --test-threads 8` |
| focused-issue_7732_svg_print_stroke_coverage | 최소 글꼴 | 0 | 20.8 | `/home/edward/.nvm/versions/node/v24.15.0/bin/node scripts/run-rust-test.mjs issue_7732_svg_print_stroke_coverage -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --test-threads 8` |
| focused-issue_7732_native_svg_program_supply | 최소 글꼴 | 0 | 12.9 | `/home/edward/.nvm/versions/node/v24.15.0/bin/node scripts/run-rust-test.mjs issue_7732_native_svg_program_supply -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --test-threads 8` |
| focused-canvaskit_document_preflight_work_budget | 최소 글꼴 | 0 | 14.7 | `/home/edward/.nvm/versions/node/v24.15.0/bin/node scripts/run-rust-test.mjs canvaskit_document_preflight_work_budget -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --test-threads 8` |
| focused-issue_6117_layer_decoration_trim | 최소 글꼴 | 0 | 1.2 | `/home/edward/.nvm/versions/node/v24.15.0/bin/node scripts/run-rust-test.mjs issue_6117_layer_decoration_trim -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --test-threads 8` |
| format | 일반 | 0 | 35.0 | `cargo fmt --all -- --check` |
| clippy-native | 일반 | 0 | 74.6 | `cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings` |
| clippy-wasm | 일반 | 0 | 65.7 | `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings` |
| workspace-build | 일반 | 0 | 93.5 | `cargo build --locked --workspace --target-dir /home/edward/mygithub/rhwp/target/pr-review` |
| clippy-all-targets | 일반 | 0 | 98.4 | `cargo clippy --locked --workspace --all-targets --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings` |
| full-regressions | 최소 글꼴 | 0 | 1216.2 | `cargo nextest run --locked --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --tests --test-threads 14 --no-fail-fast` |
| native-lib | 최소 글꼴 | 0 | 372.0 | `cargo test --locked --profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --lib -- --test-threads 14` |
| native-placeholder | 최소 글꼴 | 0 | 304.8 | `/home/edward/.nvm/versions/node/v24.15.0/bin/node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia` |
| native-direct-pdf | 최소 글꼴 | 0 | 14.7 | `/home/edward/.nvm/versions/node/v24.15.0/bin/node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia` |

전체 회귀 13,632개·집중 30개·Native 공식 4,073개는 서로 다른 범위이며 합산해 고유 검사 수로 주장하지 않는다. Native lib의 실제 4,035개(3853+15+165+2), placeholder 2개, direct PDF 36개를 확인했다. 기존 ignored/skipped는 전체 45개·Native lib 12개이며 글꼴 설치 여부로 새 skip을 추가하지 않았다.
