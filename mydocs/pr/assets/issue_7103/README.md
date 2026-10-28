---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review/visual_fixture_evidence.md
last_verified: 2026-09-14
---

# PR #7103 검증 입력과 시각 증적

최종 판정은 [개별 review](../../archives/pr_7103_review.md)를 따른다.
[기계 증적](validation.json)에 source/code SHA·입력 해시·종료값과 요약을 보존했다.

## 입력 동일성

아래 18개 실행 입력을 최종 code commit의 Git blob·SHA-256과 대조해 모두 일치했다.
원 HWP는 winchoose가 source `1c5fd9676`에 추가한 파일을 재사용했다. 한컴 PDF만 새로 보존했고
기존 tracked PDF 중 크기·SHA가 같은 파일은 없었다. 나머지는 기존 focused/OVR/Native Skia 자료의
원래 경로다. 이미 Git에 있는 자료를 이름 변경해 다시 추가하지 않았다.

| 경로 | bytes | SHA-256 |
| --- | ---: | --- |
| `tests/fixtures/issue_7103/ari-tutoring-application.hwp` | 67584 | `09596c115082f8fb873312152553741b15fe2ccfcdeecd9357195d62c94f2a43` |
| `pdf/ari-tutoring-application-2020.pdf` | 93633 | `37970ce6dffaff2d3bc119549753e591215404ca7e8f19857676b11dbebbc5a8` |
| `samples/hwp3-table-caption.hwp` | 26433 | `e8ac29656b1ca41f96b1dd9f6ef88be294c61165e2daf40f75873814291795e1` |
| `samples/issue6181/156562368_inline_tac_table_line_advance.hwpx` | 4562989 | `e5f92644f671eb8cfb0cfea9ebe9837c390b46b29329ec7858ed61d1ce423f96` |
| `samples/KTX.hwp` | 163840 | `b6c1492152f53e8dd7d4bbbb4faca88866bb8458e9018c70c936cd469ea6fab3` |
| `samples/exam_math.hwp` | 770048 | `e40e3d675373c8efb3a844fc71f209600d3b0db987a04b3808b8e74a6b1671fe` |
| `samples/21_언어_기출_편집가능본.hwp` | 435200 | `905454045ca2e236839a7cab59750678116d08af3db31dbf846819af355b8d15` |
| `samples/aift.hwp` | 5724672 | `a3e94e613a7d3dad0ee11e2df8f9572a5b7c2d704602960c2075b5fd22df995c` |
| `samples/biz_plan.hwp` | 33792 | `8b786d6824622afae2220b203beeef6e5592157e1896fea055ebc602817113c1` |
| `samples/hwpx/opengov/36384689_결재문서본문_화재발생종합보고서(제2026-298호).hwpx` | 13913 | `9de5b2b17aba9c51bfbab27f5e571780aa8e49f39f059e2d1ba8665da11df4cd` |
| `samples/issue2083_hide_fill_page.hwpx` | 208382 | `7758c15c57b1ef14fda6e6d29409ae3425f344931f2901641af84a40ef413d2e` |
| `samples/issue2470/36382471_masked.hwpx` | 16310 | `43572dad5e17395aa02d1b0000b736b8467278931086604776ef30393dd0f54b` |
| `samples/issue6542/156678235_mid_para_vpos_rewind.hwp` | 513536 | `bd2a04f4f969bdad693b21cb26fe61c16f36a14bc5b442c79a2c862777d2419c` |
| `samples/issue6754/156585314-ssagirang-barley.hwp` | 1424896 | `18350ed16867552fbaef12b0677531c0b37fcc8e585cf7d4ec0a953f812644cf` |
| `samples/issue6575/tac_picture_line_top.hwpx` | 22761 | `ae4c01001405de8deb2332cab5f469896a3b5ca12b18c1c28a48ff9719cadf85` |
| `samples/hwpx/opengov/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177).hwpx` | 37581 | `a8b5e2604e2bd35e3caf35c94d46999182240fa7d5747c79c15f00ee53fdb504` |
| `samples/re-03-latin-only-hancom.hwp` | 9216 | `52b8da3a7cd5625a5e9624aaa532be5faa29d9fc0fc96cc982109729d53cb09f` |
| `samples/issue6140/156462405_smart_expo.hwp` | 2129920 | `15fe20391ab296c69091553774026310e3e33113c11f45602e0d310810d7ddb9` |

