---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-13
---

# #7078~#7091 체리픽 검토와 메인터너 보정 기록

- 브랜치: `review/pr7078-7091-20260913`.
- 기준 devel: `1ae5ca295bddcb31b846affc62834a2a3023d24d`.
- 검증 code head: `0acc011e34097b0c7ac8273d714f37b79608082f`.
- 현재 코드·증거 head: `e670fb245a15f7e281401045065552f4f638041b`. code head 이후 source/test/build/의존성 입력 diff는 없다.
- 사용자 승인으로 upstream `integration/pr7078-7091-20260913`에 push하고 [통합 PR #7102](https://github.com/edwardkim/rhwp/pull/7102)를 생성했다. 원 PR comment·close·merge는 미실행이다.

## 범위와 최종 판정

| 원 PR | 판정 | 고유 적용·보정과 잔여 범위 |
| --- | --- | --- |
| [#7078](pr_7078_review.md) | 메인터너 보정 후 수용 가능 | 원 공통 64띠 cap은 되돌림. `0acc011e3` PDF 함수 계층화로 대체 |
| [#7085](pr_7085_review.md) | 승인 | `a6ae31b80448210f0cfc7f4caf83fcfa610e3a32`, 중첩 표 세로 정렬 |
| [#7089](pr_7089_review.md) | 승인 | `fe5cb649c4d151f45fd37be67532c66ff89a87c6` + 반례 보강 `c1a4d3345`; #7086 잔여 상단 오프셋 open |
| [#7091](pr_7091_review.md) | 메인터너 보정 후 수용 가능 | `64605f37d` 재적용·독립 census/실제 최상위/재저장 검사. #4680 전체 범위 open |

#7082·#7083·#7087·#7088은 이전 #7093에서 반영되어 다시 적용하지 않았다.
#7079·#7080·#7081·#7084·#7086·#7090은 PR이 아닌 issue다.
원 contributor commit 이력과 cherry-pick 출처, 초기 보류와 되돌림을 유지했다. source PR branch는 건드리지 않았다.

## 보류 해제 근거

#7078은 공통 그러데이션 상한 대신 PDF 함수 표현에서 수정했다. 255 고유색·510 stops를 유지하면서
MuPDF 오류가 사라졌고, Poppler에서 보정 전 무상한 PDF와 보정 PDF의 실문서·합성 반례 두 페이지
전체 96dpi 픽셀이 동일했다. 독립 한컴 표본 픽셀 차이는 채널별 1이었다. 기존 색 함수 보존과
전체 문서 충실도를 구분한다. 기존 #6822의 step=100 및 baseline/golden을 변경하지 않았다.

#7091은 동일 입력의 한컴 PDF 변환이 개선된 MCP 서버에서 148초·151쪽으로 성공했다.
현재 변환 HWP5가 실제 한컴에 전달한 후보와 byte-identical이고, 새 PDF와 수정 전 PDF의 151쪽 픽셀·텍스트가 같다.
독립 HWP5 census 1,392/358개와 실제 최상위 25개, HWP5 재저장 Rust 검사 세 개도 통과했다.
HWP3 SVG 412쪽이 수정 전과 같으며 독립 PDF의 기존 본문 차이를 전체 해결로 주장하지 않는다.

## 완료한 로컬 검증

전용 target `target/pr7078-7091-20260913`에서 Cargo 명령을 순차 실행했다.

| 단계 | exit | wall seconds |
| --- | ---: | ---: |
| clippy-native | 0 | 27.35 |
| clippy-wasm | 0 | 12.97 |
| workspace-build | 0 | 42.88 |
| clippy-all-targets | 0 | 42.62 |
| manifest-check | 0 | 0.37 |
| unit-tiers | 0 | 0.76 |
| focused | 0 | 225.12 |
| full-nextest | 0 | 300.25 |
| native-skia-lib | 0 | 128.47 |
| native-skia-placeholder | 0 | 114.41 |
| native-skia-pdf | 0 | 5.71 |
| wasm-build | 0 | 139.73 |
| wasm-parity | 0 | 4.07 |

- 집중 nextest: **35 PASS**, 전체 nextest: **9,577 PASS / 46 skipped**, 테스트 실행 299.402초.
  전체에는 leaky 표시가 없었고 slow 2건이었다.
- Native Skia lib: **4,112 PASS / 13 ignored** (rhwp 3,930 + contracts 15 + chart 165 + crypto 2).
- Native Skia placeholder: **2 PASS**. direct PDF: **4 PASS**, 이 중 selection_errors 1건 leaky 표시.
- WASM native `--no-opt` build 성공. **4문서 31쪽 native/WASM SVG 불일치 0**.
  Docker daemon 연결 불가를 재확인했으므로 Docker 최적화 배포 build 통과로 표현하지 않는다.
- source/test/config 변경 뒤 세 Clippy와 workspace build·manifest·unit-tier 검사를 모두 실행했다.
  generated suite/manifest는 검토용으로만 준비했으며 stage하지 않았다.
- #7085/#7089의 기존 후보와 같은 CLI 옵션으로 재출력한 SVG 31쪽도 전부 byte-identical이다.
  이전 visual_sweep의 font-face 주입본을 CLI 원본과 직접 byte 비교하는 방법은 사용하지 않았다.
- 원 PR의 CI 결과를 현재 통합 head의 CI 성공으로 재사용하지 않았다. 통합 PR #7102의 code candidate `e670fb245` CI가 성공했다.

### Direct PDF leaky 재확인

`Summary [   0.027s] 1 test run: 1 passed, 186 skipped`. 최초 leaky 기록은 유지하며 테스트 통과와 프로세스 종료 표시를 구분한다.
이번 보정으로 leaky 원인까지 해결했다고 주장하지 않는다.

### 실행 명령

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
cargo clippy --locked --target-dir target/pr7078-7091-20260913 -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr7078-7091-20260913 -- -D warnings
cargo build --locked --workspace --target-dir target/pr7078-7091-20260913
cargo clippy --locked --workspace --all-targets --target-dir target/pr7078-7091-20260913 -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
node scripts/rust-unit-test-tiers.mjs --check
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr7078-7091-20260913 --tests --no-fail-fast -E 'test(/issue_7077_pdf_gradient|issue_7066|issue_7086|issue_6874_group_depth|issue_7008|issue_6787|issue_6708|issue_6822|issue_7047/)'
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr7078-7091-20260913 --tests --no-fail-fast
cargo test --locked --profile release-test --target-dir target/pr7078-7091-20260913 --features native-skia --lib
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir target/pr7078-7091-20260913 --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir target/pr7078-7091-20260913 --features native-skia
env CARGO_TARGET_DIR=target/pr7078-7091-20260913 scripts/wasm-pack-locked.sh --target web --out-dir /tmp/rhwp-review-7078-7091-20260913/maintainer/wasm-pkg --no-opt
node scripts/svg_native_wasm_diff.mjs --rhwp /tmp/rhwp-review-7078-7091-20260913/maintainer/after-rhwp --pkg /tmp/rhwp-review-7078-7091-20260913/maintainer/wasm-pkg --keep-match samples/issue2470/36382471_masked.hwpx samples/issue2083_hide_fill_page.hwpx 'samples/21_언어_기출_편집가능본.hwp' samples/issue7062/tac_object_host_line_height.hwp --out /tmp/rhwp-review-7078-7091-20260913/maintainer/wasm-parity
```

## 검증 입력 커밋 확인

확인한 commit `e670fb245a15f7e281401045065552f4f638041b`에서 **29개 입력**, 기존 19개·신규 10개, 전부 실행 바이트와 Git blob 일치·LFS 0이다.
기존 HWP/HWPX/PDF의 이름을 바꿔 중복 추가하지 않는다. 새 sample11 PDF는 기존 before PDF와 전체 픽셀·텍스트가 같아 job·출력 hash만 기록했다.
독립 한컴 원본/PDF와 rhwp 검사 산출물의 역할은 개별 review와 fixture README를 따른다.

| 경로 | 기존/신규 | bytes | SHA-256 |
| --- | --- | ---: | --- |
| `pdf/21_언어_기출_편집가능본-2022.pdf` | 기존 | 851275 | `f2d858d7974393661d91a658e6b384b951114ef52783379f426a963effd97b72` |
| `pdf/german-legislative-system-candidate-2020.pdf` | 기존 | 2381759 | `f4a4b40f1c9f6938b17612a334ec61392c5b81eb38d37900040b6455470b6ffd` |
| `pdf/hwp3-sample11-hwp-2020.pdf` | 기존 | 26386908 | `3f7bf779eb1928a48690386dadf14fb71b3b0935e93d3a40d523a53d9731b2d3` |
| `pdf/issue2083_hide_fill_page-hwpx-2020.pdf` | 기존 | 202151 | `00b37911e4a74410e5a6181a20a636b700bcaa950e885a12dc4d99bb91348c94` |
| `pdf/issue2470/36382471_masked-2022.pdf` | 기존 | 51697 | `814492b502a46e56e3a3be253e7beb386d752d2f5d47bfe2bfb9c646d41747cb` |
| `pdf/pr7078-before-original-p1.pdf` | 신규 | 245462 | `5de1436886ae56750a85ea5c5af9fdf416b412e7e3b3d32c10b4417abc8ff141` |
| `pdf/pr7078-capped-original-p1.pdf` | 신규 | 87770 | `02831046acb759ce819c563078996d2b8d4eeee47881bbae02e4408e8f5c767e` |
| `pdf/pr7078-maintainer-contrast-p1.pdf` | 신규 | 247053 | `f18a3a71c9d8fe56c65c0ff6dceb3625cd791ba6af62dbac9c2a2d65dbf51b94` |
| `pdf/pr7078-maintainer-original-p1.pdf` | 신규 | 246111 | `7f65b1ca24567f0682d5edbaf9dca2f21c6a9a734dcc41e77b28b853eafe3d82` |
| `pdf/pr7091-sample11-before-2020.pdf` | 신규 | 25612131 | `a5307a886421bb6235098d3d39258029f7b6980cbb31a3b4a70da8011ff24002` |
| `pdf/tac_object_host_line_height-2020.pdf` | 기존 | 719340 | `f90ea6915a842ac2266f4dd737b2829bbb3b72b927b1658577f6ca8c8b9b6051` |
| `samples/21_언어_기출_편집가능본.hwp` | 기존 | 435200 | `905454045ca2e236839a7cab59750678116d08af3db31dbf846819af355b8d15` |
| `samples/KTX.hwp` | 기존 | 163840 | `b6c1492152f53e8dd7d4bbbb4faca88866bb8458e9018c70c936cd469ea6fab3` |
| `samples/aift.hwp` | 기존 | 5724672 | `a3e94e613a7d3dad0ee11e2df8f9572a5b7c2d704602960c2075b5fd22df995c` |
| `samples/biz_plan.hwp` | 기존 | 33792 | `8b786d6824622afae2220b203beeef6e5592157e1896fea055ebc602817113c1` |
| `samples/exam_math.hwp` | 기존 | 770048 | `e40e3d675373c8efb3a844fc71f209600d3b0db987a04b3808b8e74a6b1671fe` |
| `samples/hwp3-sample11-hwp5.hwp` | 기존 | 587264 | `412956ee85313584dcb901e162a19f68b70a35d57cbe76c95e5e3a1c6f591b9d` |
| `samples/hwp3-sample11.hwp` | 기존 | 391507 | `51b743b2823a2df9b6fac243f56aebecedbbd02e2a8baad58ffc2e5a4e695f20` |
| `samples/issue2083_hide_fill_page.hwpx` | 기존 | 208382 | `7758c15c57b1ef14fda6e6d29409ae3425f344931f2901641af84a40ef413d2e` |
| `samples/issue2470/36382471_masked.hwpx` | 기존 | 16310 | `43572dad5e17395aa02d1b0000b736b8467278931086604776ef30393dd0f54b` |
| `samples/issue7062/tac_object_host_line_height.hwp` | 기존 | 286720 | `2cf764c89943a23eff17fb8ac5ccaa1958711216b15d5eb29a9a469b97d23abb` |
| `tests/fixtures/issue_4680/german-legislative-system-candidate.hwp` | 기존 | 369152 | `78d1bf6cc4480619d2044466c6f3742fc2bac7da4b2b6cb9abbdfc158ec63735` |
| `tests/fixtures/issue_4680/german-legislative-system-hancom-2020.hwp` | 기존 | 538112 | `84bcec55e53692a935dafec0ff509f878b278c9f833e59a3f4c9be1a444444b6` |
| `tests/fixtures/issue_4680/german-legislative-system.hwp` | 기존 | 545110 | `543d67cdb4d84b876949cef4f4ec7435b99716d6fa7feebde57b024c88d40559` |
| `tests/fixtures/issue_7077/high-contrast-step255.hwpx` | 신규 | 13567 | `38ba625555495335582d47a66896e987fc3722b08b8b6735bb5b2845108c727b` |
| `tests/fixtures/issue_7091/german-candidate.hwp` | 신규 | 370688 | `032b5fc93b1f45ce3f319326cfc16b074892f25d43b18a7ba05ed77795d59715` |
| `tests/fixtures/issue_7091/pr7091-sample11-candidate-hancom.hwpx` | 신규 | 518222 | `56f36098d70369514ba759b69da3af5fa30672f7e0b4f3f0b8fd73a73a79dd97` |
| `tests/fixtures/issue_7091/sample11-before.hwp` | 신규 | 291840 | `78ad6d2d3187c07971f780ce31866eec3bf91efee297d5842401c78dc9461c8e` |
| `tests/fixtures/issue_7091/sample11-candidate.hwp` | 신규 | 309760 | `49ae0c59518e307245ac897cd466ee127ba0c4e65f2792dfd9f64c840fa6cdfd` |

## 승인부터 후속 처리까지

1. 사용자 승인 뒤 현재 4건 범위로 임시 upstream head를 push하고 devel 대상 통합 PR #7102를 생성했다. owner reviewer 자동 지정 없음.
2. 정확한 code candidate의 Full CI·CodeQL·Render Diff·Adapter·Proptest와 Policy 성공을 확인했다. code 수정은 없었다.
3. code CI 녹색 뒤 개별 원 PR 4건 review·시각 asset 8개·오늘할일을 같은 PR의 trailing docs-only commit으로 반영했다.
   오늘할일은 최신 devel의 기존 내용을 보존한다. 별도 통합 번호 review나 문서 전용 PR을 만들지 않는다.
4. 최종 trailing head CI와 fast-pass 사유·mergeability를 재확인하고 merge 승인 범위에 따라 진행한다.
5. 실제 merge 뒤 4개 원 PR에 원 head·보정·통합 merge SHA·시각 증거를 comment하고 close한다.
   #7066/#7077은 구현된 해결 범위로 종료를 연결한다. #7086/#4680은 남은 축을 명시하고 open 유지, #6874는 재개방하지 않는다.
6. devel 동기화 및 post-merge duration refresh를 확인한다. 일반 검증 CI가 다시 실행되지 않는지 확인하며 issue-close 업무 자동화와 구분한다.
7. 종료한 이 작업 소유 branch/ref/target만 정리한다. contributor 원 branch·다른 worktree·공유 target은 보존한다.

## 통합 PR code CI 완료와 trailing 기록

통합 PR #7102의 code candidate는 `e670fb245a15f7e281401045065552f4f638041b`다. [통합 code CI](https://github.com/edwardkim/rhwp/actions/runs/34751409313)와 [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34751409265), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34751409180), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/34751409302), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34751409281)가 모두 성공했다.
CI의 Linux Archive A/B/C/D는 합계 **9,384 PASS / 46 skipped**이며 Lint·Frontend·Native Skia도 성공했다.
CI Impact Policy가 성공했고 trailing 작성 직전 `MERGEABLE / CLEAN`을 확인했다.
이 문서는 검증된 code candidate 위의 single-parent review-only commit에 포함했다. 최종 trailing head CI·fast-pass 및 실제 merge는 별도 확인 대상이다.
