# lpaiu-cs 4개 PR 통합 보정·검증 기록

## 현재 결론

#7004·#7014·#7022는 승인, #7010은 메인터너 보정 후 수용 가능이다. #7010 보정은 적용·검증 완료 후 `d6b1c25bd584e766dd8121f3678a64919040a37d`로 커밋했다. 원 head와 메인터너 보정을 혼동하지 않는다.

## 검토 대상과 권한

- 검토일: 2026-09-11, 검토자: jangster77.
- 기본 경로: collaborator 매개 외부 PR. 원 PR의 사전 reviewer 할당은 완료했으며, 통합 PR의 owner 자동 지정과 구분한다.
- 작성자 lpaiu-cs는 기존 기여자다. 첫 기여자 절차를 새로 적용하지 않는다.
- 검토 브랜치: `review/lpaiu-cs-7004-7022-20260911`.
- 통합 기준: `upstream/devel b59323de0448df0a42bcb9cdd12cb80d51fe83d2`.
- 체리픽 통합 코드 head: `b788e340a1e5860a09bf89cc1f5fa71ab4b6b1e9`.
- 실제 검증 후보: 위 체리픽 head와 #7010 메인터너 보정 commit `d6b1c25bd584e766dd8121f3678a64919040a37d`의 3개 파일. [통합 검증 기록](pr_7004_review_impl.md)의 SHA-256으로 정확한 내용을 구분한다.
- 사용자가 통합 PR 생성을 승인했다. 검증한 코드와 리뷰·오늘할일·필수 PNG를 제출하며 원 PR/이슈 comment·close·merge는 아직 수행하지 않는다.

## 적용 순서와 원본 provenance

| 원 PR | 원 SHA -> 로컬 SHA |
| --- | --- |
| #7004 | `566d93eaa4be7e998cd19e96a4b48518fded5a5e -> 159b79a458df85d698d4798411de52d01f373ddc` |
| #7010 | `b6bcf361f36d63da73edf01f6d03376a9a7ac982 -> 9fccf9e175f4a3c32adfa81b3073c0d393a3f410` |
| #7010 | `29c9192f9b46d6b6c603ad1e455a997bd7f1ce2d -> 1cef159930b8888ad1c81379b62bafc3d315c272` |
| #7014 | `1c6e5dbc40f87a2bc09e1aacdd2d0056ec2c9b1d -> f190e1c6a9905dab95407095b2b60344f82fe752` |
| #7022 | `7de0bdc8dc230c09cbf0f32676d6bdb2554a0e24 -> b788e340a1e5860a09bf89cc1f5fa71ab4b6b1e9` |

4개 PR의 5개 원 커밋을 누적 적용했다. #7004/#7022의 Vite 파일 충돌은 b788e340a1e5860a09bf89cc1f5fa71ab4b6b1e9에서 두 변경의 의도를 보존해 해소했다.

## 메인터너 보정 commit과 검증 파일의 정확한 식별

| 파일 | SHA-256 |
| --- | --- |
| `rhwp-studio/tests/mutation-routing-guard.test.ts` | `e6aafc5108594738c8ecb34d5f5a53c18b10a1717da269bc3a4f40ceb13fbb53` |
| `rhwp-studio/tests/helpers/rust-mutating-exports.ts` | `8a7ef72d04289d2714979d6f3fe088609dc18679de3ac5c9593ae66f80f7f38f` |
| `rhwp-studio/tests/rust-mutating-exports.test.ts` | `6a8deea021fa99dc555fccbd6464d4b6d710bce9d91ea32ea9de9e7d53f0bcfe` |

#7010의 regex 감사 사각만 보정했다. production source/registry/baseline을 바꾸지 않았다. 최종 Node 집중·통합 검사는 위 파일 내용으로 다시 실행해 통과했다. 검증한 동일 내용의 보정 commit은 `d6b1c25bd584e766dd8121f3678a64919040a37d`다.

## 공통 로컬 검증 결과

