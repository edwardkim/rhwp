# 이전 중간 산출물의 결과와 출처

검증 과정의 JSON·원시 로그는 제출 자료에서 제외하고 `output/pr-review/planet6897-20261103/`에 유지한다.
아래 표는 제거한 제출용 중간 파일의 내용 해시와 실제 출처를 식별한다. 표의 과거 결과를 현재 head의 검증으로 취급하지 않는다.
원본·PDF·전쪽 TSV·대표 review/overlay는 그대로 보존했다. 글꼴 공급 계약인 `yeongwol-font-environment.json`은 실행 입력이므로 유지한다.

## 완료된 검증 결과

- `c183a1ac61`: [Rust 40개 명령](rust-validation-c183a1ac61.md), [실제 WASM/API 4개 실행](runtime-validation-c183a1ac61.md).
- 같은 제품: [Native 80입력·406쪽과 당시 base 대조 및 편람](current-c183a1ac61/native-validation.md), [fresh WASM 80입력·406쪽](current-c183a1ac61/wasm-validation.md).
- 같은 제품의 [보호 PR Native 17문서·1,770쪽·기준 2,153행](current-c183a1ac61/pr8389_8397/native-validation.md)과 [이전 리베이스 영향 3입력·7쪽 Native](current-c183a1ac61/rebase-scope-native-validation.md).
- 이전 producer의 [보호 PR Native/fresh WASM 전체 대조](pr8389_8397/README.md)와 [이전 리베이스 집중 대조](rebase_scope_22947f6a90/README.md)는 원 source 표시를 유지한다.
- [실제 직접 판독 기록](direct-reading-current.md), [그림 바이트 대조표](direct-reading-image-sha256.md), [주석만 변경한 회귀 함수 보존](test-header-only-audit.md).

## 개요 예약 폭과 표 이어받기 보정

### native — c183 제품

13개 입력·PDF 조합, 73쪽, 최저 raw 95.97856%, 이전 표시값 하락 0쪽.

| 입력 | 독립 PDF | 쪽수 | 최저 raw |
| --- | --- | ---: | ---: |
| [levels.hwpx](<../issue_8425/probe/levels.hwpx>) | [levels-2024.pdf](<../../../../pdf/issue-8425/levels-2024.pdf>) | 1 | 100.0% |
| [levels.hwpx](<../issue_8425/probe/levels.hwpx>) | [levels-2020.pdf](<../../../../pdf/issue-8425/levels-2020.pdf>) | 1 | 100.0% |
| [issue3307_outline_number.hwpx](<../../../../samples/task3307/issue3307_outline_number.hwpx>) | [issue3307_outline_number-2020.pdf](<../../../../pdf/task3307/issue3307_outline_number-2020.pdf>) | 9 | 99.99237% |
| [list_marker_head_synthetic.hwpx](<../../../../samples/issue7418/list_marker_head_synthetic.hwpx>) | [list_marker_head_synthetic-2024.pdf](<../../../../pdf/issue7418/list_marker_head_synthetic-2024.pdf>) | 4 | 100.0% |
| [business_plan.hwpx](<../issue_8424/inputs/after/business_plan.hwpx>) | [business_plan-2024.pdf](<../../../../pdf/issue-8424/business_plan-2024.pdf>) | 3 | 100.0% |
| [cellwrap.hwpx](<../issue_8424/inputs/after/cellwrap.hwpx>) | [cellwrap-2024.pdf](<../../../../pdf/issue-8424/cellwrap-2024.pdf>) | 1 | 100.0% |
| [scaffold_table_footnote.hwpx](<../../../../samples/issue8520/scaffold_table_footnote.hwpx>) | [scaffold_table_footnote-2024.pdf](<../../../../pdf/issue8520/scaffold_table_footnote-2024.pdf>) | 2 | 99.1045% |
| [gangseo_exempt_certificates.hwp](<../../../../samples/issue8532/gangseo_exempt_certificates.hwp>) | [issue8532_gangseo_exempt_certificates-2020.pdf](<../../../../pdf/issue8532_gangseo_exempt_certificates-2020.pdf>) | 6 | 99.84567% |
| [scaffold_outline_a.hwpx](<../scaffold_outline_numbering/scaffold_outline_a.hwpx>) | [scaffold_outline_a-2024.pdf](<../../../../pdf/issue8540/scaffold_outline_a-2024.pdf>) | 1 | 100.0% |
| [scaffold_outline_b.hwpx](<../scaffold_outline_numbering/scaffold_outline_b.hwpx>) | [scaffold_outline_b-2024.pdf](<../../../../pdf/issue8540/scaffold_outline_b-2024.pdf>) | 1 | 100.0% |
| [deferred_takeplace_fill_ahead.hwp](<../../../../samples/task1753/deferred_takeplace_fill_ahead.hwp>) | [deferred_takeplace_fill_ahead-2024.pdf](<../../../../samples/task1753/deferred_takeplace_fill_ahead-2024.pdf>) | 21 | 95.97856% |
| [deferred_takeplace_fill_ahead.hwpx](<../../../../samples/task1753/deferred_takeplace_fill_ahead.hwpx>) | [deferred_takeplace_fill_ahead-2024.pdf](<../../../../samples/task1753/deferred_takeplace_fill_ahead-2024.pdf>) | 21 | 95.97856% |
| [condense_numbered_synthetic.hwpx](<../../../../samples/issue7418/condense_numbered_synthetic.hwpx>) | [condense_numbered_synthetic-2024.pdf](<../../../../pdf/issue7418/condense_numbered_synthetic-2024.pdf>) | 2 | 100.0% |

