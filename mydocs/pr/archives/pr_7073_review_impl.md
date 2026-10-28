---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-13
---

# PR #7073·#7074·#7075·#7082·#7083·#7087·#7088 체리픽 통합 검토

사용자 지시대로 `upstream/devel`을 먼저 동기화하고 `897c6a3d8d7559d314bf863c93bbe28c0d65e945` 위의
주 작업공간 branch `review/planet6897-20260913`에 원 PR의 고유 commit 9개를 `cherry-pick -x`했다.
충돌은 없었으며 stacked #7075/#7082와 #7074는 중복 적용하지 않았다. **#7089는 제외**했다. 작업 시작 뒤 14:08에 생성된 #7091은 이번 확정 7건과 분리해 후속 대상으로 남겼다.
원 PR reviewer는 jangster77로 지정했다. 통합 PR에 owner를 reviewer로 자동 지정하지 않는다.

## 출처와 메인터너 보정

| 원 PR | 원 head | 통합 commit | 검토 기록 |
| --- | --- | --- | --- |
| #7073 | `6952477ae783235377495caeaedd0a23f1a21373` | `b42a5e288, 4725bcdbb, 1aca2baf4` | [review](pr_7073_review.md) |
| #7074 | `416e894a8133608829788cb06f7cb757751f4b5a` | `e81628483` | [review](pr_7074_review.md) |
| #7075 | `2f41f46f1fd0a268a324e7187edafc732d739f27` | `dde60e9db` | [review](pr_7075_review.md) |
| #7082 | `85b8726dd175b25f5efb743c009e8da81e146d1b` | `cabdfb10f (#7075 중복 제외)` | [review](pr_7082_review.md) |
| #7083 | `4d9fd37b186332607640915825fa5d3973f4beb6` | `b76fd6cb6 (#7074 중복 제외)` | [review](pr_7083_review.md) |
| #7087 | `16ee2974ba7b057e39f2fd0ab572b54c18406e72` | `79d217eaa` | [review](pr_7087_review.md) |
| #7088 | `186953b0369aa2f962848fdadb70f85f1a60a6ef` | `75099f5ce (#7075·#7082 중복 제외)` | [review](pr_7088_review.md) |

- `4aa80b96a`: TAC leading이 개체 뒤에 속한다는 설명과 이미 커밋된 기준 PDF 문서를 정정했다.
- `2fabd879a`: 누락된 실물 HWP와 독립 한컴 HWP5/PDF·검사 대상 변환본을 보존했다.
- `568b210d9ec970735b2f97b67afc761f2eb4778b`: visual sweep에서 발견한 법무부 로고 17.9px 오배치를
  공통 TopAndBottom 저장 host 사다리 질의로 보정하고 동일 상단·비겹침 테스트를 추가했다.
- `036f74870`, `033c181cd`, `d3dfa17e0`: #6874 독립 한컴 HWPX·후보 HWP/HWPX·한컴 재저장 증거를
  보존하고 고정폭 공백 뒤 tail 텍스트까지 올바르게 추출한 결과를 기록했다.
- 최초 후보의 9,564 PASS는 추가 보정 전의 역사적 결과다. 아래 9,565개 재실행을 최종 근거로 사용했다.

## 최종 로컬 검증

전용 `target/pr-planet6897-20260913`를 최초부터 사용했다. shared target/debug·release·pr-review와
다른 작업의 worktree는 보존했다. Cargo 계열 실행은 순차 진행했다. source/test는 `568b210d9` 이후 동일하다.

- 전체 nextest: **Summary [ 457.270s] 9565 tests run: 9565 passed (7 slow), 46 skipped**, exit 0, command wall 458.9초.
- 추가 로고/stored/synthetic/TAC/OLE 집중 32 PASS, 통합 집중 42 PASS였다. 전체 결과의 중복 subset임을 구분한다. Native Skia lib 4,112 PASS / 13 ignored, placeholder 2 PASS, direct PDF 4 PASS도 별도 feature 경로에서 확인했다.
- suite prepare, cargo fmt --all, fmt --check 후 아래 검사를 모두 통과했다.

