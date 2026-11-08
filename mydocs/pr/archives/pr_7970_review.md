# PR #7970 리뷰 — PDF 입력 호환·요청 정책·paint 보존

## 최종 판정

**승인 — 작성자 self-review에서 검증한 code head의 공동 처리 범위를 충족했다.**
대상은 #7957–#7963·#7966·#7971의 입력 거절과 관련 출력 보정이다. 아래의 승인된
자원 전용 B1과 기존 parser 할당 증명·Type3 glyph·논리 문단 복원의 미완료 범위는 유지한다.
이는 GitHub approve event나 병합 완료를 뜻하지 않는다. 문서 trailing head의 실제 CI·
fast-pass·최신 base와의 병합 상태 확인 및 작업지시자의 병합 승인이 남았다.

## 접수 정보

| 항목 | 확인 결과 |
| --- | --- |
| PR / 작성자 / base | [#7970](https://github.com/edwardkim/rhwp/pull/7970) / `edwardkim` / `devel` |
| 검토 code head | `ba1de40d331769de664c90bfdcd34ca62464c716` |
| 실제 Native/Docker WASM runtime source | `3b7a3662b3fbabac2e8544ce364497bd4c454b19` |
| 고정 시각 비교 base | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` |
| 작성 시점 live base | `04f8c2bd1617f0dbec6f10543913391ff088c4a7` |
| 규모 / 역할 | 124 files, +6,673 / −404, 24 commits. 본인 PR이며 reviewer를 지정하지 않았다. |
| 작성 시점 상태 | OPEN, non-Draft, `mergeable=true / clean`; exact code head의 37 check에 실패·미완료 0건, combined status success |
| 관련 이슈 | Fixes #7957, #7958, #7959, #7960, #7961, #7962, #7963, #7966, #7971. 아직 종료하지 않았다. |

기본 경로는 `collaborator_self_merge`, 보조 경로는 `intake_and_review`, `local_validation`,
`visual_fixture_evidence`, `review_only_fast_pass`, 대형 PR의 `rework_and_exceptions`다.
[모 절차](../../manual/pr_review_workflow.md)와 [역할 선택표](../../manual/pr_review/README.md),
위 자식 문서를 확인했다. 대형 변경의 구현·시각 판독·CI·최종 병합 판단을 구분했다.

## 구현 주장과 실제 소비 경로

| 범위 | 코드·독립 기대와 반례 | 판정 |
| --- | --- | --- |
| 요청 정책 | [policy.rs](../../../src/parser/pdf/policy.rs) → [parse_pdf_with_policy](../../../src/parser/pdf/mod.rs) → backend `ImportBudget` → [WASM 진입점](../../../src/wasm_api/pdf_import.rs)·[Worker/session](../../../rhwp-shared/pdf/import-session.js). 64 MiB 복사 전 검사, 선택 1,024쪽, source traversal과 선택 쪽 비례 누적 작업을 구분했다. 16/64 MiB 경계·선택 범위·실패 뒤 새 요청 복구를 실제 제품 경로로 검사했다. | 충족 |
| JPEG 실제 예약 | [images.rs](../../../vendor/rhwp-pdf-import-backend/src/extractors/images.rs)의 `encoded_jpeg`를 preflight와 실제 storage 분기가 공유한다. JPEG 압축 보관에 존재하지 않는 8-byte plane을 요구하지 않으며 `to_png_bytes_with_budget`와 codec hooks의 header·pixel·실제 할당·PNG·누적 검사는 유지한다. RGB/Gray/CMYK, chained/inline, pixel cap 직전·정확·직후, codec sticky·raw 경로가 반례다. | 충족 |
| 중복 처리와 추출 | backend [document.rs](../../../vendor/rhwp-pdf-import-backend/src/document.rs)의 page stream/operator cache를 text/path/image가 소비한다. positional import는 사용하지 않는 전쪽 semantic artifact 분류를 수행하지 않는다. [stream_parameters_limited.rs](../../../vendor/rhwp-pdf-import-backend/src/stream_parameters_limited.rs)와 parser의 간접 Filter/DecodeParms/Length·predictor·trailing byte 보존은 기존 스트림 경계를 유지한다. | 충족 |
| 이미지 배치·clip | [extract.rs](../../../src/parser/pdf/extract.rs)는 unique resource가 아닌 실제 paint 횟수를 문서 요청 예산에 예약한다. [image_clip.rs](../../../vendor/rhwp-pdf-import-backend/src/extractors/image_clip.rs)의 유한 볼록 교차 → inverse CTM의 alpha 합성 → [lower_picture](../../../src/parser/pdf/lowering.rs) → IR PNG·배치 속성을 Native/Canvas와 저장·재열기가 소비한다. 반복 paint를 지우거나 곡선 clip을 bounding box로 대체하지 않는다. 직사각형 crop, affine Form BBox·기존 alpha·빈 교차·거절 경계를 구분했다. | 충족 |
| 색상·닫힌 선·그리기 순서 | [path_color.rs](../../../vendor/rhwp-pdf-import-backend/src/path_color.rs)의 page/Form·q/Q scope, `s = h S`, text/path/image의 계층적 source order → positional lowering의 공통 z-order를 확인했다. 가려져야 할 글자를 삭제하지 않는다. text merge는 비텍스트 paint 경계를 넘지 않고 `same_painted_span_style`의 동일 style만 공유한다. 서로 다른 글자색을 합쳐 URI 색을 잃는 실제 반례를 수정 전 FAIL / 수정 후 PASS로 확인했다. | 충족 |
| 글꼴·공통 줄 입력 | canonical font-rule의 PostScript/full-face·metric identity를 연결하고 Regular/Bold는 분리했다. [structured_lowering.rs](../../../src/parser/pdf/structured_lowering.rs)의 `span_text_with_spaces`를 positional/structured 경로가 줄 측정 전에 소비한다. 문서 ID나 특정 위치로 fallback·간격을 고정하지 않았다. [font 규칙 자료](../../../assets/font-rules/font_rule_registry_v2.json)와 공개 trace 회귀를 대조했다. | 충족 |
| backend·최종 출력 | Native Skia PNG / fresh Docker WASM Canvas를 각각 독립 PDF와 비교했다. SVG 실제 색상·paint 소속과 HWP/HWPX 저장·재열기 내용·그림 bytes·소유·순서도 정식 검사로 확인했다. 한 backend의 성공을 다른 경로의 실행 증거로 바꾸지 않았다. | 충족 |
| HWP pagination·LineSeg 재수용 | 이번 diff는 HWP의 page split/continuation 또는 저장 LineSeg 수용 조건을 바꾸지 않는다. PDF의 고정 페이지·source paint 소속을 내리는 변경이며 기존 HWP 컷/요구 높이 공통화 변경으로 확대하지 않는다. | 비해당 |

원인 계층·반례와 실제 이전 실행은 [stage8](../../working/task_m100_7957_stage8.md),
[stage9](../../working/task_m100_7957_stage9.md), [stage11](../../working/task_m100_7957_stage11.md),
[stage12](../../working/task_m100_7957_stage12.md), [stage13](../../working/task_m100_7957_stage13.md)에 연결했다.
자원 상수·호출 경로는 helper assertion만이 아니라 CLI·공개 API·실제 Worker를 통해 검증했다.

## 완료한 로컬 검증과 재사용

- `cargo nextest run --locked --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --tests --no-fail-fast`: **10,625 PASS / 45 skip**, exit 0.
- Native Skia lib **4,043 PASS / 12 ignored**, placeholder 2건, direct PDF 4건, 자원 계약 **123 PASS**.
  JPEG 보정의 새 경계 5건 중 실제 이전 버전에서 3건이 의도한 buffer 거절로 FAIL했다.
- workspace build, fmt, Native/WASM32/workspace all-target Clippy, manifest/unit-tier 등 실행한 필수 단계 11개가 모두 exit 0이었다.
- fresh Docker WASM 공개 API **115 assertion / 실제 Worker 7개 PASS**. 새 스캔의 직접/Worker 가져오기와 HWP/HWPX 저장 후 쪽별 그림 bytes 보존을 확인했다. 실제 이전 WASM은 같은 새 입력에서 buffer 오류로 FAIL했다.
- runtime source 이후 code head까지 runtime·font-rule·binding bytes는 같았다. 최종 실행 test source의 SHA도 commit과 대조했다. Node font-rule 49건과 Studio TypeScript·2,024 PASS/2 skip은 동일 파일·의존성 결과를 재사용했다.
- Native CLI SHA-256 `6fb1baf64195da8f28d934b825f6dc2004ba86aae540ebd47bd81933619a1408`,
  fresh Docker WASM SHA-256 `7dedc27fb948458999a4905f59b5487a2b6cea8361ff85473db59b4448d99c98`.
  Studio 7797의 실제 HTTP bytes·5쪽 표시·pageerror 0건까지 확인했다.

명령·종료 코드·test source 해시·producer와 전쪽 TSV는 ignored
`output/pr-review/issue7957/extraction-buffer-20261017/`의 `validation-results.json`,
`executed-test-source-hashes.json`, `native-source.json`, `wasm-producer.json`,
`formal-public-api-worker/`와 `candidate/scores/`에 있다. 이 검토는 이미 성공한 동일 코드의
전체 빌드·회귀를 다시 실행하지 않았다. 문서-only 후행 commit에는 source/test/fixture/workflow를 섞지 않는다.

## 시각 검증·입력 보존

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라
독립 Poppler PDF와 96 DPI, Print/compat 2022 Native Skia / fresh WASM Canvas를 대조했다.
공개 15문서·364쪽 및 추가 로컬 비공개 5쪽, 총 **369쪽이 양쪽 모두 raw 90% 이상**이며
독립 PDF 쪽수 일치·누락 0쪽이었다. 공개 최저는 Native **90.14458%**, WASM **90.16683%**,
추가 비공개 원본 최저는 **98.72216% / 98.72583%**다. 새 공개 생성 5쪽은 양쪽 전쪽100%다.
공통 지원 대조군에서 고정 base 대비0.01%p 이상 하락은 없었다.

기존 360쪽도 새 producer로 재출력했으며 Native/WASM 각각 이전 PNG와 360개 전부
byte-identical이었다. 기존 확대 판독은 이 동일성 근거와 함께 재사용했다. 새 10쪽은
각 독립 PDF의 그림 외곽·본문/괘선·모서리·하단 표식을 같은 영역에서 직접 판독했다.
연결선·가림·font/간격·검정 label/파란 URI의 직접 판독은 stage11에 기록했으며 점수로 대신하지 않았다.
새 clipping·paint·style 회귀는 공개 원본의 전쪽90%와 직접 판독을 확인한 후 추가했다.
기존 baseline/golden/래칫의 완화·실패 검사 제거·실물 px 핀 갱신은 없었다.

**승인된 예외는 두 가지뿐이다.** 신규 지원 입력은 고정 base가 기존 오류로 거절하므로
기준 TSV 부재를 기록하고 후보의 독립 PDF 전쪽90%·공통 지원 대조군 비교를 적용했다.
별도 생성 size 1쪽은 자원 정책·실패 후 복구로 한정해 **73.54167% / 78.19253%**를 그대로
남겼고 글꼴 차이는 [B1](../../plans/task_m100_7957_backlog.md)이다. 이 쪽을 시각 통과로
세거나 다른 문서에 예외를 넓히지 않았다. 자동 `candidateGate`를 임의의 PASS로 덮어쓰지 않았다.

공개 입력의 실제 파일·commit blob SHA-256을 대조했다. 아래 목록은 code head에 포함된
검증 입력이며 원본 PDF 자체가 독립 기준이다. 기존 커밋 파일은 중복하지 않았다.
추가 비공개 원본은 개인정보가 포함된 로컬 보조 검증으로만 보존하며 파일·해시·이미지·TSV를
공개하지 않는다. 공개 fixture는 같은 치수의 독립 생성 대조군이고 두 입력의 점수를 서로 대신하지 않는다.

| 용도 | 저장소 입력 | 바이트 | SHA-256 |
| --- | --- | ---: | --- |
| text-paint-order | [pdf/issue7966/text-paint-order-controls.pdf](../../../pdf/issue7966/text-paint-order-controls.pdf) | 1945 | `c6226b37743ab8482c928229a4320743a32a4aff8e3e5658f90beed09df8e922` |
| close-stroke | [pdf/issue7966/close-stroke-controls.pdf](../../../pdf/issue7966/close-stroke-controls.pdf) | 1335 | `02d0b20cf1a46305df97fbcc9ad1fa4ae383233e068f005ff2b82384b3bcf390` |
| affine-control | [pdf/issue7966/affine-clip-controls.pdf](../../../pdf/issue7966/affine-clip-controls.pdf) | 1676 | `a395c7899600b097d6cf28a7e352e0f3c16416108b54b2b5cdf95b577fb5c7b6` |
| image-count | [pdf/issue7966/exceeds-32.pdf](../../../pdf/issue7966/exceeds-32.pdf) | 1460215 | `cd8465adc8c96e7dd6dc2412b0d198ea0d54ae019824cad63443bc31139ea3d4` |
| image-control | [pdf/issue7966/image-placement-controls.pdf](../../../pdf/issue7966/image-placement-controls.pdf) | 7365 | `05b94e819617287c141b321d3c6c4dc1467fdad69a27d17a429b34b6febe7c02` |
| pages | [samples/issue7957/resource-limit.pdf](../../../samples/issue7957/resource-limit.pdf) | 1920186 | `b5dd80e561c055c028b5297c7b5ad04c1c1de6de6b246282635b5915680e11e7` |
| operators | [pdf/pr7704/6778-square-2020.pdf](../../../pdf/pr7704/6778-square-2020.pdf) | 930679 | `d4c1fe7a8a3c025b2be3648ba6e2ad1123eff3dde008983edadcef7e8f7b6656` |
| image | [samples/issue7959/bounded-import-path.pdf](../../../samples/issue7959/bounded-import-path.pdf) | 595334 | `0def50f6b391130c745754b44de320e14a3fd1c88a2dc35dce09e2b3c3bb418d` |
| paths | [samples/issue7960/path-budget.pdf](../../../samples/issue7960/path-budget.pdf) | 3331810 | `1372d5a2e4edf596d8d622026049fe6980130103a93a5c317b20716c6e0a9428` |
| parse | [samples/issue7961/extraction.pdf](../../../samples/issue7961/extraction.pdf) | 6130211 | `9c99058abf2d22571117ce58a123c32bbc3065756ff4e3d88d3ec5f034331bb3` |
| decoded | [samples/issue7963/decoded-bytes.pdf](../../../samples/issue7963/decoded-bytes.pdf) | 2194591 | `26acc6a382ec7c5732b5d2d89518d25531494722ccd128ce707288c82e87413a` |
| extraction-public | [pdf/issue7971/scan-jpeg-buffer-controls.pdf](../../../pdf/issue7971/scan-jpeg-buffer-controls.pdf) | 2107269 | `d056da61de9235369c99f7b21c8dd538a016455b7f49b646692e504083172cba` |
| nested | [pdf/issue5875/nested_table_text_caption-2020.pdf](../../../pdf/issue5875/nested_table_text_caption-2020.pdf) | 525733 | `6308671fc56b1bfd9e72660edd6edf574a4e16f75e84e0bf35399339d9ce3702` |
| colour | [pdf/issue7959/path-colour-controls.pdf](../../../pdf/issue7959/path-colour-controls.pdf) | 2219 | `14cc0df58fc30b6a735c4d1a6010d6f848f54ffd1a68260110eb5196fb1e6981` |
| words | [pdf/issue7958/positioned-word-gap.pdf](../../../pdf/issue7958/positioned-word-gap.pdf) | 3011 | `ec5cb353f18c1225f24dc73b7ecc6ea729bc1d17c445f24b810b56ffb90d82c5` |
| size | [samples/issue7962/valid-pdf-16777215-bytes.pdf](../../../samples/issue7962/valid-pdf-16777215-bytes.pdf) | 16777215 | `992b31ffa7cbb9280f3dc4f37b1ad6ccf7f96c91ce70e88c6993535555816968` |
| size-16777216 | [samples/issue7962/valid-pdf-16777216-bytes.pdf](../../../samples/issue7962/valid-pdf-16777216-bytes.pdf) | 16777216 | `0aeb2e75f7dbee75cabe11694cb69d61e0ab02970eee253379a6c1d6db66069a` |
| size-16777217 | [samples/issue7962/valid-pdf-16777217-bytes.pdf](../../../samples/issue7962/valid-pdf-16777217-bytes.pdf) | 16777217 | `e6f6835520f3434ac5ed2ccde5bd7f88143e1b15eb5c28af4c8d3fb33c30d90a` |

전체 공개 Native/fresh WASM TSV 원문(각365쪽, resource-only 1쪽 포함)과 base TSV는
[PR 본문](https://github.com/edwardkim/rhwp/pull/7970)에 그대로 보존했다. 대표 review/overlay
PNG26개는 [asset 원장](../assets/issue7957-20261017/README.md)에 있고 최종 PR head SHA로
고정해 본문에 실제 Markdown 이미지로 표시한다. 로그·TSV·중간 JSON은 커밋하지 않았다.

## 원격 CI·최신 base 확인

code head `ba1de40d331769de664c90bfdcd34ca62464c716`의 다음 실행은 모두 completed/success였다.

- [Full CI](https://github.com/edwardkim/rhwp/actions/runs/39013264008): lint·Native Skia·frontend package·A/B/C/D builder/worker·Build & Test 성공.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39013264048): Rust·JavaScript/TypeScript·Python·Actions Analyze와 GHAS check 성공. 해당 PR merge ref의 열린 alert 0건.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39013264077), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/39013264087), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/39013264005), [Controller](https://github.com/edwardkim/rhwp/actions/runs/39013260770) 성공.

일반 PR에서 독립 `WASM Build` job의 skip은 정상이며 로컬 fresh Docker WASM 검증과 구분한다.
PR REST의 오래된 `.base.sha`를 live devel로 쓰지 않았다. 실제 branch ref와 synthetic merge
`25e73a4f712a14972bb3372470990f6a27f5172c`의 부모 `[04f8c2bd…, ba1de40d…]`, tree
`cc2120ec2c80a09b8abcf023dd1dcedf6b4abac9`를 확인했고 로컬 `merge-tree`와 일치했다.
이는 최신 base 통합 CI 근거이며 **369쪽을 통합 tree에서 다시 Visual Sweep했다는 의미는 아니다**.
기존 고정 producer의 시각 결과와 최신 base 통합 CI를 구분한다.

## 남은 경계와 병합 후 계획

`parserAllocationBoundsProven=false`를 유지한다. 전체 parser/font/cache/codec의 모든 할당 증명,
Type3 glyph 프로그램 전체·논리 문단·이미지 조각 복원과 B1 Helvetica는 해결로 확대하지 않는다.
곡선·오목·복합 이미지 clip은 이번 유한 볼록 지원 범위 밖이다.

[실행 순서](pr_7970_review_impl.md)에 따라 trailing head의 실제 fast-pass·required checks와
최신 base/head를 확인하고 별도 병합 승인을 받는다. 아직 병합·이슈 종료·정리를 실행하지 않았다.
후속 처리가 승인되면 실제 merge SHA·CI·해결 범위와 남은 경계를 PR 및 관련 이슈에 기록한다.
대표 공개 PNG는 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue7957-20261017/…`
형식으로 고정하고 [시각 절차](../../manual/verification/visual_sweep_guide.md#github-merge-comment)를
연결해 `--body-file`로 게시·API 재조회한다. 본인 PR이므로 외부 contributor 감사 문구는 비해당이다.