### wasm — c183 제품

13개 입력·PDF 조합, 73쪽, 최저 raw 96.1185%, 이전 표시값 하락 0쪽.

| 입력 | 독립 PDF | 쪽수 | 최저 raw |
| --- | --- | ---: | ---: |
| [levels.hwpx](<../issue_8425/probe/levels.hwpx>) | [levels-2024.pdf](<../../../../pdf/issue-8425/levels-2024.pdf>) | 1 | 100.0% |
| [levels.hwpx](<../issue_8425/probe/levels.hwpx>) | [levels-2020.pdf](<../../../../pdf/issue-8425/levels-2020.pdf>) | 1 | 100.0% |
| [issue3307_outline_number.hwpx](<../../../../samples/task3307/issue3307_outline_number.hwpx>) | [issue3307_outline_number-2020.pdf](<../../../../pdf/task3307/issue3307_outline_number-2020.pdf>) | 9 | 99.93728% |
| [list_marker_head_synthetic.hwpx](<../../../../samples/issue7418/list_marker_head_synthetic.hwpx>) | [list_marker_head_synthetic-2024.pdf](<../../../../pdf/issue7418/list_marker_head_synthetic-2024.pdf>) | 4 | 100.0% |
| [business_plan.hwpx](<../issue_8424/inputs/after/business_plan.hwpx>) | [business_plan-2024.pdf](<../../../../pdf/issue-8424/business_plan-2024.pdf>) | 3 | 100.0% |
| [cellwrap.hwpx](<../issue_8424/inputs/after/cellwrap.hwpx>) | [cellwrap-2024.pdf](<../../../../pdf/issue-8424/cellwrap-2024.pdf>) | 1 | 100.0% |
| [scaffold_table_footnote.hwpx](<../../../../samples/issue8520/scaffold_table_footnote.hwpx>) | [scaffold_table_footnote-2024.pdf](<../../../../pdf/issue8520/scaffold_table_footnote-2024.pdf>) | 2 | 99.14798% |
| [gangseo_exempt_certificates.hwp](<../../../../samples/issue8532/gangseo_exempt_certificates.hwp>) | [issue8532_gangseo_exempt_certificates-2020.pdf](<../../../../pdf/issue8532_gangseo_exempt_certificates-2020.pdf>) | 6 | 99.87175% |
| [scaffold_outline_a.hwpx](<../scaffold_outline_numbering/scaffold_outline_a.hwpx>) | [scaffold_outline_a-2024.pdf](<../../../../pdf/issue8540/scaffold_outline_a-2024.pdf>) | 1 | 100.0% |
| [scaffold_outline_b.hwpx](<../scaffold_outline_numbering/scaffold_outline_b.hwpx>) | [scaffold_outline_b-2024.pdf](<../../../../pdf/issue8540/scaffold_outline_b-2024.pdf>) | 1 | 100.0% |
| [deferred_takeplace_fill_ahead.hwp](<../../../../samples/task1753/deferred_takeplace_fill_ahead.hwp>) | [deferred_takeplace_fill_ahead-2024.pdf](<../../../../samples/task1753/deferred_takeplace_fill_ahead-2024.pdf>) | 21 | 96.1185% |
| [deferred_takeplace_fill_ahead.hwpx](<../../../../samples/task1753/deferred_takeplace_fill_ahead.hwpx>) | [deferred_takeplace_fill_ahead-2024.pdf](<../../../../samples/task1753/deferred_takeplace_fill_ahead-2024.pdf>) | 21 | 96.1185% |
| [condense_numbered_synthetic.hwpx](<../../../../samples/issue7418/condense_numbered_synthetic.hwpx>) | [condense_numbered_synthetic-2024.pdf](<../../../../pdf/issue7418/condense_numbered_synthetic-2024.pdf>) | 2 | 100.0% |