| 검사 | exit | wall seconds |
| --- | ---: | ---: |
| `clippy-native` | 0 | 26.54 |
| `clippy-wasm` | 0 | 22.64 |
| `workspace-build` | 0 | 28.43 |
| `clippy-all-targets` | 0 | 99.52 |
| `manifest-check` | 0 | 0.66 |
| `unit-tiers` | 0 | 1.41 |
| `focused` | 0 | 3.11 |
| `native-skia-lib` | 0 | 238.72 |
| `native-skia-placeholder` | 0 | 157.45 |
| `native-skia-pdf` | 0 | 9.02 |
| `wasm-build` | 0 | 291.14 |
| `wasm-parity` | 0 | 0.94 |

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all
cargo fmt --all -- --check
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-planet6897-20260913 --tests --no-fail-fast
cargo clippy --locked --target-dir target/pr-planet6897-20260913 -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-planet6897-20260913 -- -D warnings
cargo build --locked --workspace --target-dir target/pr-planet6897-20260913
cargo clippy --locked --workspace --all-targets --target-dir target/pr-planet6897-20260913 -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
node scripts/rust-unit-test-tiers.mjs --check
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-planet6897-20260913 --tests --no-fail-fast -E 'test(/issue_7047_textless|issue_7062_tac|issue_7079_tac|issue_7076_export_pdf|issue_4680_hwp3_odd|issue_4680_hwp3_shape_flip|issue_6874_hwp3|issue_6266_form|issue_2069|issue_6524|issue_6708/)'
cargo test --locked --profile release-test --target-dir target/pr-planet6897-20260913 --features native-skia --lib
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir target/pr-planet6897-20260913 --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir target/pr-planet6897-20260913 --features native-skia
env CARGO_TARGET_DIR=target/pr-planet6897-20260913 scripts/wasm-pack-locked.sh --target web --out-dir $EVIDENCE_DIR/wasm-pkg --no-opt
node scripts/svg_native_wasm_diff.mjs --rhwp $EVIDENCE_DIR/after-rhwp --pkg $EVIDENCE_DIR/wasm-pkg --keep-match tests/fixtures/issue_7047/housing-lease-standard-form.hwp samples/issue7062/tac_object_host_line_height.hwp tests/fixtures/issue_7079/vaccination-briefing-logo.hwp samples/issue6266/seizure_list_form_button.hwp tests/fixtures/issue_7076/ship-collision-analysis-form.hwp --out $EVIDENCE_DIR/wasm-parity
```

`$EVIDENCE_DIR`는 사용자가 정할 재실행 산출 디렉터리다. 임시 로그 디렉터리 자체가 merge 증거를 대신하지 않는다.
WASM은 Docker daemon에 연결할 수 없어 dev guide의 native `--no-opt` 진단 경로를 사용했다.
최적화 Docker 배포 build를 실행했다고 주장하지 않는다. WASM/native 비교는 아래 실물 5종 전 17쪽이다.
새 generated suite/manifest는 검증용 파생물이며 stage하지 않았다.

## 독립 시각·구조 검증

- 실물 5종, HWP/SVG/tree/PDF 3+10+1+1+2 = 17쪽을 전부 export/ledger 대조했다.
- 공식 visual_sweep, Chrome webfont, 96dpi, RGB threshold 32를 사용했다. 선택 11쪽의 자동 flagged는 0이나
  한컴 전체 일치 판정은 아니다. 각 review의 지표·잔여 차이와 실제 패널을 함께 판단한다.
- lease 3쪽·TAC 2쪽·logo 1쪽·압류목록 1쪽의 가시 변경을 확인했다. TAC 4·10쪽은 빈 TextLine bbox만
  변하고 SVG는 동일하다. ship의 기본 SVG는 의도적으로 Screen이므로 기본 PDF를 별도로 raster/text 비교했다.
- 새 TopAndBottom 보정은 초기 통합 대비 lease 3쪽만 변경했다. #7073 review의 한컴 이미지 rect 관측과
  새 회귀 테스트가 수용 기준이며 임의 좌표 clamp는 없다.
- OVR5(KTX, exam_math, 21_언어, aift, biz_plan) 142쪽·표 48개: 공식 page/size 비교와 추가 exact x/y/w/h/page
  비교에서 변화 0. 모든 페이지의 모든 개체·글자를 완전 비교한 것으로 확대하지 않는다.
- fidelity 도구의 단일 페이지 SVG 집계는 숫자 suffix 없는 파일을 0으로 세는 제한이 있다. 실제 manifest와
  visual sweep의 seizure/ship는 각각 SVG 1·tree 1·PDF 1이며 페이지 누락이 아니다.
- 신규 TAC fixture의 6쪽 off_canvas 1·overflow_cell 1은 기준 devel에서도 같은 bbox와 stderr를 재현했다.
  table bottom은 page보다 30.1467px 길고 cell stderr는 10.1px overflow다. 이 기존 문제를 고친 것으로 쓰지 않는다.
  [6쪽 before/after/한컴/overlay](../assets/pr7074_existing_overflow_review_p006.png)
- 독일 법령체계 before 저장본은 한컴 MCP 900초 timeout FAILED, 후보는 61.961초에 326쪽 PDF 출력 성공.
  pgct 4개와 도형 190개의 storage 비트를 독립 한컴 저장본과 대조했다. #4680 전체 해결·264쪽 일치 주장은 제외한다.

## 바이너리와 입력 provenance

한컴 변환은 engine 2020이며 실제 실행 제품은 `12.0.0.4605`(한컴오피스 2022), PDF Producer는
Hancom PDF 1.3.0.550이다. 파일명의 2020은 engine 선택명이다. 토큰·서버 URL은 기록하지 않는다.

| 바이너리 역할 | SHA-256 |
| --- | --- |
| 기준 devel 897c6a3d8 | `3e6cd8e21726b7318206fa3eb4de10942587a9f6d83f12a6330cc09567140574` |
| 보정 runtime source 568b210d9, visual/OVR | `b210c734c8dd9c6a9961b5ac762a87ab3a710448fdc67c2c6efa73192d82bd3a` |
| 전체 nextest build 산출 | `5a1f95f5d31081b1970475ecd04e208625ee91f0c7156db76553f87aca11cdb3` |

visual 바이너리는 `568b210d9`에 커밋한 runtime 패치를 커밋 직전에 빌드한 산출이다. 전체 nextest build 산출과 해시가 다르므로 17쪽 SVG/tree 동등성을 별도 export해 확인했다. manifest의 경로·build provenance 차이는 가시 산출과 구분했다.

직접 시각·구조·OVR 검증에 사용한 24개 입력/기준/검사 산출은 아래와 같이 실제 Git blob과 byte-identical이다.
Downloads에만 둔 입력은 없다. LFS pointer로 대체된 항목은 없다. source fixture와 독립 oracle, 후보 산출의 역할은 각 README 및 review에서 구분했다.

| 경로 | bytes | SHA-256 | 마지막 파일 commit |
| --- | ---: | --- | --- |
| [`pdf/german-legislative-system-candidate-2020.pdf`](../../../pdf/german-legislative-system-candidate-2020.pdf) | 2381759 | `f4a4b40f1c9f6938b17612a334ec61392c5b81eb38d37900040b6455470b6ffd` | `2fabd879a` |
| [`pdf/housing-lease-standard-form-2020.pdf`](../../../pdf/housing-lease-standard-form-2020.pdf) | 151946 | `f73f17d57b566d35f1641eef4ba1ab580aaf13725493b29e5ec88e7496e64e39` | `2fabd879a` |
| [`pdf/pr_planet6897_open_ci_20260828/by_saved_version/pr6281_issue6266_seizure_list_form_button-2020.pdf`](../../../pdf/pr_planet6897_open_ci_20260828/by_saved_version/pr6281_issue6266_seizure_list_form_button-2020.pdf) | 28988 | `6b04114179cef4091a38f4dfd44d419fe6ebe9ad93d5dff47c046f2a167d27a9` | `94bf0c7ca` |
| [`pdf/ship-collision-analysis-form-2020.pdf`](../../../pdf/ship-collision-analysis-form-2020.pdf) | 47352 | `fee10b0762d642a51207db8e6116246764c2b3fa52ccee9f2d2021bd0f368b9a` | `2fabd879a` |
| [`pdf/tac_object_host_line_height-2020.pdf`](../../../pdf/tac_object_host_line_height-2020.pdf) | 719340 | `f90ea6915a842ac2266f4dd737b2829bbb3b72b927b1658577f6ca8c8b9b6051` | `b76fd6cb6` |
| [`pdf/vaccination-briefing-logo-2020.pdf`](../../../pdf/vaccination-briefing-logo-2020.pdf) | 150614 | `464f85f6eefd6efd1d932d3d1976b18d1f580ee496974943414d57a9426394d9` | `2fabd879a` |
| [`samples/21_언어_기출_편집가능본.hwp`](../../../samples/21_언어_기출_편집가능본.hwp) | 435200 | `905454045ca2e236839a7cab59750678116d08af3db31dbf846819af355b8d15` | `d774adcbe` |
| [`samples/KTX.hwp`](../../../samples/KTX.hwp) | 163840 | `b6c1492152f53e8dd7d4bbbb4faca88866bb8458e9018c70c936cd469ea6fab3` | `dceb76f45` |
| [`samples/aift.hwp`](../../../samples/aift.hwp) | 5724672 | `a3e94e613a7d3dad0ee11e2df8f9572a5b7c2d704602960c2075b5fd22df995c` | `2ccb1bf85` |
| [`samples/biz_plan.hwp`](../../../samples/biz_plan.hwp) | 33792 | `8b786d6824622afae2220b203beeef6e5592157e1896fea055ebc602817113c1` | `6a68267a3` |
| [`samples/exam_math.hwp`](../../../samples/exam_math.hwp) | 770048 | `e40e3d675373c8efb3a844fc71f209600d3b0db987a04b3808b8e74a6b1671fe` | `91d9374f1` |
| [`samples/issue6266/seizure_list_form_button.hwp`](../../../samples/issue6266/seizure_list_form_button.hwp) | 25663 | `17d2984f1f15fd6455c5439266b5e6f444a0affae835e3d2739395a66b1b69c3` | `3c827cd07` |
| [`samples/issue7062/tac_object_host_line_height.hwp`](../../../samples/issue7062/tac_object_host_line_height.hwp) | 286720 | `2cf764c89943a23eff17fb8ac5ccaa1958711216b15d5eb29a9a469b97d23abb` | `e81628483` |
| [`tests/fixtures/issue_4680/german-legislative-system-before.hwp`](../../../tests/fixtures/issue_4680/german-legislative-system-before.hwp) | 369152 | `c3926c32c65f29968b7134e175d176b99f2bce37f7205057cf2df16d11e72679` | `2fabd879a` |
| [`tests/fixtures/issue_4680/german-legislative-system-candidate.hwp`](../../../tests/fixtures/issue_4680/german-legislative-system-candidate.hwp) | 369152 | `78d1bf6cc4480619d2044466c6f3742fc2bac7da4b2b6cb9abbdfc158ec63735` | `2fabd879a` |
| [`tests/fixtures/issue_4680/german-legislative-system-hancom-2020.hwp`](../../../tests/fixtures/issue_4680/german-legislative-system-hancom-2020.hwp) | 538112 | `84bcec55e53692a935dafec0ff509f878b278c9f833e59a3f4c9be1a444444b6` | `2fabd879a` |
| [`tests/fixtures/issue_4680/german-legislative-system.hwp`](../../../tests/fixtures/issue_4680/german-legislative-system.hwp) | 545110 | `543d67cdb4d84b876949cef4f4ec7435b99716d6fa7feebde57b024c88d40559` | `2fabd879a` |
| [`tests/fixtures/issue_6874/seizure-list-candidate-hancom-2020.hwpx`](../../../tests/fixtures/issue_6874/seizure-list-candidate-hancom-2020.hwpx) | 35583 | `be66f55370f4a9db1248f59a7ad26b36c223b1bdab6bfbf4ad02182e4bad5452` | `d3dfa17e0` |
| [`tests/fixtures/issue_6874/seizure-list-candidate.hwp`](../../../tests/fixtures/issue_6874/seizure-list-candidate.hwp) | 6144 | `7032583b0fe09510d7d1e0ae6bfbf6cbad04d153199a7507fe415af5c41475de` | `036f74870` |
| [`tests/fixtures/issue_6874/seizure-list-candidate.hwpx`](../../../tests/fixtures/issue_6874/seizure-list-candidate.hwpx) | 11103 | `69311f338d115cddbdca89faf497f1d0284ce9d97a566c2f3a0031acbd9e4c51` | `036f74870` |
| [`tests/fixtures/issue_6874/seizure-list-hancom-2020.hwpx`](../../../tests/fixtures/issue_6874/seizure-list-hancom-2020.hwpx) | 33269 | `28e95b27718f727c97ac0aca20abea5a253d4429cd92f78146205a25d74d43f9` | `036f74870` |
| [`tests/fixtures/issue_7047/housing-lease-standard-form.hwp`](../../../tests/fixtures/issue_7047/housing-lease-standard-form.hwp) | 100352 | `fe295c12c5f9bd9e7e73a540c7c7ecd044f18ed58ea9826c0de5d55ba2710439` | `b42a5e288` |
| [`tests/fixtures/issue_7076/ship-collision-analysis-form.hwp`](../../../tests/fixtures/issue_7076/ship-collision-analysis-form.hwp) | 54784 | `9c7952549166ec9b20efc8f32521fd3d1673f367599e6e82652b65c782e6bc44` | `79d217eaa` |
| [`tests/fixtures/issue_7079/vaccination-briefing-logo.hwp`](../../../tests/fixtures/issue_7079/vaccination-briefing-logo.hwp) | 113664 | `474e748d21402dc539cf9a1f65c761fe9144a8abb91627f9846810c46b2d8d05` | `2fabd879a` |

## 원격 통합·후속 처리 경계

이 문서는 로컬 검증에 따른 수용 판정이다. 최종 통합 head의 GitHub CI·MERGEABLE/CLEAN 확인과 실제 merge 전에는 원 PR을 close하지 않는다. 원 PR의 녹색 CI를 통합 후보의 Full CI로 대신하지 않는다.

최종 head의 Full CI, MERGEABLE/CLEAN, source head 및 최신 devel을 확인한 뒤 통합 PR을 merge한다.
그 뒤 원 PR 7건에 통합 PR/merge SHA·고정 review/PNG 링크·보정·잔여 범위를 comment하고 close한다.
#7047·#7062·#7079·#7076·#6874의 종료를 확인하고 #4680은 OPEN을 유지한다. #7089와 contributor fork branch는 보존한다.
post_merge.md에 따라 devel 동기화와 duration refresh를 확인한다. devel에는 검증 CI를 추가 실행하지 않는다.
이번 작업 소유 local/ref/upstream 임시 head와 전용 target만 active Cargo/Rust 부재 확인 뒤 정리한다.

## 통합 code candidate의 GitHub CI

[통합 PR #7093](https://github.com/edwardkim/rhwp/pull/7093)의 code candidate `d3dfa17e0f1cb454793b16c19628405e6f02b535`를 검증했다.
CI preflight는 `fast_pass=false reason=no-trailing-review-only-commits`였고 full classification을 선택했다.
Lint·Native Skia·Archive A~D build/test·Frontend package gate가 실제 실행되어 success였다.
CodeQL의 Python/JavaScript/Rust 분석과 Render Diff의 Canvas visual diff도 success였다.
비해당 gate의 skipped를 실제 테스트 실행으로 세지 않는다.

| Workflow | 실행 | 결과 |
| --- | --- | --- |
| CI Impact Policy Controller | [34741218780](https://github.com/edwardkim/rhwp/actions/runs/34741218780) | completed / success |
| Proptest roundtrip | [34741218882](https://github.com/edwardkim/rhwp/actions/runs/34741218882) | completed / success |
| Adapter inter-diff | [34741218860](https://github.com/edwardkim/rhwp/actions/runs/34741218860) | completed / success |
| Render Diff | [34741218798](https://github.com/edwardkim/rhwp/actions/runs/34741218798) | completed / success |
| CI | [34741218879](https://github.com/edwardkim/rhwp/actions/runs/34741218879) | completed / success |
| CodeQL | [34741218885](https://github.com/edwardkim/rhwp/actions/runs/34741218885) | completed / success |

이 code candidate와 같은 branch/PR의 녹색 결과를 확인한 뒤 review·asset·오늘할일만 single-parent trailing commit으로 추가한다. 그 새 head의 preflight·required aggregate·mergeability는 push 뒤 다시 확인한다.
