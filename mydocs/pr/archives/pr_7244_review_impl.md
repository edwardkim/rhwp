---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7244부터 #7262까지 체리픽 통합 실행 기록

## 현재 판정과 범위

**최초 보류 8개 사유를 보정한 뒤 전체 회귀에서 발견한 #7261 추가 회귀도 focused 재검증을 마쳤다. 최종 통합 게이트 재실행 중.** 2026-09-18 조회 당시 planet6897의 open/non-draft PR 14개를 처리했다. reviewer jangster77을 먼저 지정했다. 주 작업공간에서 최신 upstream/devel `18a9fa85e`를 기반으로 `codex/planet-review-20260918`을 만들었다. 검증 전용 worktree는 `/Users/tsjang/rhwp-planet-verify-20260918`, 전용 target은 `target/planet-review-20260918`이다.

| PR | source head | 로컬 적용 commit | 판정 |
| --- | --- | --- | --- |
| [#7244](pr_7244_review.md) | `3945f5e1defd499f772a170c9da2073739803ec3` | `a7e30431c` | 메인터너 보정 후 수용 가능 |
| [#7245](pr_7245_review.md) | `ec9da112a380b9ae5cf2443a03521781974ab01b` | `e9ea3ca15` | 승인 |
| [#7246](pr_7246_review.md) | `aa4c9a291215e6707b9211e4d2d72ad7fda1dd44` | `1bc7c273d` | 승인 |
| [#7248](pr_7248_review.md) | `050394e006d4e838d10c4c2d0e05220ad7846b66` | `c9f407ff4` | 승인 |
| [#7249](pr_7249_review.md) | `7497b8e59d557e84d0d5cd1ee05e1a9ab77bba8b` | `fe2d59443` | 승인 |
| [#7250](pr_7250_review.md) | `008afddffcb48912fc5c15aa5a26a27ee16dc122` | `3805d3e70` | 승인 |
| [#7251](pr_7251_review.md) | `40fd97a62606c0525f50c37f6f5b18310d04d5c7` | `a63d7a351` | 개별 사유 해소·최종 게이트 대기 |
| [#7252](pr_7252_review.md) | `8fec62e01449b2b280d739665f31ee3aaa63b989` | `9327d8712` | 개별 사유 해소·최종 게이트 대기 |
| [#7253](pr_7253_review.md) | `af407d86707f82c570fac054a85f64a69d3df5d5` | `03fec50df` | 개별 사유 해소·최종 게이트 대기 |
| [#7255](pr_7255_review.md) | `3380ada6d92ad2bd818d9427ddcaf793fa54cc0e` | `075d11d16` | 승인 |
| [#7256](pr_7256_review.md) | `ee3a903d855c1fbc0eea90d8fda2331ea56a4ca8` | `3fac1a159` | 개별 사유 해소·최종 게이트 대기 |
| [#7259](pr_7259_review.md) | `91ec20dc30992961ae5e4f3dc1b3332bdf73fac7` | `2182dcadb` | 메인터너 보정 후 수용 가능 |
| [#7261](pr_7261_review.md) | `c242af317d074a9923c7971e2e004120b4088574` | `a0859738b` | 메인터너 보정 후 수용 가능 |
| [#7262](pr_7262_review.md) | `2afaa71a1cf8bb1a7ffe0f46f481ac8df89d4afe` | `f1da7e4e3`, `66015f64b` | 개별 사유 해소·최종 게이트 대기 |

## 적용 순서·conflict 처리

고유 commit 15개를 위 순서로 누적했다. #7256에 포함된 #7253 source `af407d867`은 중복 적용하지 않았다. #7262는 코드 `cfbab066c`와 증적 `2afaa71a1` 두 commit이다. 원 contributor history를 amend/rebase하지 않았다.

- #7246 off-canvas baseline: 정상 교체 86712의 옛 행을 복원하지 않고 새 진단 delta만 합쳤다. `3-11월 ... 위9미주사이8구분선아래7` 10→11을 반영했다.
- #7248 builder: #7245 공용 ID 할당과 열별 para_shape_id를 동시에 전달했다.
- #7252 body-overflow: 정상 86712의 옛 행은 복원하지 않았다. 외부 BinData 링크 fixture 15→6 강화는 적용했으나 통합 재검증에서 7이므로 보류했다.
- #7253: devel의 `shared_empty_frame` 분할을 보존하며 `hwp5_page_scale_cross_para_reset`을 결합했다.
- #7259: #7244와 중복된 SVG None/Zoom 코드는 하나로 유지했다. 통합 결과와 달라진 쪽 배경 주석을 바로잡았다.
- `30b9cca95`는 줄바꿈 형식·주석 정리와 한컴 PDF 보존이며 실행 의미 변경은 없다. 분석 → 수정·검증 → 결과보고 → 커밋 순서를 지켰다.

## 실행 검증 원장

검증 제품은 `66015f64b`, 현재 입력 확인 commit은 `30b9cca953848cb03dd16fcff6c7a548007ce623`다. Native/WASM 빌드 후 변경은 위 형식·주석뿐이다. 빌드 후 생성된 SVG를 다시 예전 제품으로 relabel하지 않았다. 일부 sweep의 git_head는 증적 PDF 커밋 전후로 다르므로 실제 바이너리 hash를 함께 고정한다.

| 검증 | 실제 결과 |
| --- | --- |
| Native CLI release-test build | PASS |
| focused 13모듈 | 41건 중 40 PASS / 1 FAIL (#7253) |
| 추가 #6800 / #1244 | 5/5 및 3/3 PASS |
| body-overflow | 16 partition 중 15 PASS / 1 FAIL (#7252) |
| off-canvas | 16/16 PASS |
| fresh WASM --no-opt | PASS (303.08초) |
| Studio tsc / render-backend | PASS / 64 PASS |
| fmt | 최초 충돌 줄바꿈 1곳 실패 → 보정 후 전체 fmt PASS |
| 전체 nextest·Clippy 3종·Native Skia 전체 | 미실행: focused/시각 blocker 확인 후 아직 확대하지 않음 |

전체 로컬 Rust 실행은 **81건 중 79 PASS / 2 FAIL**이다. 원 source의 green CI는 통합 제품 전체 검증으로 재사용하지 않는다. failed 검사를 삭제하거나 허용치를 임의로 확대하지 않았다.

재현 명령(검증 worktree, `DEVELOPER_DIR=/Library/Developer/CommandLineTools`):

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
CARGO_TARGET_DIR=/Users/tsjang/rhwp/target/planet-review-20260918 cargo build --locked --profile release-test --bin rhwp
node scripts/run-rust-test.mjs <review에 기재한 모듈> -- --cargo-profile release-test --target-dir /Users/tsjang/rhwp/target/planet-review-20260918
CARGO_TARGET_DIR=/Users/tsjang/rhwp/target/planet-review-20260918 scripts/wasm-pack-locked.sh --target web --out-dir /private/tmp/rhwp-planet-review-20260918/wasm-final --no-opt
```

## Visual Sweep·전쪽 후보 원장

13입력에 `fidelity_compare --text-only --export-all-svg --layout-ledger`를 실행했고, 영향·인접·정상 대조 페이지 **Native 28쪽 + fresh WASM 28쪽**을 webfont Chrome 경로로 캡처했다. 27/28쌍은 rhwp PNG bytes도 동일하다. 차이는 TOC 2쪽이며 Native/WASM 모두 기준 PDF의 사각형 리더와 다르다. 자동 flag는 양쪽 0이지만 이를 승인 증거로 대신하지 않았다. 각 PR에 사람 판독·ink-match·이미지를 연결했다.

정상 86712 HWP p26·28·29를 추가 대조했고 각 backend 64쪽을 유지한다. 기존 입력을 교체하거나 rename하지 않았다. `issue_7243_nested_fragment_padding` 2건도 통과했다.

- 정상 86712 p26: [native compare](../assets/pr7244_review/native_corrected_86712_compare_026.png) · [native overlay](../assets/pr7244_review/native_corrected_86712_overlay_026.png) · [native review](../assets/pr7244_review/native_corrected_86712_review_026.png) · [wasm compare](../assets/pr7244_review/wasm_corrected_86712_compare_026.png) · [wasm overlay](../assets/pr7244_review/wasm_corrected_86712_overlay_026.png) · [wasm review](../assets/pr7244_review/wasm_corrected_86712_review_026.png)
- 정상 86712 p28: [native compare](../assets/pr7244_review/native_corrected_86712_compare_028.png) · [native overlay](../assets/pr7244_review/native_corrected_86712_overlay_028.png) · [native review](../assets/pr7244_review/native_corrected_86712_review_028.png) · [wasm compare](../assets/pr7244_review/wasm_corrected_86712_compare_028.png) · [wasm overlay](../assets/pr7244_review/wasm_corrected_86712_overlay_028.png) · [wasm review](../assets/pr7244_review/wasm_corrected_86712_review_028.png)
- 정상 86712 p29: [native compare](../assets/pr7244_review/native_corrected_86712_compare_029.png) · [native overlay](../assets/pr7244_review/native_corrected_86712_overlay_029.png) · [native review](../assets/pr7244_review/native_corrected_86712_review_029.png) · [wasm compare](../assets/pr7244_review/wasm_corrected_86712_compare_029.png) · [wasm overlay](../assets/pr7244_review/wasm_corrected_86712_overlay_029.png) · [wasm review](../assets/pr7244_review/wasm_corrected_86712_review_029.png)

기존 devel 대조에 사용한 `/private/tmp/rhwp-pr7243-repair/rhwp-fix3`는 직전 작업의 최종 산출물이며 hash `23eedc9fe66c5fb58328c163ad9535c203107cfcdd104479478722fc2129177e`를 원 증적과 대조했다. 해당 보정 commit `43a0fbeec`와 최신 base `18a9fa85e`의 `src`, `Cargo.toml`, `Cargo.lock`, `rhwp-studio/src` diff는 0이다. 이번에 base를 새로 빌드했다고 주장하지 않는다. 이 제품과 통합 CLI에서 외부 BinData 링크 fixture의 overBottom > 2px는 모두 7건이다.

| 산출물 | SHA-256 |
| --- | --- |
| Native CLI | `9c561f66d882392acc1e5a2f5489c35178a7891528a8354a31436e937a617b3f` |
| WASM | `1c70aef5791b9b3dcaaad32ab61d5610752b50362690770bfe2ec71207493ed2` |
| WASM JS | `a7353a7603b7e07db2d33ff93fff6b213ea79e01da91c190cbb607e752c6b5a7` |

## 검증 입력 커밋 원장

아래는 최초 검토 시점 `30b9cca953848cb03dd16fcff6c7a548007ce623`의 입력 원장이다. #7251 기준 PDF는 이후 `a165bd4bf`에서 정상 재변환본으로 교체했으며 최신 해시는 개별 review에 기록했다. #7255의 새 경로 HWP는 기존 issue6924 HWP와 동일하여 중복을 제거하고 기존 한컴 PDF를 재사용했다. 아래 두 행은 최종 정본 경로·해시로 갱신했다. 다른 원본 bytes는 유지한다. 커밋된 원본·기존 PDF를 그대로 재사용한다. #7259 신규 PDF만 같은 commit에서 추가했으며 124,290 bytes, 1쪽이다. 저장 metadata `hancom-office-2010` / `8.5.8.1677`을 확인하여 engine 2020으로 start → status(succeeded, 23초) → download했다. PDF Creator `Hwp 2020 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`, 생성 KST 2026-09-18 14:54:18. PDF 1.4 버전 자체를 검토 제한으로 삼지 않는다.

| 입력/기준 경로 | SHA-256 |
| --- | --- |
| [pdf/1192000-202100017-policy-research-report-2020.pdf](../../../pdf/1192000-202100017-policy-research-report-2020.pdf) | `34c9173724fa5d2ef5e0b2a796b56c9b564d3f5e0c9224609dffd507921ada48` |
| [pdf/86712_regulatory_analysis-hwp-2024.pdf](../../../pdf/86712_regulatory_analysis-hwp-2024.pdf) | `bc1025b0607bbac01fea960997fa54430fd8dcc2604831b02940e7815cbcf84f` |
| [pdf/SO-SUEOP-hwp-2020.pdf](../../../pdf/SO-SUEOP-hwp-2020.pdf) | `12f7e011dcf1aaf90484a79c60b7ca853dcea860f135263ac41abf7c9ae613a7` |
| [pdf/hwpctl_API_v2.4-hwp-2020.pdf](../../../pdf/hwpctl_API_v2.4-hwp-2020.pdf) | `1d289727dd40ed35e48135bf16df06fe4cd080d967441ff464fb0e0b205fae74` |
| [pdf/issue7235/156467175_press_release_header_logo_p1-2020.pdf](../../../pdf/issue7235/156467175_press_release_header_logo_p1-2020.pdf) | `1e791edc71173ec436fbdca3480705a97beade1f496318212440955077763cc2` |
| [pdf/tac_object_host_line_height-2020.pdf](../../../pdf/tac_object_host_line_height-2020.pdf) | `f90ea6915a842ac2266f4dd737b2829bbb3b72b927b1658577f6ca8c8b9b6051` |
| [pdf/task2287/1342000_edu_curriculum_map-hwp-2020.pdf](../../../pdf/task2287/1342000_edu_curriculum_map-hwp-2020.pdf) | `bc16ee92640644307fd1ce92ab8ffe663088210caa7740bef620daaa0187e197` |
| [samples/86712_regulatory_analysis.hwp](../../../samples/86712_regulatory_analysis.hwp) | `ee82c7755617003cb972ba398da9cffadfed24ac0fa068eee1a5347da7658a88` |
| [samples/SO-SUEOP.hwp](../../../samples/SO-SUEOP.hwp) | `b5e410d4972b988240eb79f462c75ab3f92ce3415af6ed7f411906cc7498396c` |
| [samples/basic/request.hwp](../../../samples/basic/request.hwp) | `99e63b90f4aa3197029299ab087bc46225b3c27c0d07d424145b20879b45f12e` |
| [samples/hwp3-sample10.hwp](../../../samples/hwp3-sample10.hwp) | `d9ceb35d8abfb73e9afbe349bccb2a986cf552d66ae7f3f34dcfe4d9d385cb48` |
| [samples/hwpctl_API_v2.4.hwp](../../../samples/hwpctl_API_v2.4.hwp) | `d11dd1331083be4e8c989dfbd587777626b3d77686d3436c35a2c20da9494603` |
| [samples/issue1891/86712_regulatory_analysis.hwpx](../../../samples/issue1891/86712_regulatory_analysis.hwpx) | `0f4f055c74a3d39f70e417ca6c700880d9645a202798cd0e5c11f6e32c180a19` |
| [samples/issue1891_external_bindata_link.hwpx](../../../samples/issue1891_external_bindata_link.hwpx) | `ce9f7275b9c84e4f032c218b9b6f94cf53c24f6e91fba30be7fd55b17acee924` |
| [samples/issue6202/156483689-turmeric-industry-standardization.hwp](../../../samples/issue6202/156483689-turmeric-industry-standardization.hwp) | `bd24e80fda9e298ffb05dcdb64c22752a4ed78716b358076db26b2e721e41dc4` |
| [samples/issue6800/1192000-202100017-policy-research-report.hwp](../../../samples/issue6800/1192000-202100017-policy-research-report.hwp) | `fd0b95cb4239b08e2ab9130b6b697af56dda379f1c9029dbf5f5a0b97af5ceee` |
| [pdf/148751598-briefing-2020.pdf](../../../pdf/148751598-briefing-2020.pdf) | `98ce52ec0a6ed25ba73070b22c743cb72d129113456ae3019f417d1b53ee9dd3` |
| [samples/issue6924/148751598-briefing.hwp](../../../samples/issue6924/148751598-briefing.hwp) | `03c93b021e01652b1ca5ba3a4a301decf9da33484af7d088987327efcb59e610` |
| [samples/issue7062/tac_object_host_line_height.hwp](../../../samples/issue7062/tac_object_host_line_height.hwp) | `2cf764c89943a23eff17fb8ac5ccaa1958711216b15d5eb29a9a469b97d23abb` |
| [samples/issue7190/3011411_tac_picture_second_line.hwpx](../../../samples/issue7190/3011411_tac_picture_second_line.hwpx) | `02053a3e690a008ba1045d3ffc151d38fd25567eb518d5e713a6ebb3b721edbd` |
| [samples/issue7232/cell_align_justify-2020.pdf](../../../samples/issue7232/cell_align_justify-2020.pdf) | `825ebaf02e78e2eacd572638b767fd3a98e87536a303662ff7bd5bfc8367beeb` |
| [samples/issue7232/cell_align_justify.hwpx](../../../samples/issue7232/cell_align_justify.hwpx) | `aae71df3733cf3161f54ebb44e586e548a643215481f1dba3adf2f238a255e34` |
| [samples/issue7232/cell_align_left-2020.pdf](../../../samples/issue7232/cell_align_left-2020.pdf) | `43bab6e2141dffd9dd87c2931040f4a329b84c9b099f3decd2227deab130d381` |
| [samples/issue7232/cell_align_left.hwpx](../../../samples/issue7232/cell_align_left.hwpx) | `e3a1a27bf429b4464f50ce280c3ed154e878816d1823215ce64c7a26511e5ffd` |
| [samples/issue7235/156086935_none_image_fill-2020.pdf](../../../samples/issue7235/156086935_none_image_fill-2020.pdf) | `b5ee3e11064f87f8f2e11fadc66d8479dafab48b74abdae862e7a342e2ad4608` |
| [samples/issue7235/156086935_none_image_fill.hwp](../../../samples/issue7235/156086935_none_image_fill.hwp) | `b12aa445776bef38a003e8679865a9a0ed2f729ecef18099625c976efe64d6ae` |
| [samples/issue7235/156467175_press_release_header_logo_p1.hwp](../../../samples/issue7235/156467175_press_release_header_logo_p1.hwp) | `559b2760ccb678ae22484bc7de8bc61347fff17d3b980d9bec3f2ecec8e07d57` |
| [samples/lseg-05-tab.hwp](../../../samples/lseg-05-tab.hwp) | `0066108df04204324c0802414087563e7673e30f3d142dd014af14e318153d2c` |
| [samples/task2287/1342000_edu_curriculum_map.hwp](../../../samples/task2287/1342000_edu_curriculum_map.hwp) | `623b00d56beffc45d27c5bf23911bdc49d3a541ded8aecbb323d0716a2bc9f4e` |
| [tests/fixtures/issue6802/1400000-200600006_toc_leader_fill-2020.pdf](../../../tests/fixtures/issue6802/1400000-200600006_toc_leader_fill-2020.pdf) | `a55f4f55c89f70273996e30a3fb1885e3b3fcb86f14348824f01f713065ad6a6` |
| [tests/fixtures/issue6802/1400000-200600006_toc_leader_fill.hwp](../../../tests/fixtures/issue6802/1400000-200600006_toc_leader_fill.hwp) | `e9bc5e78b412876ad3a810d01ba3b5ca5077e921f13cd15dde755c8f537bbeae` |
| [tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame-2020.pdf](../../../tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame-2020.pdf) | `0dde093557a0a11cec2f01af94f3a8dcf5004c30d267f6f5ea3fa5ac65f9b4bf` |
| [tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame.hwp](../../../tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame.hwp) | `41f8f0349840a72476606a45843caf01b12013b5e536fca5f6214ff76872ceb3` |
| [tests/fixtures/issue7174/SO-SUEOP-hancom2020.hwp](../../../tests/fixtures/issue7174/SO-SUEOP-hancom2020.hwp) | `b5a60eec041d1efed3d9b0594adab8e51856fcbd7879f0e7d66c754349cddfd4` |
| [tests/fixtures/issue7174/SO-SUEOP-hancom2020.hwpx](../../../tests/fixtures/issue7174/SO-SUEOP-hancom2020.hwpx) | `cf1ac18087d4dc9dd3ad3bc4e97c265a23f76e4f1c52522ccb9dbb110bfc3585` |

전수 래칫은 이 commit의 추적 `samples` 코퍼스를 읽는다. 개별 baseline 경로·계수는 [body-overflow](../../../tests/fixtures/body_overflow_baseline.tsv)와 [off-canvas](../../../tests/fixtures/off_canvas_baseline.tsv), #7252 감소분 입력 hash는 [기여자 행별 근거](../../report/6976-body-overflow-ratchet-tighten/rows.md)를 참조한다. 합성 단위 문서는 테스트 코드에서 생성·소비하며 별도 복사본을 만들지 않는다.

## 남은 순서·rollback·후속 처리

1. 개별 review의 보류 8개를 원인별로 보정하고 해당 경계·PDF/overlay를 다시 확인한다. 실패한 테스트를 단순 삭제하거나 tolerance를 확대하지 않는다.
2. 최종 code head에서 필요한 전체 회귀·lint·Native Skia gate를 완료하고 결과를 보고한 뒤 보정 commit을 만든다.
3. 사용자 PR 요청 시 upstream의 임시 head branch로 push하고 devel 대상 통합 PR을 만든다. owner를 자동 reviewer로 지정하지 않는다. 원 PR 번호별 review와 오늘할일을 같은 통합 PR에 포함하며 통합 번호만을 위한 별도 review 문서를 만들지 않는다.
4. 최종 head CI를 확인하고 승인된 merge 후 실제 merge SHA와 증적 이미지가 보이는 한국어 comment를 원 PR·해당 이슈에 게시한다. 범위 밖 이슈는 닫지 않는다.
5. post_merge.md에 따라 duration refresh 확인·devel 동기화·소유 target/worktree/임시 branch 정리를 한다. post-merge 검증 CI를 새로 실행하지 않는다.

원격 push·PR 생성·comment·close·merge는 이번 검토에서 수행하지 않았다. 취소/제외 시 의존 순서 역순(#7256→#7253 등)과 공통 conflict 보정을 함께 검토하며 contributor branch를 지우지 않는다. 현재 검토 branch와 검증 target은 후속 보정을 위해 보존한다. scratch log/JSON/TSV는 커밋하지 않는다.

## 메인터너 보정 1 — #7252

최신 devel 동일 입력·동일 7개 넘침 좌표를 확인해 15→7 상한으로 정정했다. 보정 후 body-overflow 16/16 PASS(exit 0). [개별 근거](pr_7252_review.md)에 기록했다. 위 최초 실행 표는 수정 전 결과로 보존한다.

## 메인터너 보정 2 — #7244·#7259

source crop → contain destination → 최종 paint를 SVG/WebCanvas/Skia/CanvasKit에서 일치시켰다. 쪽 배경 None은 기존 stretch를 유지한다. crop 검사 2개 수정 전 FAIL → 보정 후 crop·쪽 배경 3 PASS, 실물 6 PASS, CanvasKit raster 4 PASS, Studio 64 PASS 및 tsc PASS. Native/fresh WASM 실물 3쪽씩 재캡처는 초기 PNG와 동일하며 실제 Skia/Canvas2D/CanvasKit 9쪽의 compare·overlay·review 27 PNG를 추가했다. [#7244](pr_7244_review.md)와 [#7259](pr_7259_review.md)에 hash·이미지·본문 글꼴 잔여 차이를 구분했다. 코드·소비 경로가 바뀐 나머지 PR과 통합 전체 게이트는 아직 남아 있다.

## #7261 메인터너 보정

저장 host의 위여백을 공통 점유 구간으로 만들고 예약·최종 표 원점이 함께 소비한다.
강화한 실물/비대칭 여백 2개는 수정 전 실패, 보정 후 통과했고 별도 분할 경로 3개도 통과했다.
Native/fresh WASM 27~29쪽 캡처는 서로 동일하며 Native 27·29쪽은 수정 전 bytes와 같다.
목표 표 윗변은 674.84→671.07px, PDF 괘선 671.27px다.
새 제품 hash·각 overlay·남은 범위는 [#7261 검토](pr_7261_review.md)에 기록했다.

## #7253·#7256 메인터너 보정

저장 프레임 원점·빈 슬롯·소유 줄의 가로 및 세로 위치를 함께 보정했다. 수정 전 실패를 확인한 좌표 검사 포함 8/8, 음성 대조 8/8 통과. Native/fresh WASM 3~6쪽과 대조 9~10쪽의 최신 증적·제품 해시는 [#7253](pr_7253_review.md#보정-후-visual-sweep)에 있다. 바깥 wrapper 외곽선·글꼴 잔여 차이와 최종 게이트 미실행은 유지한다.

## #7251 보정 판정 보완

내부 run 리더 반례와 정상 PDF 확보는 `a165bd4bf`에서 해소했다. PDF span은 제목 BatangChe / 점 Haansoft Batang이며 남은 U+2024 폭·글리프 fallback 차이를 실제 원인으로 구분했다. 유효한 한 줄·칸 내부 표시·비리더 보존의 제한된 범위에서 수용하고 #6802 전체는 OPEN 유지한다. 전체 글꼴 대체를 제품 수정이나 기본 환경 일치 증거로 쓰지 않는다.

## #7262 메인터너 보정

같은 행에서 재개하는 rowspan의 컷 소유권과 요구 높이 예약을 공유했다. 3.17px 잘림 반례 실패 후 5/5, #6981 8/8, #7243 2/2 통과. 32~34쪽 내용 보존과 다음 행, 정상 86712 26·28·29쪽을 Native/fresh WASM으로 재검증했다. 최신 증적·해시·남은 PDF 분할 차이는 [#7262](pr_7262_review.md#보정-후-visual-sweep)에 있다.


## 추가 통합 검증 — 저장 흐름 원점과 중복 입력

`227a31dfd` 전체 nextest는 10,071 PASS / 1 FAIL / 50 skipped였다. #7261 보정이 연속
빈 표의 흐름 원점과 paint 원점을 혼동하여 56345 마지막 표를 본문 아래 15.57px로 밀었다.
저장 사다리가 `height + outer_top + outer_bottom`과 정확히 같은 경우의 흐름 원점을
공통 span에 반영했다. 새 반례 수정 전 FAIL, 보정 후 3/3 PASS, 분할 표 대조 3/3 PASS,
본문 넘침 16/16 PASS. 기준값을 완화하지 않았다. 최종 전체 게이트는 다시 실행 중이다.

`37777c683`은 #7255 신규 경로와 동일한 기존 HWP를 재사용하고 중복 HWP/PDF·IR 원장 두 행을
제거한 커밋이다. #6925 1/1, #6924 대조 2/2, IR sweep 4/4(exit 0, 1 leaky 표지) 통과.
기존 PDF 기반 p1·p2 compare·overlay·review를 같은 커밋에 보존했다.

Studio 동일 제품의 타입 검사, 단위 1,760 PASS / 2 skipped, production build 통과.

추가 보정 후 전체 nextest 재실행은 **10,073 PASS / 50 skipped**였다. Native/fresh WASM
14입력 각30쪽을 재캡처했고 기존28쪽은 backend별 직전 보정 PNG와 동일하다.
Native Skia·lint 및 정책 검사는 순차로 이어 실행 중이다.