위 보정의 공식 전쪽 TSV는 c183 Native/WASM 검증 목록에서 같은 입력으로 찾을 수 있다.
수정 전후 그림과 핵심 관계는 [PR #8433 리뷰](../../archives/pr_8433_review.md), [PR #8547 리뷰](../../archives/pr_8547_review.md)의 해당 입력·쪽에 연결한다.
반례 분석의 원시 항목·측정 중간값은 아래 해시의 원본을 로컬 output에 보존하며 공식 전체 통과의 근거로 단독 사용하지 않는다.

## 중간 산출물 식별표

| 이전 제출 경로 | SHA-256 | 기록된 출처·범위 |
| --- | --- | --- |
| `base-comparison-capture-4b149b6cdf.json` | `2e74a4d8e64a935b0c66c7430ee6ee5cd5c3d28930ddd815c8eeee12fadd1372` | recorded_at=2026-11-04T20:14:29.747890+00:00 |
| `base-native-provenance.json` | `1a687bda89fc7e76620c422d41b6d494af382e76261e7a5a00f62e879195ea7c` | recorded_at=2026-11-04T16:00:00.194124+00:00; pages=406 |
| `base-native-strict-audit.json` | `d30fcb8d6b87fb69d3f4f1d23fea77b000c2186c2c8491a4b7cdace411cac7ad` | recorded_at=2026-11-04T20:12:25.280760+00:00; backend=native; pages=406 |
| `base-wasm-strict-audit.json` | `9b036052b14e1f1ff659ea131e5875f0621d60661c48049fc090e78ea9c584f2` | recorded_at=2026-11-04T20:12:25.355463+00:00; backend=wasm; pages=406 |
| `claim-source-bindings-native.json` | `cf1ab711d084555c4b9c29965ff2ddf80e0442120734c75cabe4d82c4e353b45` | recorded_at=2026-11-04T19:02:47.485156+00:00; backend=native |
| `claim-source-bindings-wasm.json` | `e8f515a06d7150bf0035625b2ea5762099a799c73c5efc137e45489bdff02b63` | recorded_at=2026-11-04T19:02:47.491021+00:00; backend=wasm |
| `claim-table-score-audit.json` | `4b42de58f96ca2e8380c0aece59ee67437f4d15bef63173d07b49d4e02d5f6a6` | recorded_at=2026-11-04T19:43:49.212431+00:00 |
| `current-c183a1ac61/handbook-audit.json` | `80871a6ed6b8065253b748cb530db2ee7ff43142a2e56af8124bced44bf09b7f` | recorded_at=2026-11-04T23:05:12.821761+00:00; pages=383; status=passed |
| `current-c183a1ac61/handbook-required-handbook-8153-gate.json` | `02471702957b1ff9301dd49499c1f017138476c63426151cf4443a92df3ad3f4` | backend=native-skia; pages=383; status=passed |
| `current-c183a1ac61/handbook-silhouette_manifest.json` | `0804d3774fc379f492eaae83a3a09b11ea963913b513d28359043c71697d3846` | 중간 대응·진단 원장; 위 완료 기록과 개별 리뷰 참조 |
| `current-c183a1ac61/native-audit.json` | `64991e63aabcf55178c6be311358666325b10a7a6d63b629a19c8bfa0271311f` | recorded_at=2026-11-04T22:13:33.543767+00:00; backend=native; cases=80; pages=406; minimum=92.45232 |
| `current-c183a1ac61/native-base-comparison.json` | `30e7569ad3813f27efbeb35d2f7b72bf8d37f7015fe2bec77bb332fa9d96eac3` | recorded_at=2026-11-04T22:28:45.041130+00:00; backend=native; pages=406 |
| `current-c183a1ac61/pr8389_8397/native-audit.json` | `46f40af91cacee48ab0da9b92a0ff6b322302fdb00fcdb33a214676b88c2b16d` | recorded_at=2026-11-04T23:39:31.692412+00:00; code_commit=c183a1ac61c2aa4188989fd93032cae68d3538c1; backend=native |
| `current-c183a1ac61/rebase-scope-native-audit.json` | `aa95d01d52027088f7e86fad531cb9fb437f9d55de9c9c8341d9f2e126e20bf0` | recorded_at=2026-11-04T23:39:32.271676+00:00; cases=3 |
| `direct-reading-current.json` | `d1cf4295c85bed8a29dcae497b6ce6c3f178f3bf4e062440e62d6973eea3c81d` | recorded_at=2026-11-05T00:21:57.128235+00:00 |
| `full-nextest-c183a1ac61.json` | `1a03e8ea6eb8fb929fe91dd35593b6207b0ca67c97288c6e87ceb0d10172fa82` | recorded_at=2026-11-04T22:26:10.518791+00:00; passed=12563; exit_code=0 |
| `inherited-8548-wasm-preservation.json` | `d8efdce308885449a12b75a812ab5f8eb4d64d341820e308ebbeedc76811f83a` | recorded_at=2026-11-04T18:44:06.548736+00:00; cases=32; pages=72 |
| `latest-product-validation-a7e5ce7112.json` | `77b2ade374473043cfd8f13f0755626f31e5d784b2047f380a9d9d5835a6dc85` | recorded_at=2026-11-04T18:31:36.749179+00:00; base_sha=22947f6a90c998e95fa51238d5b907a0ebd37162 |
| `manual-review-current-assets.json` | `d7eed7b8518c80e459395568e0a916b27b11d0705b96548311fb07f6e1e95402` | recorded_at=2026-11-04T19:10:48.403069+00:00 |
| `marker-slot-focused-regression-c183a1ac61.json` | `595f7f0cfac331f6379b630a6aafb1e72e109488c830fdfa80fb9bdbb91548d6` | recorded_at=2026-11-04T21:33:22.242556+00:00; code_commit=c183a1ac61c2aa4188989fd93032cae68d3538c1 |
| `pr-body-preparation-audit.json` | `53a359bc7ba2a2d9367cb8c4514eacd8bb62deb7f0dbb161132977b2bb613b78` | 중간 대응·진단 원장; 위 완료 기록과 개별 리뷰 참조 |
| `pr8389_8397/native-provenance.json` | `5a0453f5cf90a0b10d2f7c1372007eb88b6e6c2531f39a01d160d46649f680c5` | recorded_at=2026-11-04T15:53:00.083280+00:00 |
| `pr8389_8397/wasm-provenance.json` | `d0a6c860c46bd55ddda2d6206b11948c69da1a3759829e63152b1193b3b04355` | recorded_at=2026-11-04T17:09:52.680477+00:00 |
| `pr_8433/marker-slot-diagnostic.json` | `07a986fd934d09038eb28069add109849ecdcbe3c1d95ddb6650433f7560c3dc` | recorded_at=2026-11-04T19:56:12.560877+00:00; status=독립 출력 관측과 사양 대조; 구현 미수정; 고정 회귀 기대값 아님 |
| `pr_8433/reference-width-c183a1ac61/control-provenance.json` | `3e24f30eec99c3779e198b0d2917d3f374b8db36d17fd43060bbb20981e50e14` | recorded_at=2026-11-04T21:25:20.576121+00:00; code_commit=c183a1ac61c2aa4188989fd93032cae68d3538c1 |
| `pr_8433/reference-width-c183a1ac61/native-product.json` | `79f9489bfed42dda741a25c92e0ff36a438353033edf618a2712bdff59a863a8` | code_commit=c183a1ac61c2aa4188989fd93032cae68d3538c1 |
| `pr_8433/reference-width-c183a1ac61/native-provenance.json` | `434d02ff884ca30b9ec8fcfbdf0f47e72ad01dd6a11a105b8bec1f3549c0838f` | recorded_at=2026-11-04T20:52:36.618972+00:00; code_commit=c183a1ac61c2aa4188989fd93032cae68d3538c1 |
| `pr_8433/reference-width-c183a1ac61/native-selected-audit.json` | `76c6d3745d8cb6ba02e598432f51fe3267d092e83d9825388b3b5d8cf84a8839` | recorded_at=2026-11-04T21:04:31.673072+00:00; code_commit=c183a1ac61c2aa4188989fd93032cae68d3538c1; backend=native; cases=13; pages=73; minimum=95.97856; compared_pages=73; strict_display_drops=0 |
| `pr_8433/reference-width-c183a1ac61/number-slot-before-after.json` | `8ceaf023ca41af0983365321950045971708ce60fc719503160ab408f7d51cc5` | recorded_at=2026-11-04T21:12:03.271801+00:00; status=같은 참조 형식 관계가 기존 실제 출력에서 7개 실패, 보정 출력에서 7개 통과한 JSON 진단; 이전 Rust 함수 실행을 뜻하지 않음 |
| `pr_8433/reference-width-c183a1ac61/wasm-provenance.json` | `95c9e3fabd602c5d6496621203fac912cdf8763073862ff2a2906913ca1bbee7` | recorded_at=2026-11-04T21:09:19.052792+00:00; code_commit=c183a1ac61c2aa4188989fd93032cae68d3538c1 |
| `pr_8433/reference-width-c183a1ac61/wasm-selected-audit.json` | `b09428bfa6333f136aa8678dd1409550c26cca341d5a921409d574d7819914bf` | recorded_at=2026-11-04T21:23:19.682978+00:00; code_commit=c183a1ac61c2aa4188989fd93032cae68d3538c1; backend=wasm; cases=13; pages=73; minimum=96.1185; compared_pages=73; strict_display_drops=0 |
| `pr_8476/pr8476-claim-p007-pushed_none-native-source.json` | `e3f5bc9b3a4694b9a2948d5cd29b482c2c426528c31617fec50f0fe02e2dc288` | backend=native |
| `pr_8476/pr8476-claim-p007-pushed_none-wasm-source.json` | `fc1efa6924918e4e48a63c4f984710238aac98684a18b2311b050aff4d1463a1` | backend=wasm |
| `pr_8476/pr8476-claim-p008-pushed_none-native-source.json` | `95ae574bdacf0cfcf613dbdaccdbd558a78ac3c485e681077e6652431a01dab3` | backend=native |
| `pr_8476/pr8476-claim-p008-pushed_none-wasm-source.json` | `748d0e80a6fc4b23c6a4501c9d05985b7393bd49c2833c768b071acd72408e8a` | backend=wasm |
| `pr_8547/boundary-diagnostic/8532-page3-items.json` | `5a2f1d6a59439d4e151281b9d15b2532c3e329f9a298c73ce7f278c38d403a49` | 중간 대응·진단 원장; 위 완료 기록과 개별 리뷰 참조 |
| `pr_8547/boundary-diagnostic/8532-source-row38.json` | `d4e427677392b154add1791c7c2d3d182ac0b0178824c4fff031d285a8ecc5c4` | 중간 대응·진단 원장; 위 완료 기록과 개별 리뷰 참조 |
| `pr_8547/boundary-diagnostic/observations.json` | `30bfb36fa94cf38024963797e83d0b5b681bd1e06998a6962b1424ea4a8dd064` | recorded_at=2026-11-04T19:26:52.854886+00:00; status=independent source/physical height path analysis pending; no implementation or test expectation change |
| `pr_8547/closing-frame-c183a1ac61/provenance.json` | `965667850bdc679088a1a9fed226bccaa59ff10a297d0cec89204e66adfaa81c` | recorded_at=2026-11-04T21:15:20.891022+00:00; code_commit=c183a1ac61c2aa4188989fd93032cae68d3538c1 |
| `pr_8547/closing-frame-c183a1ac61/source-frame-relation-diagnostic.json` | `68bb832f3f64a7cfbe333517182dc2444fe9222f11bc8de109d98184c60ff937` | recorded_at=2026-11-04T20:45:42.941754+00:00; status=기존 실제 출력에서 FAIL / 보정 실제 출력에서 PASS인 진단; 새 Rust 함수의 이전 버전 실행을 뜻하지 않음 |
| `pr_8547/closing-frame-e61ed1ca73/native-comparison.json` | `242ccd2ad37c6317e6312376bc391c8e4571453ab4a1f21ef657f5dbd0c0f4fd` | recorded_at=2026-11-04T20:09:12.681524+00:00; backend=native; status=Native/fresh WASM 해당 경로의 전쪽 수치 대조 완료; 직접 판독·회귀·통합 최종 검증 대기 |
| `pr_8547/closing-frame-e61ed1ca73/native-product.json` | `481bacd77163c0e0ba095fb2f3952f25f290ad3b13e2a87fee2969313624ded1` | code_commit=e61ed1ca7329d3dacffc9c7fc76e602f30f43f66 |
| `pr_8547/closing-frame-e61ed1ca73/native-provenance.json` | `590ac5ab9ad74eb688fcf1814417893075a515eaeab13a866c38486a62d26c71` | recorded_at=2026-11-04T20:10:38.101490+00:00; code_commit=e61ed1ca7329d3dacffc9c7fc76e602f30f43f66; backend=native |
| `pr_8547/closing-frame-e61ed1ca73/page3-owned-cut.json` | `0df70998af517c46277d8be87bf827b6f16e4bf360a7802bdbfb7b9675a3e65d` | code_commit=e61ed1ca7329d3dacffc9c7fc76e602f30f43f66 |
| `pr_8547/closing-frame-e61ed1ca73/wasm-comparison.json` | `ceb480af7d87c230df04616001b982d6a89725b3a477e4a7e8f7dc373da17652` | recorded_at=2026-11-04T20:26:02.868629+00:00; backend=wasm; status=Native/fresh WASM 해당 경로의 전쪽 수치 대조 완료; 직접 판독·회귀·통합 최종 검증 대기 |
| `pr_8547/closing-frame-e61ed1ca73/wasm-provenance.json` | `69442e9d973f302eab552a1d980aef0b3f9e769eb909d3aa977a5ec1cf4dcba3` | recorded_at=2026-11-04T20:32:50.794138+00:00; code_commit=e61ed1ca7329d3dacffc9c7fc76e602f30f43f66; backend=wasm |
| `rebase_scope_22947f6a90/base-comparison.json` | `c8df4f2417b9edc66ef1427dc1a869df64e1996374141ff67f5730bbc849c8a9` | recorded_at=2026-11-04T19:08:03.569754+00:00; cases=6 |
| `rebase_scope_22947f6a90/base-product-execution.json` | `a4a4e7b1d9a617c1236e61da61952b390209e549f6a06accb40174580c0ff2bb` | 중간 대응·진단 원장; 위 완료 기록과 개별 리뷰 참조 |
| `rebase_scope_22947f6a90/candidate-provenance.json` | `49bcbb50bd224c3cd4b6cfaa567648c20a8f0933f097c57cea6f65f75d63a322` | recorded_at=2026-11-04T18:22:44.992882+00:00; cases=6 |
| `rebase_scope_22947f6a90/native-base-comparison.json` | `0c43fa3641296ec3347e7a39c90cab17bba92c206cd5202c31709f7f47af3d01` | recorded_at=2026-11-04T18:53:34.510342+00:00; cases=3 |
| `rebase_scope_22947f6a90/panel-provenance.json` | `74c066f7d638893df9d814d95a768596c846d72dceb61372056abe5bf37c01ab` | 중간 대응·진단 원장; 위 완료 기록과 개별 리뷰 참조 |
| `rebase_scope_22947f6a90/previous-candidate-pixel-comparison.json` | `5dea12cb869d54351eca5b40c1f92c0ac19deb11bede088c68eaaa9bf19eeee2` | recorded_at=2026-11-04T18:36:29.261347+00:00; pages=10 |
| `regression-header-body-equivalence.json` | `5fe498c9674da8ea966fe01ec12ab8674766a68b0c9a464937c830dc97368b4c` | recorded_at=2026-11-04T19:02:26.996794+00:00 |
| `review-preparation-a7e5ce7112.json` | `f386c091ad25923c303218c4e9c1d1c0aa19208f69e280eb79464f9072205384` | recorded_at=2026-11-04T19:09:23.334880+00:00; product_sha=a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973; base_sha=22947f6a90c998e95fa51238d5b907a0ebd37162; status=preparation; final common validation pending |
| `review-preparation-links.json` | `2ee94d1ff2165dd4771bca884c7485b3654715fc4a9d5d8d1896f8ef67dc9e33` | recorded_at=2026-11-04T19:12:34.539836+00:00; exit_code=0 |
| `runtime-validation-c183a1ac61.json` | `401f786aad78347430d1fed18e15c3eeaf562cc1c03499143ce4ab882786289a` | recorded_at=2026-11-04T21:39:34.305154+00:00; code_commit=c183a1ac61c2aa4188989fd93032cae68d3538c1 |
| `rust-validation-a7e5ce7112.json` | `fdadcf3b399e8b3261aeaa3539b34ff09feda38c4e4b11211ae19f40d52f77ad` | recorded_at=2026-11-04T18:31:36.614559+00:00; base_sha=22947f6a90c998e95fa51238d5b907a0ebd37162 |
| `rust-validation-c183a1ac61.json` | `3b267bdd0ad09bbc39114a7edfe5c32122dc8b44bda4bd94e431ee88fe2be2b9` | recorded_at=2026-11-04T23:04:03.215253+00:00; base_sha=22947f6a90c998e95fa51238d5b907a0ebd37162 |
| `test-header-audit-c183a1ac61.json` | `174003eb22c4bce4d6ac39556252189f1d998609f0e6445b9df42b1ec1a3179c` | recorded_at=2026-11-04T21:47:02.182975+00:00 |
| `whole-page-capture-summary.json` | `bba281e130e4f2c2a893e2d0bae86528b4f40362bb33451ec12aacb04bec6bcc` | recorded_at=2026-11-04T18:12:23.127933+00:00 |