| 검증 | 실제 결과 |
| --- | --- |
| Rust 집중 nextest | 11개 통과; 조각 삭제 복원 7개, 캐시 목록 1개, 셀 도형 경로 3개 |
| 전체 nextest, release-test, 8 threads, no-fail-fast | 9,471개 통과, 46개 건너뜀, 실패 0개; 실행 360.994초 |
| Rust export 감사 및 추출기 집중 테스트 | 최종 보정본 13개 통과 |
| Studio 및 npm/editor 통합 Node 테스트 | 최종 보정본 1,649개 통과, 2개 건너뜀 |
| 회전 oracle Python 테스트 | 8개 통과 |
| Undo-depth workflow Python 계약 | 5개 통과 |
| E2E 목록 검사 | 추적 파일 129개와 manifest 129행 일치 |
| cargo fmt --all -- --check | 통과 |
| native Clippy, WASM32 Clippy | 각각 -D warnings 통과 |
| workspace build, workspace all-target Clippy | 각각 통과 |
| Rust suite manifest --check | 1,258 sources, 28 suites + 20 exceptions, 48/48 targets 일치 |
| 잠금 파일 보호 wrapper WASM 빌드 | 통과; 210초, 새 pkg 사용 |
| Studio tsc + Vite production build | 통과 |
| 실제 Vite Undo 깊이 E2E | 110라운드, 이력 255개, 조각 삭제 103개, snapshot 슬롯 0, 키보드 Undo 255/255 통과 |

모든 Cargo 계열 명령은 Mac의 `target/pr-review`에서 순차 실행했다. 전체 회귀에 `cargo test --profile release-test --tests`를 사용하지 않았다. 원 PR의 CI 성공을 로컬 실행 결과로 바꿔 기록하지 않는다. 원시 로그·임시 스크립트·JSON은 커밋 대상에서 제외한다.

## 실제 실행 명령

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast -E 'test(/issue_5769_delete_fragment_byte_identity|issue_5890_raw_cache_inventory|issue_7005_cell_shape_cell_path/)'
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast
node --test rhwp-studio/tests/rust-mutating-exports.test.ts rhwp-studio/tests/mutation-routing-guard.test.ts
npm --prefix rhwp-studio test
python3 tools/hangul_rotation_oracle/test_oracle.py
python3 scripts/check_e2e_manifest.py
python3 -m unittest scripts.tests.test_undo_depth_e2e_workflow
cargo fmt --all -- --check
cargo clippy --locked --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg
npm --prefix rhwp-studio run build
CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' VITE_PORT=17716 npm --prefix rhwp-studio run e2e:undo-depth
```

위 Cargo 계열은 동시에 실행하지 않았다. WASM 성공 후 새 pkg로 브라우저를 실행했다.

## 시각 증적과 산출물 정책

#7014의 물리 1쪽 두 셀 도형 실제 클릭·Shift 선택·드래그·Ctrl+Z를 검증했다. 최종 1440×1100 viewport에서 앱 상단만 캡처한 PNG 4장을 `mydocs/pr/assets/pr_7004_7022_lpaiu_cs_20260911/`에 보관한다. 최종 폭 측정은 9070 → 6818 → 9070이며 Undo 뒤 전체 shape properties 일치까지 단언했다. 초기 CDP 800×600 viewport 측정 9070 → 5744 → 9070은 최종 증적의 수치로 재사용하지 않는다.

- [#7004 최종 리뷰](pr_7004_review.md)
- [#7010 최종 리뷰](pr_7010_review.md)
- [#7014 최종 리뷰 및 직접 표시 이미지](pr_7014_review.md)
- [#7022 최종 리뷰](pr_7022_review.md)

임시 스크립트·실행 로그·JSON·중간 SVG·중복 PDF는 커밋하지 않는다. 이번 검증은 기준 PDF 변환이 필요한 배치 일치 주장이 아니므로 PDF를 새로 만들지 않았다. GitHub 코멘트에는 #7014 리뷰에 준비한 raw 이미지 Markdown을 실제 merge SHA로 치환해 직접 표시한다. 사용자 승인으로 통합 PR을 제출한다. 원 PR/이슈 종료와 merge 후 코멘트는 아직 수행하지 않았다.