## 한컴 기준

- 입력 `lastSavedWith.product=hancom-office-2022`, 저장 버전 `12.0.0.535`에 따라 engine 2020 선택.
- client `hwp-convert-mcp-2024-client-20260824-011002`, 실제 한컴 `12.0.0.4605`.
- job `54190c9b-5b88-4447-aa35-e6cd0274f8ea`; start → status(succeeded, terminal=true) → download 완료.
- 2026-09-13 13:46:07Z 시작, 13:46:32.911Z 완료; elapsed 25767ms, worker23620ms.
- direct DLL host, input preprocessing 없음, printMethod0, one-up PDF; 1페이지,93633bytes.
- server/download SHA가 위 PDF 해시와 일치. 인증 환경 파일·URL·토큰은 증적에 포함하지 않는다.
- 한컴 서버와 macOS Chrome의 실제 글꼴 공급이 다르므로 동일 glyph shape를 주장하지 않는다.

## 코드와 재현 경로

- source `1c5fd967674e238b3d4e42948c1abe5d4c279f6f` → cherry-pick `d92086eec6fa0f053e3fc50f7845aedc0202dd27`.
- devel 대조군 `cd2d9e8a430e2664181326c684b7491aa451cd31`; contributor의 유일한 production
  변경인 layout.rs를 해당 SHA로 복원해 만든 `baseline-rhwp`를 별도 보존했다.
- 메인터너 code head `ef489fd0290b96706f8fda265ed2f19b3265601a`; 전용 target `target/pr7103-review-20260913`.
- 원 PR CLI(`candidate-rhwp`)와 메인터너 최종 CLI를 구분했다. 기존 공유 `target/release/rhwp`를
  최신 빌드로 간주하지 않았다. 진단 후 source는 원복했고 최종 해시를 대조했다.

```sh
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr7103-review-20260913 --tests --test-threads 6 --no-fail-fast
RHWP_BIN=target/pr7103-review-20260913/debug/rhwp \
  venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 0 \
  --source tests/fixtures/issue_7103/ari-tutoring-application.hwp \
  --reference-pdf pdf/ari-tutoring-application-2020.pdf \
  --label pr7103-maintainer --reference-grade hancom-2020-12.0.0.4605 \
  --text-only --export-all-svg --layout-ledger \
  --out-dir /private/tmp/rhwp-7103-review-20260913/maintainer-fidelity
VISUAL_SWEEP_CHROME='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
  venv/bin/python scripts/visual_sweep.py \
  --rhwp-bin target/pr7103-review-20260913/debug/rhwp \
  --key pr7103-maintainer --hwp tests/fixtures/issue_7103/ari-tutoring-application.hwp \
  --pdf pdf/ari-tutoring-application-2020.pdf --pages 1 \
  --out /private/tmp/rhwp-7103-review-20260913/maintainer-sweep
```

OVR는 `tools/object_visual_regression.py`의 `ovr5` preset을 사용하고 `RHWP` 및 `git_head()`를
각 fresh CLI/해당 code SHA로 지정했다. devel에서 `--no-hwp --save-baseline`, 최종 후보에서
그 baseline을 `--baseline`으로 전달했다. 27/20/15/74/6페이지와 표9/9/3/27/0개가 각각 같고 회귀0건이다.
이 5문서를 새로 한컴 변환하거나 raster 비교한 것은 아니다.

## 최종 메인터너 보정 검증

