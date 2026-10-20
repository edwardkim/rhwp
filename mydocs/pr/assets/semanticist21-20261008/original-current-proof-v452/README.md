# 원 PR 편집·정상 대조의 현재 Native/fresh 증거

생산 코드 `b4ff8c81b5d986baf47bad9a3eee83cbd9d8367b`, 정책 base
`552f5228d2dbfd6046c838f14241704f1bcb8a05`.
Native24쌍27쪽 exit0·892.431초, fresh24쌍27쪽 exit0·1021.503초.
양쪽 최저 `current-04` 2쪽92.07644%, 미달/누락0이며 각 원본/독립 Print
전체 쪽수가 일치한다. 입력/Print SHA와 source guard는 현재 바이트로 확인했다.
WASM SHA-256 `94fe70f13609c5a006b8e9044463fac7ae96fe5fe03ae33bc0c0153b19f7867c`.

27쪽 Native/fresh 및 이전7f04c 소스의 실제 RGBA는 전부 동일하다. 현재 대표
16개 review를 직접 열었으며, 최저 원본 본문/각주를 같은 좌표2배/3배 확대했다.
누름틀 외곽·셀 소속·뒤 문단, 스타일 삭제의 가시 대조군과 HWP/HWPX 저장 결과,
본문·각주 번호·구분선·쪽 소속·내용 순서를 대조했다. 글자 획·폭·자간의
기존 차이는 남으며 독립 Print와 완전 픽셀 일치를 주장하지 않는다.
가맹사업/CGMP/KPS/K-water 전체나 Canvas의 미완료를 이 범위 통과로 대신하지 않는다.

## 재현과 원문 증거

전쪽 `--silhouette-only --compat 2022 --embed-fonts full`과 검증된 공급 글꼴을
사용하고 fresh에는 `--wasm-pkg pkg`를 지정했다. 정확한 명령/입력/Print/폰트/
source/package SHA·TSV·최종 로그·직접 판독·PNG 해시는 ignored
`output/pr-review/semanticist21-20261008/full-width-visible-alignment-v452/`의
`original-edited-current-plan.json`, `original-edited-{native,fresh}-result.json`,
각 대상의 `run_manifest.json`/`silhouette_manifest.json`/`silhouette.tsv`,
`original-current-qualification.json`, `original-edited-zooms/provenance.json`에 보존한다.
확대는 왼쪽 rhwp/오른쪽 Print: 2쪽 본문 `(150,150)-(715,725)` 2배,
1쪽 각주 `(150,910)-(715,1030)` 3배다.

## 입력별 전쪽 결과와 독립 기준

| 대상 | 쪽수 | 양쪽 최저(%) | 입력 / Print SHA-256 |
| --- | --- | --- | --- |
| `current-01-31414e6363d5` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/current-print-inputs-v175/additional-edited-note-style-blank-endnote-v175.hwpx` / `pdf/semanticist21-20261008/new_note_paragraph_style-0460-31414e6363d5-2020.pdf`<br>`31414e6363d510ce28bdbb933b24431ac1f2642f7d65b4ef219c74340a3e973d` / `cb9699e0a96125f882bb3d556c612f8280c3c80b4a3abb4d82b10134fab5d9b9` |
| `current-02-56839e167e11` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/current-print-inputs-v175/additional-edited-note-style-blank-footnote-v175.hwpx` / `pdf/semanticist21-20261008/new_note_paragraph_style-0458-56839e167e11-2020.pdf`<br>`56839e167e1177d8c29b636a93508e0d59e7874e5242fc8ae6cb6c320a361d3d` / `d1a129b1c2367ec6031688aa46dc9ab0c81a532316fdd56febe18c0cc86e541f` |
| `current-03-b3c9896ea898` | 1 | 97.04843 | `mydocs/pr/assets/semanticist21-20261008/current-print-inputs-v175/additional-edited-note-style-hancom-hwp-footnote-v175.hwpx` / `pdf/semanticist21-20261008/new_note_paragraph_style-0464-b3c9896ea898-2020.pdf`<br>`b3c9896ea898d3b3f9185b473a7328bc53ac886c0f607221a9df28b7d483f5ad` / `1b6f51dec5e6c0dc72be5875359650c223e6ee4c102ec9044535a08d187bf0c5` |
| `current-04-6b4f6a375029` | 2 | 92.07644 | `mydocs/pr/assets/semanticist21-20261008/current-print-inputs-v175/additional-edited-note-style-missing-named-style-v175.hwpx` / `pdf/semanticist21-20261008/new_note_paragraph_style-0462-6b4f6a375029-2020.pdf`<br>`6b4f6a375029dcb0c9def5b4443683d1b55cf09acbdb9cb9ab87ac2d94888edb` / `778594c4e099ec86ddaa825f1dd307c195c4ffe49b842a5082df0c15274d6828` |
| `current-05-17a174162fd3` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/current-print-inputs-v175/clickhere-wrapper-current-v175.hwpx` / `pdf/semanticist21-20261008/pr-preparation-current-inputs-v175/clickhere-wrapper-current-v175-17a174162fd3-2020.pdf`<br>`17a174162fd3fcd1c51bd04a25a577396c44bd51ad1c8ab001f9184b16dc4de3` / `c87e5e65928d4b5dfa1a47c11b06104af0150cfd03ee63082ad13b52a78fa1ba` |
| `current-06-fd079a4fe1a5` | 2 | 97.04919 | `mydocs/pr/assets/semanticist21-20261008/current-print-inputs-v175/additional-edited-note-style-hancom-hwp-endnote-v175.hwpx` / `pdf/semanticist21-20261008/pr-preparation-current-inputs-v175/additional-edited-note-style-hancom-hwp-endnote-v175-fd079a4fe1a5-2020.pdf`<br>`fd079a4fe1a5a9b338f72d58820f8b249dfd679081a657beb09d0afacf436a9e` / `df186aff12e586a72ceb8eb48143d1e0f4320e29314136296c603033cad069a8` |
| `current-07-ac171fe6f2f9` | 2 | 97.77838 | `mydocs/pr/assets/semanticist21-20261008/current-print-inputs-v175/additional-edited-note-style-hancom-hwpx-endnote-v175.hwpx` / `pdf/semanticist21-20261008/pr-preparation-current-inputs-v175/additional-edited-note-style-hancom-hwpx-endnote-v175-ac171fe6f2f9-2020.pdf`<br>`ac171fe6f2f9a71daccfc68b1277c1f9f2bfee9d14bcedfbc4e12b09199b826c` / `9f472a01c8d654c4022f0772a7676ef05f570a40557ea1e750d83db950bbbff2` |
| `current-08-12fb864391ef` | 1 | 97.77742 | `mydocs/pr/assets/semanticist21-20261008/current-print-inputs-v175/additional-edited-note-style-hancom-hwpx-footnote-v175.hwpx` / `pdf/semanticist21-20261008/pr-preparation-current-inputs-v175/additional-edited-note-style-hancom-hwpx-footnote-v175-12fb864391ef-2020.pdf`<br>`12fb864391efd32d8e34ff9e8583af0b540c13e6c224a399a0505356b2439810` / `7f05b25dce68fbd035b8bc3a5b10c8cfe593ee2a1c6464e9c4745aa4379da9d0` |
| `child-offset` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/wrapper-anchor-counters-v289/child-offset-8b1ec78415f1.hwpx` / `pdf/semanticist21-20261008/wrapper-anchor-counters-v289/child-offset-2020.pdf`<br>`8b1ec78415f1b35232de4a6f1d5964bd2b5039d15a2049b855a638160c4abfc5` / `5b96ff466e5f3f6a3f348448eb4d941bcf6ff56c72264c5505c16c3268519b8c` |
| `parent-padding` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/wrapper-anchor-counters-v289/parent-padding-61e9cbd25806.hwpx` / `pdf/semanticist21-20261008/wrapper-anchor-counters-v289/parent-padding-2020.pdf`<br>`61e9cbd2580635e282ebc5489988152b458f3eee94f64f8aba749f6d7bc2e6ff` / `56d0f3b0773b1aeb243db404eddcac04bd74a2745758272cf93239bc27fe7d8f` |
| `child-flow` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/wrapper-anchor-counters-v289/child-flow-faeb7331e8fb.hwpx` / `pdf/semanticist21-20261008/wrapper-anchor-counters-v289/child-flow-2020.pdf`<br>`faeb7331e8fb9ecd81df5dc761847197c1e7c1ff99c8cfcf02e4b58d58bbe576` / `ae28e12d6665682c8040415f760d35af9f395294c4aac0656408b1d910faf454` |
| `child-inline` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/wrapper-anchor-counters-v289/child-inline-fda0d3d71ef2.hwpx` / `pdf/semanticist21-20261008/wrapper-anchor-counters-v289/child-inline-2020.pdf`<br>`fda0d3d71ef2729fee28c0c685fac5bad5c71943f1346676fdfd99df17d1130d` / `1b82b4e57205af99268c67e77ab46540883955455e1b30d929f50ad252324000` |
| `parent-height` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/wrapper-anchor-counters-v289/parent-height-96f2cd69236d.hwpx` / `pdf/semanticist21-20261008/wrapper-anchor-counters-v289/parent-height-2020.pdf`<br>`96f2cd69236d0be49f62d10c86b39cb20b082384b48946be875cf7d6e7cd4a88` / `860ee5d293ba53b51c82eb280e1d268aca5f2c5e42f2c0f86b7cebb78e515a28` |
| `normal-save-hwp` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/wrapper-anchor-counters-v289/normal-save-hwp-bebf8d7399c2.hwp` / `pdf/semanticist21-20261008/wrapper-anchor-counters-v289/normal-save-hwp-2020.pdf`<br>`bebf8d7399c24fb70c545be6f65af899637d9496f3241e97fb51735957fb5056` / `cbc0c0bb52abb8cdb26c4da25279b91ea01434a94c6f1dd86fab050133a8b27a` |
| `pr7657-after-01-abfe32b6753b` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/pr7657-batched-correction-v406/after/counter-01-unchanged-empty.hwpx` / `pdf/semanticist21-20261008/original-pr7657-cell-counters-v406/counter-01-unchanged-empty-abfe32b6753b-2020.pdf`<br>`abfe32b6753b3cd686ab161046918718d39297139abee0c0508635b2fe688b62` / `8d30808ac58ee634aa0b3a1653bf4fb108dacf98f830e3049a2b12e7093eb05a` |
| `pr7657-after-02-8ebbea4fa234` | 1 | 99.76088 | `mydocs/pr/assets/semanticist21-20261008/pr7657-batched-correction-v406/after/counter-02-visible-style.hwpx` / `pdf/semanticist21-20261008/original-pr7657-after-v406/counter-02-visible-style-8ebbea4fa234-2020.pdf`<br>`8ebbea4fa2349165a036ab6a7d832d7b475ce001cccd4715458cd4ed8d4703e6` / `38a281f126f2fe383639d3b8f1c9932317bcf9d6153d9fffe9ce52c9eb61fb6c` |
| `pr7657-after-03-c09b29828e2b` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/pr7657-batched-correction-v406/after/counter-03-first-of-two.hwpx` / `pdf/semanticist21-20261008/original-pr7657-after-v406/counter-03-first-of-two-c09b29828e2b-2020.pdf`<br>`c09b29828e2b0780faa8bb28c65b96e3c03bc73b2a5f29abb64540eec1b584d7` / `e7785732ad79741e5fafca8fb3e0e8537a16ab6ccf8cbffd4c18cbc5c7abdea3` |
| `pr7657-after-04-7291e9f07690` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/pr7657-batched-correction-v406/after/counter-04-second-of-two.hwpx` / `pdf/semanticist21-20261008/original-pr7657-after-v406/counter-04-second-of-two-7291e9f07690-2020.pdf`<br>`7291e9f076905a69e9ec7afd8b7da0247eadec011dac7c68e35ac83ca3928236` / `d68495d907bb76513ebee1570029774f6c54b1a62b230cefb19e6bb934d95f59` |
| `pr7657-after-05-d4d84c1d50ed` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/pr7657-batched-correction-v406/after/counter-05-direct-format-preserved.hwpx` / `pdf/semanticist21-20261008/original-pr7657-after-v406/counter-05-direct-format-preserved-d4d84c1d50ed-2020.pdf`<br>`d4d84c1d50ed75e54150605d7e62b080481d646bf391f0b7fa4af9a9c8d8610e` / `fb2d5d93456f347973ef43838a10013506a981e12b02bd59ef2853b6877f1e5a` |
| `pr7657-after-06-3fd34172d6c9` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/pr7657-batched-correction-v406/after/counter-06-restored-para-shape.hwpx` / `pdf/semanticist21-20261008/original-pr7657-cell-counters-v406/counter-06-restored-para-shape-3fd34172d6c9-2020.pdf`<br>`3fd34172d6c9934e05951a5e52035e5d68a63f959c048ccae530ae726f9d10a9` / `0ecb759b0adee459b1717ae7d1e69f7978780214022e92dc8e6d5c1fa39996fa` |
| `pr7657-after-07-c1e9bcef6a63` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/pr7657-batched-correction-v406/after/counter-07-idempotent-style.hwpx` / `pdf/semanticist21-20261008/original-pr7657-after-v406/counter-07-idempotent-style-c1e9bcef6a63-2020.pdf`<br>`c1e9bcef6a63c593061ee95b2e3fcb6f1e60bcea16c749dfc6c6bb9c45f5457c` / `026ec70b46623be06c6f8041248e6a5085df3063ff3e33a514557140ffe4bfc9` |
| `pr7657-after-08-4ab2beea5fd6` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/pr7657-batched-correction-v406/after/counter-08-first-para-spacing.hwpx` / `pdf/semanticist21-20261008/original-pr7657-after-v406/counter-08-first-para-spacing-4ab2beea5fd6-2020.pdf`<br>`4ab2beea5fd61706834bf8f929347e0be4b9b4de3e117ecef59f7b62623ea379` / `e131778b5e2b92d68717166b2d8e82a5500ed58d110b4996355a43ec2ebe8e51` |
| `pr7657-after-10-e788c42c854c` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/pr7657-batched-correction-v406/after/original-cell-style-after-delete.hwp` / `pdf/semanticist21-20261008/original-pr7657-after-v406/original-cell-style-after-delete-e788c42c854c-2020.pdf`<br>`e788c42c854c71b454563e554a0f95863bf5c56325d5bbb13e53920cb22d5fcc` / `32344ee08712aac6d6e8e5cf6bd8999c9b2c835846fc0e445f943ba3be9c7bf5` |
| `pr7657-after-11-a1805d6c9409` | 1 | 100.00000 | `mydocs/pr/assets/semanticist21-20261008/pr7657-batched-correction-v406/after/original-cell-style-after-delete.hwpx` / `pdf/semanticist21-20261008/original-pr7657-after-v406/original-cell-style-after-delete-a1805d6c9409-2020.pdf`<br>`a1805d6c940915094d5a97397159f29ea6f4310151d379136cc1e768353d9f00` / `121668ce8d33b5e9e3169844d883fa4fc36eaa2421c3492c4df8f92b7c23e05e` |

## PNG SHA-256

- `native-child-offset-p001-review.png` — `d57253aaf22b5aa27a9a2efd6a3f0e111021dec93007b156977138b7c6e1aeaf`
- `native-child-offset-p001-overlay.png` — `aee72710d42456860651becc947e0dded1d5d786cd5ed3a787364dc6bf19d03c`
- `fresh-child-offset-p001-review.png` — `90fc6e38b2bf2ce7b0f524a040f69ce483647a2116ff028e7d4df16f9f2544be`
- `fresh-child-offset-p001-overlay.png` — `3548325240a706e84daafcb3039698d0ddabeb887706358e560a99175a2447f3`
- `native-current-04-6b4f6a375029-p001-review.png` — `00e1ac05af071dc34710c822f0035aa91dccdc1ebaa6e07e2b55b6b461857992`
- `native-current-04-6b4f6a375029-p001-overlay.png` — `0be1569111ce2b3d141bd4a55ab5570d5e2822d114929b61bbf162b6737545ce`
- `fresh-current-04-6b4f6a375029-p001-review.png` — `ea5565f7b313051f3a744c6cd6339af97677ae9fd98ce011bd9f9b358919a33a`
- `fresh-current-04-6b4f6a375029-p001-overlay.png` — `f36e22b93d4e30bfc96d12ad642c7bb6ff13dffe873d72462b45e728f944b9ea`
- `native-current-04-6b4f6a375029-p002-review.png` — `354cf22885144a9320f1b67a7e9753f0e2d387d88909612d9014e2c35efb4a6e`
- `native-current-04-6b4f6a375029-p002-overlay.png` — `762d4cb96f8a508a64948d14a7734a7c165c3208d5765c1682aecff4c44b7d35`
- `fresh-current-04-6b4f6a375029-p002-review.png` — `9941220c1a6d2b331472b2a6758dc85a4b76a33334d115c81b410e9d2d1cd9ff`
- `fresh-current-04-6b4f6a375029-p002-overlay.png` — `83dc850a28c6d1737f0a5b277191594f2e58c1c6880dd664d81463835861b186`
- `native-current-05-17a174162fd3-p001-review.png` — `39396c5b40b9660f01da023179f8dcf08a70d039a41e21d008966161e6246c13`
- `native-current-05-17a174162fd3-p001-overlay.png` — `a9b987e7986210df664b4fd5f686a36a2ff85b43575668c5a0d5a2c87a9ac097`
- `fresh-current-05-17a174162fd3-p001-review.png` — `d9e31c4c97701c24f19720205f35d4cd033ddae952ae7ea511e6a69db778b315`
- `fresh-current-05-17a174162fd3-p001-overlay.png` — `f042b990e70404a6ea1f030f353843aef2e3e5119e4a20ae1317a1d15b7c6839`
- `native-normal-save-hwp-p001-review.png` — `91b4ea1bf0c3aa0801d339fb0e6a0f812fc8a6def49c08935a34d506b5dd961d`
- `native-normal-save-hwp-p001-overlay.png` — `4e87d3ddaecd22b82fc5f888de44a35d0def445e176ba178602e42095c2c970d`
- `fresh-normal-save-hwp-p001-review.png` — `f80c7388eb9d7a576c87243b2e17561d0fea03a450008448a0297f3f6b7d2527`
- `fresh-normal-save-hwp-p001-overlay.png` — `66a949f5e277aba6089c0e7c86e991e253f9e9b0052f96b551785c2bf425ba0f`
- `native-pr7657-after-02-8ebbea4fa234-p001-review.png` — `fd97bb5525551ad88d2182abad8aeadafc4fad846afc0da80d1c43dd61b7d822`
- `native-pr7657-after-02-8ebbea4fa234-p001-overlay.png` — `2d92cada27a224d97ca80323dac622650064a2a32dd36d02174f5f90dcb9ce81`
- `fresh-pr7657-after-02-8ebbea4fa234-p001-review.png` — `999be97f7bafcc1b05cf1210ddea44e8a6c1d350ab2e38284cfc9deea10427b5`
- `fresh-pr7657-after-02-8ebbea4fa234-p001-overlay.png` — `2eb814f3cb04deeb3960a9a2e76c7f5dbe25a9c59c25a6dea354881f56e9a7b8`
- `native-pr7657-after-10-e788c42c854c-p001-review.png` — `773ca50954584911600be14ceed5a3d0403cd0c84cfcf46fcf05aa0c3ddc7c95`
- `native-pr7657-after-10-e788c42c854c-p001-overlay.png` — `1b2d54409bc89a8661a7d8c11efed0eaea2e30bd695cb0f2446b3edfeb853f8a`
- `fresh-pr7657-after-10-e788c42c854c-p001-review.png` — `eb6d319663db7332cfb8efeea04a0b2120c2c9cfcab23230522be2b03ee1b339`
- `fresh-pr7657-after-10-e788c42c854c-p001-overlay.png` — `67927290266348298b87325fe85a93b0bcf037e98dd90b4c228935b401a1e988`
- `native-pr7657-after-11-a1805d6c9409-p001-review.png` — `88f1e2b0481b136c6963bec332f1e714e32ce0d7eac37df11c8c7c1b98cf1985`
- `native-pr7657-after-11-a1805d6c9409-p001-overlay.png` — `668bd0936c5475ddcf6628527048c1b4480be90472aabbd055fefa32b91958f7`
- `fresh-pr7657-after-11-a1805d6c9409-p001-review.png` — `57635138c2ffc6e29fbf2a05525aff3dffcab6c6ba6626f6c3ea0b8cba6bf8eb`
- `fresh-pr7657-after-11-a1805d6c9409-p001-overlay.png` — `807fea5f8cbe6f3a8614953d2e732e6f90460736d0de876fad068f964c16932d`
- `lowest-fresh-p001-footnote.png` — `57c5cd898c8f2fcd6f8a37c64a08d6882e6472789e8c027404cad0675414e42a`
- `lowest-fresh-p002-body.png` — `4b09232ccd044a41b05819c49425874602b70581cb6f406236c99894c3765123`
- `lowest-native-p001-footnote.png` — `57c5cd898c8f2fcd6f8a37c64a08d6882e6472789e8c027404cad0675414e42a`
- `lowest-native-p002-body.png` — `4b09232ccd044a41b05819c49425874602b70581cb6f406236c99894c3765123`