- 최종 code head: `ef489fd0290b96706f8fda265ed2f19b3265601a` (원 source·체리픽과 구분). 상세 판정은 개별 review 참조.
- 초기 보정 `6bb907fa9`의 전체 검사에서 #6575의 그림 -8px와 #6140의 글자 겹침 6건을 검출했다.
  HWPX 구역 머리의 재기준화 축을 일반 역산에서 제외하고 저장 문단 시작 위치를 fit/paint에 함께
  보존해 고쳤다. 기존 기대값·baseline은 수정하지 않았고 최종 head에서 관련 검사와 전체를 다시 통과했다.
- 메인터너 최종 code/test의 로컬 bytes가 Git blob 및 검증 전 SHA-256과 일치했다.
- `validation.json`은 코드·입력 해시와 실행 종료값/요약, 비교 지표를 보존한다. 전체 transient 로그는
  `/private/tmp/rhwp-7103-review-20260913`에 있으며 인증 정보는 기록하지 않았다.
- 최종 focused는 #7103 4개와 #6078/#6181/#7049/#6754/#6972/#6575 및 text-overlap baseline이다.
  #6972는 모델로 만든 기존 회귀이며 실제 `56288` 파일을 실행 입력으로 세지 않는다.
- Native Skia: lib 및 `issue_2225_missing_picture_placeholder`, `render_p37_direct_pdf_export`.
- 최종 Native CLI와 fresh WASM 모듈의 SVG 및 Chrome에서 실행한 WASM SVG를 대조했다.
  Chrome 검증은 새 모듈을 직접 import한 경로이며 Studio 전체 앱 E2E는 아니다. Sweep은 CLI의
  `--font-style` local alias를 유지하고 직접 WASM 화면은 webfont projection을 공급하므로 raster는
  서로 다르다. 원 SVG의 byte 일치와 글꼴 공급에 따른 raster 차이를 구분했다.
  Docker daemon 부재로 호스트 `--no-opt` 진단 빌드를 사용했으며 최적화 배포 빌드를 실행했다고 하지 않는다.
- fidelity/Visual Sweep/OVR5의 최종 출력은 최종 CLI로 재확인했다. 한컴 PDF의 text-only 원장에만 남는
  c/d/e/f/g 각10개를 가시 문자 누락으로 판정하지 않았다.

```sh
CARGO_TARGET_DIR=target/pr7103-review-20260913 \
  scripts/wasm-pack-locked.sh --target web \
  --out-dir /private/tmp/rhwp-7103-review-20260913/pkg-maintainer --no-opt
node scripts/svg_native_wasm_diff.mjs \
  tests/fixtures/issue_7103/ari-tutoring-application.hwp \
  --rhwp target/pr7103-review-20260913/release-test/rhwp \
  --pkg /private/tmp/rhwp-7103-review-20260913/pkg-maintainer \
  --out /private/tmp/rhwp-7103-review-20260913/maintainer-svg-parity --keep-match
# 위 fidelity/visual_sweep 명령은 최종 fresh CLI 및 maintainer-* 출력 디렉터리로 실행한다.
```

최종 3-way는 **한컴 / 원 PR(clamp) / 메인터너 보정** 순서다. PDF raster 794×1122와 SVG raster
794×1123의 A4 반올림 차이는 하단 흰색 1행으로만 맞췄다. 내용 rescale·수평/수직 이동은 하지 않았다.
OVL도 같은 좌표계에서 R=한컴 gray, G=B=보정 gray로 만들었다. 제일 위 제목부터 하단 로고까지
직접 확인했으며 글꼴 차이는 표의 위치 오차와 별개로 기록했다.

| 버전 | flagged p.1 | pixel match | ink match |
| --- | ---: | ---: | ---: |
| devel | 1 | 78.78501% | 13.62244% |
| 원 PR | 0 | 87.38109% | 36.66524% |
| 메인터너 보정 | 0 | 92.42056% | 56.36344% |

- [최종 3-way](../pr7103_tac_review_p001_3way.png)
- [최종 OVL](../pr7103_tac_review_p001_ovl.png)
