# 통합 변경의 독립 Print 시각 증적

- 검증 code SHA: `39a634e547621c32ae6f9ba8c1694d35c827078b`
- 기준 devel SHA: `463c48cc228170b576c832591b76826116e86a49`
- Native/fresh WASM 각 21문서·181쪽을 96dpi PNG로 새로 출력했다. 전쪽 90% 이상, 독립 Print 쪽수 일치다.
- 전체 TSV 원문과 SHA는 이 code 변경을 제출한 통합 PR 본문에 문서별로 보존한다. TSV·중간 JSON·실행 로그는 Git에 커밋하지 않는다.
- 좌표를 고정한 핵심 페이지는 양쪽 98% 이상이다. 독립 Print 좌표 ±2px와 내용·소속·순서 관계를 함께 검사한다.
- 렌더 소스 SHA manifest를 커밋 파일과 대조했다. Native 및 fresh WASM producer의 소스가 이 code SHA와 같다.
- fresh WASM SHA-256: `839e76297ee8ca90c053f1ce46542b96b5c21b4cddde02a415b0975e4903affc`
- Studio 1,875개 단위 검사, Studio·Chrome·Firefox·VSCode 실제 배포 빌드 및 폰트/확장 배포 계약 검사를 통과했다.
- Rust 회귀는 최초10,354PASS/8FAIL/45SKIP에서 사용자 지정으로 실물6건 이관·폰트 계약2건 수정 후 관련55PASS를 합산해 통과 처리했다. 새 head에서 전체를 재실행했다고 보고하지 않는다.
- 최종 Skia·문서 검사와 fmt·Clippy 3경로를 통과했다. 정확한 명령과 원본 로그·TSV는 ignored `output/pr-review/planet6897-followup-20261011/`에 보존한다.
- 각 review PNG는 실제 출력·독립 Print·overlay이며 standalone overlay도 함께 보존한다.
- Native #7669 p1 −0.01179%p, fresh WASM ratio_steps p1 −0.05623%p의 소폭 하락은 사용자 승인으로 수용했다. 90%/98% 조건은 유지했다.

## 셀 첫 저장 간격 보정의 근거

- 고용 문서의 해당 첫 문단에는 배치 상자가 없는 `ColumnDef(colCount=1)`과 텍스트가 있다. 원본 앞 간격·첫 LineSeg vpos는 각각200 HWPUNIT다.
- 생산: `cell_paragraph_spacing::paragraph_before`가 흐름 앞 간격과 물리 저장 띠를 구분한다. 측정: `table_partial.rs` 선택 줄 높이가 `occupied_before()`를 소비한다.
- 실제 배치: 같은 파일의 `allow_first_stored_lead`가 원래 `controls.is_empty()` 때문에 단 설정 메타데이터를 거부했다. 이를 공용 `control_is_width_neutral_marker`의 전부 만족 조건으로 바꾸고 `layout_composed_paragraph_in_frame`의 column-top trim에 전달한다.
- Table·Picture·Shape·Equation·각주/미주 등 배치 상자를 소유한 컨트롤은 이 조건에서 제외된다. 기존 단일 열 스냅·선형 셀·첫 줄 이후 컷 조건은 유지한다. 문서 ID나 특정 좌표 예외는 없다.
- 26번 첫 글자 기준선은 독립 Print112.99651px, 보정 전110.44px, 보정 후113.10667px다. 같은 독립 기대값 ±2px에서 FAIL→PASS를 확인했다. 27번은 선행 공백 run에 대응하는 PDF 공백 원점을 사용한다.
- 정상 대조를 포함한 관련 검사29건과 새 핵심 위치 검사21건을 실행했고 최초 전체 회귀에서도 함께 통과했다. 문단·표/셀 소속, 내용의 한 번 보존, 표 내부 포함과 앞뒤 순서를 함께 보호한다.

## 문서별 전쪽 결과

| 입력 | 전체 쪽 | Native 최저% | fresh WASM 최저% | 좌표 보호·대표 PNG 쪽 |
| --- | ---: | ---: | ---: | --- |
| `samples/issue6025/3232693_employment_support_criteria.hwpx` | 4 | 98.81410 | 96.17493 | 4 |
| `samples/issue7702/curly_quote_latin_slot.hwpx` | 1 | 99.48959 | 99.51554 | 1 |
| `samples/issue7816/base_charpr0_20pt.hwpx` | 1 | 99.98461 | 100.00000 | 1 |
| `samples/issue7816/base_style_charpr_20pt.hwpx` | 1 | 99.99227 | 100.00000 | 1 |
| `samples/issue7816/odd_margin_left.hwpx` | 1 | 99.72581 | 99.93642 | 1 |
| `samples/issue7816/odd_margin_right.hwpx` | 1 | 99.75112 | 99.92752 | 1 |
| `samples/issue7816/odd_spacing_next.hwpx` | 1 | 99.28751 | 99.80752 | 1 |
| `tests/fixtures/issue5598/narrow-cell-indent--7832-defined.hwpx` | 1 | 100.00000 | 100.00000 | 1 |
| `tests/fixtures/issue5598/narrow-cell-indent-0-defined.hwpx` | 1 | 100.00000 | 100.00000 | 1 |
| `samples/issue7702/half_space_ties.hwp` | 2 | 100.00000 | 100.00000 | 1 |
| `samples/issue7702/ratio_steps.hwp` | 2 | 98.30939 | 99.23084 | 2, 1 |
| `samples/issue7702/relative_size_letter_spacing.hwp` | 2 | 99.69124 | 99.94499 | 2, 1 |
| `samples/issue7702/size_ratio_relsize.hwp` | 2 | 99.82457 | 99.90344 | 2, 1 |
| `samples/issue7816/indent_room_boundary.hwpx` | 2 | 99.41454 | 99.84689 | 2 |
| `samples/issue7816/odd_spacing_prev.hwpx` | 2 | 99.61954 | 99.88219 | 2 |
| `samples/issue7824/synth_ladder_spacing_before.hwpx` | 2 | 99.60545 | 99.90553 | 2 |
| `samples/issue7095/36308670_split_row_declared_tail.hwpx` | 2 | 99.73672 | 99.83858 | 2, 1 |
| `samples/issue7816/indent_align_matrix.hwpx` | 3 | 99.50987 | 99.86866 | 3 |
| `samples/issue7816/odd_indent_ladder.hwpx` | 3 | 99.49224 | 99.85350 | 3 |
| `samples/task1749/saved_bounds_cumulative_page_break.hwp` | 5 | 99.22885 | 99.68462 | 1, 2, 5 |
| `samples/issue5966/1130000-202100008_franchise_review_report.hwp` | 142 | 92.22387 | 92.79245 | 57, 71 |

## 입력과 독립 PDF

### remaining-employment

- 입력: `samples/issue6025/3232693_employment_support_criteria.hwpx` / SHA-256 `b53e9a83a79903b9c7cf69521519dcdf7b128348ac26ec21e6ac7b8fc6c0e441`
- 독립 한컴 Print: `pdf/pr_6088_6144/hancom2020/pr_6088_6144_issue6025_employment_support_criteria_3232693_employment_support_criteria-2020.pdf` / SHA-256 `985dab08280f0cb7a2489c796614337e230871428bc0ac98b5cd3be4b5abd579`
- native 전쪽 TSV SHA-256: `92f4bea2346c5a7e0a14c18712b06adc3f4beaa06e3b94740ee3b50662a448d0`
- wasm 전쪽 TSV SHA-256: `f58a89aa5b2d37f24d607aef87b4950506f85d368fef0878c681c4c23af63e8d`
- [remaining-employment-native-p004-review.png](remaining-employment-native-p004-review.png) / SHA-256 `f9c72bbe7ef21189749024767b6ef6e7d466646647b5e4e160d479c27b289f71`
- [remaining-employment-native-p004-overlay.png](remaining-employment-native-p004-overlay.png) / SHA-256 `f6907785b31745bb79ccb714963649b46f9aa9e7d4080753c42aacc0e25c013e`
- [remaining-employment-wasm-p004-review.png](remaining-employment-wasm-p004-review.png) / SHA-256 `6050b0938e955198fc5385119ef9e44e58586b3a745e939162dd85b5f60ba504`
- [remaining-employment-wasm-p004-overlay.png](remaining-employment-wasm-p004-overlay.png) / SHA-256 `0dbe07307033fccc0a6c22c006dd4992281bba167d47c0cd1314ef47ce8712c4`

### issue7702-curly_quote_latin_slot

- 입력: `samples/issue7702/curly_quote_latin_slot.hwpx` / SHA-256 `3e77b28b6c3787c255ed41c7fcbe308b4d9e059f816752aa418ad6016ee5a3b0`
- 독립 한컴 Print: `pdf/issue7702/curly_quote_latin_slot-2020.pdf` / SHA-256 `6f8cec61bbbb364e9c0db158bbe34e5fad7c926add30104f61fcb4d8824a56be`
- native 전쪽 TSV SHA-256: `188b487870cd7af73efb29b8cd02e0fb8044da02c84e1e72a76675f7a927f1e0`
- wasm 전쪽 TSV SHA-256: `75b9710bcae3676e0a9c78c4057fd047f0d1a5268db0416106def5d139530679`
- [issue7702-curly_quote_latin_slot-native-p001-review.png](issue7702-curly_quote_latin_slot-native-p001-review.png) / SHA-256 `7f15951b3f6f50be70e17c8eb6c6d9b88f4cfccaab800be9eee0cbf5c12243ee`
- [issue7702-curly_quote_latin_slot-native-p001-overlay.png](issue7702-curly_quote_latin_slot-native-p001-overlay.png) / SHA-256 `4693620b770eb90fd8115eba892ae40f4a9a976e61b989c016fe7a271ea3f86e`
- [issue7702-curly_quote_latin_slot-wasm-p001-review.png](issue7702-curly_quote_latin_slot-wasm-p001-review.png) / SHA-256 `a004404fb7f449faf4de51a19612719d8a9d13eced36ac59c5b7484bda3d79b7`
- [issue7702-curly_quote_latin_slot-wasm-p001-overlay.png](issue7702-curly_quote_latin_slot-wasm-p001-overlay.png) / SHA-256 `812525e5aeeedb6f08b521bbb1e92c9ab407db3ab1bf040c14832cc095fbea1f`

### issue7816-base_charpr0_20pt

- 입력: `samples/issue7816/base_charpr0_20pt.hwpx` / SHA-256 `56a6dba2b7a3e71b3bec9c046414971d9cbcc1d375e8a6e2065d2e8e8cd68bb7`
- 독립 한컴 Print: `pdf/issue7816/base_charpr0_20pt-2020.pdf` / SHA-256 `13b3de1d63d98c578f92f0741ad0676d7f7eb5f52cf821f96451d47d7001032a`
- native 전쪽 TSV SHA-256: `e0ff5e942bb069a88b60252e2f86483f2338626e185890c3321502fa05ed4734`
- wasm 전쪽 TSV SHA-256: `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b`
- [issue7816-base_charpr0_20pt-native-p001-review.png](issue7816-base_charpr0_20pt-native-p001-review.png) / SHA-256 `4794fc7f8ea2d4032bc091a06ec92c6fb01b74bd3b888d0f121cfaaf35845583`
- [issue7816-base_charpr0_20pt-native-p001-overlay.png](issue7816-base_charpr0_20pt-native-p001-overlay.png) / SHA-256 `614c3c44dee9d540f875db2a6ec12f735fae65db2f7aca72e372602a27b1ec5d`
- [issue7816-base_charpr0_20pt-wasm-p001-review.png](issue7816-base_charpr0_20pt-wasm-p001-review.png) / SHA-256 `73c96389eaec4c5e952cdee909129baf300215367a49b4a7e018e2501ba65c1e`
- [issue7816-base_charpr0_20pt-wasm-p001-overlay.png](issue7816-base_charpr0_20pt-wasm-p001-overlay.png) / SHA-256 `0151e0a253a4c8c0a6f92c6b452f16c48798f90d0f8c03417af61095197c9a1f`

### issue7816-base_style_charpr_20pt

- 입력: `samples/issue7816/base_style_charpr_20pt.hwpx` / SHA-256 `ce2138d14ed74d0e89114a7e6b53790ccee34f935ebe0dfd2a8dd10c961511fa`
- 독립 한컴 Print: `pdf/issue7816/base_style_charpr_20pt-2020.pdf` / SHA-256 `0dfc22976291339552174b817cd06232f47081e2a67b83c7664383db738c75e4`
- native 전쪽 TSV SHA-256: `0c55bc0c3bfcf95b77a8826be045f7a5b41f77b3006b2590a99ad47798a73e13`
- wasm 전쪽 TSV SHA-256: `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b`
- [issue7816-base_style_charpr_20pt-native-p001-review.png](issue7816-base_style_charpr_20pt-native-p001-review.png) / SHA-256 `8872fca18eeba604e09042e868b33b32a26bd699890701b6df3ad5fe9186e849`
- [issue7816-base_style_charpr_20pt-native-p001-overlay.png](issue7816-base_style_charpr_20pt-native-p001-overlay.png) / SHA-256 `5986fc88741c56e1afe02bf1dd6ac1082be21f0ea96751adcd8d7afeb20cb09e`
- [issue7816-base_style_charpr_20pt-wasm-p001-review.png](issue7816-base_style_charpr_20pt-wasm-p001-review.png) / SHA-256 `1fdca005d2bab8318bc7b6781b422c597edb0aec597103d1dd0ab4263de2ab65`
- [issue7816-base_style_charpr_20pt-wasm-p001-overlay.png](issue7816-base_style_charpr_20pt-wasm-p001-overlay.png) / SHA-256 `b387500541b19bdc9210804fd6dfb1d58ef583ddd10ab355294d5dc8bbcc52ab`

### issue7816-odd_margin_left

- 입력: `samples/issue7816/odd_margin_left.hwpx` / SHA-256 `d2df1cda9f8d5fdd1357331f90bc5e4afa68af128c6e047c721c0d5c1b50c08b`
- 독립 한컴 Print: `pdf/issue7816/odd_margin_left-2020.pdf` / SHA-256 `0a28ee4533f9c04a66cf3ac89975a930d817925e3a1980ba3dea2c87d7e03d63`
- native 전쪽 TSV SHA-256: `6b9d55d446f90c9807551180321396d59039ac5ac40b7e7b3b6c8d8b01d6b68f`
- wasm 전쪽 TSV SHA-256: `f54856025dc12b0dc72a3bfd7893278712c7eac6dc9ad0f8f3d789165b5617c1`
- [issue7816-odd_margin_left-native-p001-review.png](issue7816-odd_margin_left-native-p001-review.png) / SHA-256 `514348496f6763d62991183179f53bb70ec5ea8bec81e96eb603206c1dc073e7`
- [issue7816-odd_margin_left-native-p001-overlay.png](issue7816-odd_margin_left-native-p001-overlay.png) / SHA-256 `2d48d0a83e82350a95a8b8cd1b4cccc2e46843aae8c98e4ac0f22a1b0db9e206`
- [issue7816-odd_margin_left-wasm-p001-review.png](issue7816-odd_margin_left-wasm-p001-review.png) / SHA-256 `9cb33c05f392c215413d0d2b8011f0bc05a331b9f57b04cee6ecbb2744c8b542`
- [issue7816-odd_margin_left-wasm-p001-overlay.png](issue7816-odd_margin_left-wasm-p001-overlay.png) / SHA-256 `c4b0dea0e3c71364eb7958d1aed6eaf679d7673b65624609c87bd1a78b51a19f`

### issue7816-odd_margin_right

- 입력: `samples/issue7816/odd_margin_right.hwpx` / SHA-256 `8e0b98351133b87db88e71ccd7db5413687dc92b1e75c6b22e091ccdf93af807`
- 독립 한컴 Print: `pdf/issue7816/odd_margin_right-2020.pdf` / SHA-256 `afa61192e086b90e245df2eba4157b34ef01858c3f25202b0118694f80c1a9f8`
- native 전쪽 TSV SHA-256: `d3cf81508c19e43a4a6f316452f1bf949a6e737ef194b803a69975ab5d38d689`
- wasm 전쪽 TSV SHA-256: `7547bb6e7610e2258325b6a4259844b34e40d055f61140c8c40f470a616c5966`
- [issue7816-odd_margin_right-native-p001-review.png](issue7816-odd_margin_right-native-p001-review.png) / SHA-256 `eafa0bf0dc123fd5dbcd10d0eacb645a6ae8378aa634da1348927d2e937336f0`
- [issue7816-odd_margin_right-native-p001-overlay.png](issue7816-odd_margin_right-native-p001-overlay.png) / SHA-256 `e5354afa0562023e66416a83ea533a387732f7fc321e9f116852c06d29f26c8c`
- [issue7816-odd_margin_right-wasm-p001-review.png](issue7816-odd_margin_right-wasm-p001-review.png) / SHA-256 `0e924069fc9a5907eefca1f826d5dfa598fd426d66a9a8da1c2b0ef13cbd60f9`
- [issue7816-odd_margin_right-wasm-p001-overlay.png](issue7816-odd_margin_right-wasm-p001-overlay.png) / SHA-256 `1566c505cfa170aeddcc5a4ef41d4b69dd0dcf759a0830d030a7a55fa4d9830a`

### issue7816-odd_spacing_next

- 입력: `samples/issue7816/odd_spacing_next.hwpx` / SHA-256 `3f0d53d5411b99bc6577c5d669b5cf2617855d4a903ecb529496bab88edc82b5`
- 독립 한컴 Print: `pdf/issue7816/odd_spacing_next-2020.pdf` / SHA-256 `ebcfd95ed196b097a379b938b039db2d26ca7d0d2aadcaa9b51cdd4ea4df1bfd`
- native 전쪽 TSV SHA-256: `93dbf7935721aad6d0255ed26cbf3d7af80ae9c48cf9734d2cdae8435b1c12a5`
- wasm 전쪽 TSV SHA-256: `515c6e8b9f370ab370e6cab149c50f70a6a961c3d2b0aeaeccd93095af998cb8`
- [issue7816-odd_spacing_next-native-p001-review.png](issue7816-odd_spacing_next-native-p001-review.png) / SHA-256 `017bd4e50450d8e7df597eba4d47a337749008298bdbbb6b6d9b91780e3a6c2d`
- [issue7816-odd_spacing_next-native-p001-overlay.png](issue7816-odd_spacing_next-native-p001-overlay.png) / SHA-256 `e2d28419923e002821ea24ce9c6711f3e3cd3b8702428623510ff11f90c78c99`
- [issue7816-odd_spacing_next-wasm-p001-review.png](issue7816-odd_spacing_next-wasm-p001-review.png) / SHA-256 `6689ec9a32fa33e29141867b011c960d05b8e6af7f59ff1443cae1a81c225bb5`
- [issue7816-odd_spacing_next-wasm-p001-overlay.png](issue7816-odd_spacing_next-wasm-p001-overlay.png) / SHA-256 `57072d84ea0e2e6ebb08d90e896cae23d31324d8cf2e9a4165533becb783d068`

### narrow-cell-indent--7832-defined-rust

- 입력: `tests/fixtures/issue5598/narrow-cell-indent--7832-defined.hwpx` / SHA-256 `47950a486a132d7819c5ed011b04f81a77cffdaa61ce67969c926e0bd0a693b8`
- 독립 한컴 Print: `pdf/issue5598-rust-defined-controls-20261015/narrow-cell-indent--7832-defined-rust-2020.pdf` / SHA-256 `39c1559e62dc98e34d6530498bf838939bf290e5ad4b72241be18ac26cb5257a`
- native 전쪽 TSV SHA-256: `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b`
- wasm 전쪽 TSV SHA-256: `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b`
- [narrow-cell-indent--7832-defined-rust-native-p001-review.png](narrow-cell-indent--7832-defined-rust-native-p001-review.png) / SHA-256 `6f3f2793125572422bb53c6146c418fd02af2ea1bec8e371a87bd1b5393eecab`
- [narrow-cell-indent--7832-defined-rust-native-p001-overlay.png](narrow-cell-indent--7832-defined-rust-native-p001-overlay.png) / SHA-256 `792caee62016b3200460109308e449a04472d288554f671b1ab029c6f6595077`
- [narrow-cell-indent--7832-defined-rust-wasm-p001-review.png](narrow-cell-indent--7832-defined-rust-wasm-p001-review.png) / SHA-256 `e8534acf8c748375d5ab50c62956c491efe3f83bf3f185eebeaec3291c4fd6fd`
- [narrow-cell-indent--7832-defined-rust-wasm-p001-overlay.png](narrow-cell-indent--7832-defined-rust-wasm-p001-overlay.png) / SHA-256 `9897765808e6a6464c2960f8b87e7d7e2d2c39dc4413848fabd89e4dc5bf51c9`

### narrow-cell-indent-0-defined-rust

- 입력: `tests/fixtures/issue5598/narrow-cell-indent-0-defined.hwpx` / SHA-256 `039bc60fe5667542cb457a61ed005cbbb22911e45a6a0475dcd7aca2ae7bf117`
- 독립 한컴 Print: `pdf/issue5598-rust-defined-controls-20261015/narrow-cell-indent-0-defined-rust-2020.pdf` / SHA-256 `b2a1f82b7d171589cd3690703b9229886ee5c4a1b79ae04e13254ce38f5960a9`
- native 전쪽 TSV SHA-256: `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b`
- wasm 전쪽 TSV SHA-256: `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b`
- [narrow-cell-indent-0-defined-rust-native-p001-review.png](narrow-cell-indent-0-defined-rust-native-p001-review.png) / SHA-256 `acae448f78f36a5c6ddfd780b0b7f5c0a08755a464c927bfe32935bbef8a36f7`
- [narrow-cell-indent-0-defined-rust-native-p001-overlay.png](narrow-cell-indent-0-defined-rust-native-p001-overlay.png) / SHA-256 `a67b97c8e2f1ae919c4e07498ee29489b6efd60ac33a22b72b5ebcf1be318a63`
- [narrow-cell-indent-0-defined-rust-wasm-p001-review.png](narrow-cell-indent-0-defined-rust-wasm-p001-review.png) / SHA-256 `288dbb5fa21fc3a41245a35ca5f12611d0af6f140d117fe7ea18a8fcd8cbae24`
- [narrow-cell-indent-0-defined-rust-wasm-p001-overlay.png](narrow-cell-indent-0-defined-rust-wasm-p001-overlay.png) / SHA-256 `9704cb63c177f6f03e76bb675be0c1eab189432fc9eee9608fd1d12bc7b683f9`

### issue7702-half_space_ties

- 입력: `samples/issue7702/half_space_ties.hwp` / SHA-256 `68eb54ca9ed43acb33d39602c7b39b3b622f46f41cbf9a60a271f112a7f600d5`
- 독립 한컴 Print: `pdf/issue7702/half_space_ties-2020.pdf` / SHA-256 `b4f70af19d6ff02159b17cf746b6e0f18e5b4ebcd6b9451a19f4e67318fb6c63`
- native 전쪽 TSV SHA-256: `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c`
- wasm 전쪽 TSV SHA-256: `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c`
- [issue7702-half_space_ties-native-p001-review.png](issue7702-half_space_ties-native-p001-review.png) / SHA-256 `3e0578bb5bae7030773c3d6898a6ce28d46c6cb5a6a69f383ea02f2ccdd112ea`
- [issue7702-half_space_ties-native-p001-overlay.png](issue7702-half_space_ties-native-p001-overlay.png) / SHA-256 `1fdd1829d28ec006f6dcb334a8dcf42671868ee0479bd8514aa94425e33edf62`
- [issue7702-half_space_ties-wasm-p001-review.png](issue7702-half_space_ties-wasm-p001-review.png) / SHA-256 `dfde3201e9970d1519bd56c67b214db305ffa27bb6714f57773df9e165334b25`
- [issue7702-half_space_ties-wasm-p001-overlay.png](issue7702-half_space_ties-wasm-p001-overlay.png) / SHA-256 `4cbdce21f758e78ee5fa3b167a87024f91ac2aa79ef78f0c5031d141644fe471`

### issue7702-ratio_steps

- 입력: `samples/issue7702/ratio_steps.hwp` / SHA-256 `80ff2b2071de772caecad5d04bc8a96738ba621fb496d4f6c348b2293c00a548`
- 독립 한컴 Print: `pdf/issue7702/ratio_steps-2020.pdf` / SHA-256 `e17199a1e5ffcfe897d3f3eebb66c63a810fc7b1451755235c59e8d54cc03971`
- native 전쪽 TSV SHA-256: `bb19ecffa70bc8c2dff38b818d9fb8b1cee4df16a0f3502d010922382fa5063d`
- wasm 전쪽 TSV SHA-256: `08be84e3f787dd6174b06d34b81d9bcbc85aac9221eaa30ce085916cc46237c8`
- [issue7702-ratio_steps-native-p002-review.png](issue7702-ratio_steps-native-p002-review.png) / SHA-256 `4cac5728c85d829048768c31b855e0b034d52c5ea27c8bcad9d4d50feb3d676a`
- [issue7702-ratio_steps-native-p002-overlay.png](issue7702-ratio_steps-native-p002-overlay.png) / SHA-256 `057acefe5ecfc3c806142298f4fa281992812dcb6547398bbd211caeaf152e86`
- [issue7702-ratio_steps-wasm-p002-review.png](issue7702-ratio_steps-wasm-p002-review.png) / SHA-256 `b69e1c76a4425e22bdc242147606a76a67d9027f19812136bdb5f71027b27c72`
- [issue7702-ratio_steps-wasm-p002-overlay.png](issue7702-ratio_steps-wasm-p002-overlay.png) / SHA-256 `c48a7f4d60d68a2ea6fdf06cc5c5d254739bb19909b9c4e04ab54709d4917d56`
- [issue7702-ratio_steps-native-p001-review.png](issue7702-ratio_steps-native-p001-review.png) / SHA-256 `f606c34dd2e34ee798e46c0c9e691ddb633e43482074f362dd2be15f63411680`
- [issue7702-ratio_steps-native-p001-overlay.png](issue7702-ratio_steps-native-p001-overlay.png) / SHA-256 `f28c98042cd862b8deac53a8b523a63f1a15b5731c8c323391076151e63e849d`
- [issue7702-ratio_steps-wasm-p001-review.png](issue7702-ratio_steps-wasm-p001-review.png) / SHA-256 `4b6190c151a4367a05e99cb8b84b5494b6b85a6f2f4c7f08c1e1bfec7646ea78`
- [issue7702-ratio_steps-wasm-p001-overlay.png](issue7702-ratio_steps-wasm-p001-overlay.png) / SHA-256 `b496bae2598551ff35f573ad4f6b97e21323f7ed529eb80ed93229846d7a30f7`

### issue7702-relative_size_letter_spacing

- 입력: `samples/issue7702/relative_size_letter_spacing.hwp` / SHA-256 `435f0500fbdd18a68a66d5468dd8ce355c4d0cf90d0f7e02fab0b4048332ecbe`
- 독립 한컴 Print: `pdf/issue7702/relative_size_letter_spacing-2020.pdf` / SHA-256 `d5569e4b679ff27798b83e620262991f1b90428fd8987ae1f786e4ee7b182d47`
- native 전쪽 TSV SHA-256: `b5d49e83a4cffe3abdfb4d778af168ad08e2ef1430b59f57602e58c32d72f684`
- wasm 전쪽 TSV SHA-256: `0b641eebb22039ebcffc13d607c958bc719493eacaadc20c7f7c9919fed74a02`
- [issue7702-relative_size_letter_spacing-native-p002-review.png](issue7702-relative_size_letter_spacing-native-p002-review.png) / SHA-256 `9e3dafa8f1a32eac248878001f7e2d57859c233414322105f528dedb3e27825a`
- [issue7702-relative_size_letter_spacing-native-p002-overlay.png](issue7702-relative_size_letter_spacing-native-p002-overlay.png) / SHA-256 `ec3d937e0fb21157926d39821d3ab6932aca20355658acfcd4e5698bfdc9b3dd`
- [issue7702-relative_size_letter_spacing-wasm-p002-review.png](issue7702-relative_size_letter_spacing-wasm-p002-review.png) / SHA-256 `a99c6c44e2c8b4c6234e6ca11cdda42c3ff9c1a50fa0645d5d9e2e62a711b381`
- [issue7702-relative_size_letter_spacing-wasm-p002-overlay.png](issue7702-relative_size_letter_spacing-wasm-p002-overlay.png) / SHA-256 `8f28fbdaaaf66c7d4bd0eeeafde76cb605259cfe31e253ecae9760a387d4d513`
- [issue7702-relative_size_letter_spacing-native-p001-review.png](issue7702-relative_size_letter_spacing-native-p001-review.png) / SHA-256 `660d3a52754640ebf9b249e092a5fb3162048b1a9aa3ff4a4a8a212b48123bc1`
- [issue7702-relative_size_letter_spacing-native-p001-overlay.png](issue7702-relative_size_letter_spacing-native-p001-overlay.png) / SHA-256 `b0c7552c7f9deef1cc65037270f7aac83b8ecaba84d608ca86b93ec0198b6b97`
- [issue7702-relative_size_letter_spacing-wasm-p001-review.png](issue7702-relative_size_letter_spacing-wasm-p001-review.png) / SHA-256 `c0568855e33f5e85dcc8214aa641854aa38fce672e9c5ed2aa360b119685028a`
- [issue7702-relative_size_letter_spacing-wasm-p001-overlay.png](issue7702-relative_size_letter_spacing-wasm-p001-overlay.png) / SHA-256 `f4a6e86b07c1d84240ac69532e20fe6539638e1f060682fc4db85b55d097f867`

### issue7702-size_ratio_relsize

- 입력: `samples/issue7702/size_ratio_relsize.hwp` / SHA-256 `19d245828414a15e5cdc516feab055329a4d6c17e66116827e1823f040e19940`
- 독립 한컴 Print: `pdf/issue7702/size_ratio_relsize-2020.pdf` / SHA-256 `59929ead1ffa26cb6de0bc3a2eb7462d963541deb1c59eec8b4a6656cb5ac0b2`
- native 전쪽 TSV SHA-256: `4bf258430a72028669af4cc305567cba6e69ed2ee1ad148ee9bdb0275fe5ba63`
- wasm 전쪽 TSV SHA-256: `f3111e034eaea9ee137749050050996eb267bb86865e1481c0f7e8b2bf3702d7`
- [issue7702-size_ratio_relsize-native-p002-review.png](issue7702-size_ratio_relsize-native-p002-review.png) / SHA-256 `bcd83e7facc7b6f063b357f80afb413d2ee65c67d7cf6500db03e2995eb215a4`
- [issue7702-size_ratio_relsize-native-p002-overlay.png](issue7702-size_ratio_relsize-native-p002-overlay.png) / SHA-256 `381db11c82f0d5e4153c5819a4f9d98bb79f05fbbf6dee1dcfb081d70466d05c`
- [issue7702-size_ratio_relsize-wasm-p002-review.png](issue7702-size_ratio_relsize-wasm-p002-review.png) / SHA-256 `0bf9e7decf7fbda35fe6f6e847aa87b854a31863ff84f18b3cea1652da27e4bf`
- [issue7702-size_ratio_relsize-wasm-p002-overlay.png](issue7702-size_ratio_relsize-wasm-p002-overlay.png) / SHA-256 `20c52ca8194a1029fec517714c80fcd505f9cb32ee4dce3244dbc4633f43c765`
- [issue7702-size_ratio_relsize-native-p001-review.png](issue7702-size_ratio_relsize-native-p001-review.png) / SHA-256 `79c24d620a7e4175152d46d4252712b4503f1a11ce20b54b3ff9419b268be77f`
- [issue7702-size_ratio_relsize-native-p001-overlay.png](issue7702-size_ratio_relsize-native-p001-overlay.png) / SHA-256 `6ec897d63dfba571005108922d8bbd4f1d0029e2fbb20982a685daef26b6843e`
- [issue7702-size_ratio_relsize-wasm-p001-review.png](issue7702-size_ratio_relsize-wasm-p001-review.png) / SHA-256 `6f4517b0e04e460d27e2251a8cf126c2ac0125e05a18eafe335fa55c3ca45de4`
- [issue7702-size_ratio_relsize-wasm-p001-overlay.png](issue7702-size_ratio_relsize-wasm-p001-overlay.png) / SHA-256 `21179393070aad8195a1de5863941a25782ec95db3dab44d48c4767d2beb2e6e`

### issue7816-indent_room_boundary

- 입력: `samples/issue7816/indent_room_boundary.hwpx` / SHA-256 `729a16821f0c5cde949d84f8b588d49e081a9fe4dce3981a18c64cb082dc29a3`
- 독립 한컴 Print: `pdf/issue7816/indent_room_boundary-2020.pdf` / SHA-256 `b29b775705d6f8e2a1500b4cbbde967b783e2c0d421b3a925fca59a7672ca0ad`
- native 전쪽 TSV SHA-256: `ab0fac11a61c31984b26abaaca5c9fe424e92c0f5a4397e163bfbc253bc947f9`
- wasm 전쪽 TSV SHA-256: `266c5bf6cfcc0521ab65ce3b958c2322a8cb713da2b8e85ac088a415354cf8bf`
- [issue7816-indent_room_boundary-native-p002-review.png](issue7816-indent_room_boundary-native-p002-review.png) / SHA-256 `7e8e4bd481a4d2527f78dc06e56df5cc5519b4c99c25b9edc79cbb7d3f7bb743`
- [issue7816-indent_room_boundary-native-p002-overlay.png](issue7816-indent_room_boundary-native-p002-overlay.png) / SHA-256 `fe486dc017dee824353628ee474f3909423daf2a6b3b6df2bf286752c47decf2`
- [issue7816-indent_room_boundary-wasm-p002-review.png](issue7816-indent_room_boundary-wasm-p002-review.png) / SHA-256 `99d0264af52505878212d326a6e6bde4d6014299e6c817bf804fd929c57996a8`
- [issue7816-indent_room_boundary-wasm-p002-overlay.png](issue7816-indent_room_boundary-wasm-p002-overlay.png) / SHA-256 `00128c285c6ac76a5e5e05c422b8c470fe9202aef6b0e94e3d5d2a59f68ca38b`

### issue7816-odd_spacing_prev

- 입력: `samples/issue7816/odd_spacing_prev.hwpx` / SHA-256 `b1d3a82bda242971e0aae87a07fe0859089479024bc22d53368854c1b33770ed`
- 독립 한컴 Print: `pdf/issue7816/odd_spacing_prev-2020.pdf` / SHA-256 `ac80c85b8e3bc4af3dcc0669e1beaad1baefc82333adad28ee4b3af91867cc18`
- native 전쪽 TSV SHA-256: `93eae16d259133ba325d0bd8a387da9d788b179f67d9c55243800407d3a279f9`
- wasm 전쪽 TSV SHA-256: `11d745b242ae71c9c7772a8f2a229640c313b20498ea0ed246b2670a8b03fee7`
- [issue7816-odd_spacing_prev-native-p002-review.png](issue7816-odd_spacing_prev-native-p002-review.png) / SHA-256 `01d1b8167b6105a060bc712b6c7ed74c323e0b2995f56d971bbe6c777f9230f8`
- [issue7816-odd_spacing_prev-native-p002-overlay.png](issue7816-odd_spacing_prev-native-p002-overlay.png) / SHA-256 `ad0d8a4a8d6a1f514f4650b58bcf5fdbf31d382170e60dd70270b906bd42df8e`
- [issue7816-odd_spacing_prev-wasm-p002-review.png](issue7816-odd_spacing_prev-wasm-p002-review.png) / SHA-256 `cb1ee42c8df79ed739c7b2b57f190194964ed5fca70616a1a2f92035388a9ca4`
- [issue7816-odd_spacing_prev-wasm-p002-overlay.png](issue7816-odd_spacing_prev-wasm-p002-overlay.png) / SHA-256 `b163306f8d87d656d6f24dedf79fb451f31abc569b5f305a5c066dab5de7bb6e`

### issue7824-synth_ladder_spacing_before

- 입력: `samples/issue7824/synth_ladder_spacing_before.hwpx` / SHA-256 `b2027eab1a4209482c0047cccd00f13ad964ea5d6cf45a7f56aefd9ff41a380c`
- 독립 한컴 Print: `pdf/issue7824/synth_ladder_spacing_before-2020.pdf` / SHA-256 `59220c8ed402c4609da05122329fd6cd34526ecb9074b04bac7ce9b88375006f`
- native 전쪽 TSV SHA-256: `7075429af8588a3095be32f578954a9061b1593a54cf1248012d23d0d6d4d9a1`
- wasm 전쪽 TSV SHA-256: `60dd6467bee8e477b3c1e155db9c06beeac8accd7bd248b7e86673b814182c4c`
- [issue7824-synth_ladder_spacing_before-native-p002-review.png](issue7824-synth_ladder_spacing_before-native-p002-review.png) / SHA-256 `62a7d8236d097faeee1b281477e1c6ddd19f81d1b8c848aba581093baf42fffd`
- [issue7824-synth_ladder_spacing_before-native-p002-overlay.png](issue7824-synth_ladder_spacing_before-native-p002-overlay.png) / SHA-256 `a2e3ec5ad12014eeaeccb275764a818d4c15699a884a212a46b8054ba7c3a0ba`
- [issue7824-synth_ladder_spacing_before-wasm-p002-review.png](issue7824-synth_ladder_spacing_before-wasm-p002-review.png) / SHA-256 `0813ae1bd5aaaae37742717607a6f82785106f936809b3250ba8faffe345a24c`
- [issue7824-synth_ladder_spacing_before-wasm-p002-overlay.png](issue7824-synth_ladder_spacing_before-wasm-p002-overlay.png) / SHA-256 `0808facdc4a3f4e324ebff0a8a82c1cc912e36806b2393ada9bc414b918e6611`

### pr7669-reduced

- 입력: `samples/issue7095/36308670_split_row_declared_tail.hwpx` / SHA-256 `4982d53c16ba51c0366aea8eb9303d748a26750849f582fb47413e1a80979de4`
- 독립 한컴 Print: `pdf/issue7095/36308670-split-row-declared-tail-2020.pdf` / SHA-256 `42ad85ae601c832778e7501c57b6d10448bc7741c1e00f564beb344c6e88da02`
- native 전쪽 TSV SHA-256: `f07237b446f5126f90ce2a0c7618943b4a1803909c6bc5d12d96711be864d1e5`
- wasm 전쪽 TSV SHA-256: `76cd5c4e29adad6b01388a17d22739817512c1e6b119f7a1a935c79c8a4e73b3`
- [pr7669-reduced-native-p002-review.png](pr7669-reduced-native-p002-review.png) / SHA-256 `1f14ffe6404aa67b066bf2b57879425898de534de2d3ef8f5430b970f9d5f403`
- [pr7669-reduced-native-p002-overlay.png](pr7669-reduced-native-p002-overlay.png) / SHA-256 `be2481800c6f3220062c51b7977c75156a0fc3d0a6d0bb5147d3059a20597368`
- [pr7669-reduced-wasm-p002-review.png](pr7669-reduced-wasm-p002-review.png) / SHA-256 `b4ec15976c33f097ba3deee43d14624912a01d27a33634d791fdf8e9fb4ad951`
- [pr7669-reduced-wasm-p002-overlay.png](pr7669-reduced-wasm-p002-overlay.png) / SHA-256 `c3f4e8f52b5d79323f79f5f520da1b5b4e2302ae2a481323d91b66262c67c7fe`
- [pr7669-reduced-native-p001-review.png](pr7669-reduced-native-p001-review.png) / SHA-256 `86249dfd9950354d65b101255062fa00704ffc8d83c9c15149ea1198ebd0fa06`
- [pr7669-reduced-native-p001-overlay.png](pr7669-reduced-native-p001-overlay.png) / SHA-256 `b475b284d14e7b59365a45be5f11fc19dfa8dca1f684af3a9147c7dd4fe6530e`
- [pr7669-reduced-wasm-p001-review.png](pr7669-reduced-wasm-p001-review.png) / SHA-256 `5290ce56dbd63a5f77aef92afc8e84b5ec385f063f0284d9a2399ff6d322c0eb`
- [pr7669-reduced-wasm-p001-overlay.png](pr7669-reduced-wasm-p001-overlay.png) / SHA-256 `6df2199283073e8acde09f544dba00d1f332d0a5a4052fdd459f48fa6d456c8b`

### issue7816-indent_align_matrix

- 입력: `samples/issue7816/indent_align_matrix.hwpx` / SHA-256 `42c3586ce8b73ee156687d18840fec13f1ca72275fc7c82c982e5a9c82ed04c1`
- 독립 한컴 Print: `pdf/issue7816/indent_align_matrix-2020.pdf` / SHA-256 `2bf812fec7a213b60f919621645f0b2ece6b47de05e64a48d792bdb08158515a`
- native 전쪽 TSV SHA-256: `40da65504bb9d7dda86c8f5d4a08533fe084db9be9b89bae69748633b7c73e65`
- wasm 전쪽 TSV SHA-256: `6875647c959b18e3f2e1db759ac05240cee10fba1e0b368d4870a5af15a6442f`
- [issue7816-indent_align_matrix-native-p003-review.png](issue7816-indent_align_matrix-native-p003-review.png) / SHA-256 `41cae7bb052f518039b2cf9888f7070c3f010a8f9123824ec3a121d9e237824a`
- [issue7816-indent_align_matrix-native-p003-overlay.png](issue7816-indent_align_matrix-native-p003-overlay.png) / SHA-256 `2089bc3e12bd0b724dcbdf525022d60b7913f65b63f60a10d5a0563787b2f719`
- [issue7816-indent_align_matrix-wasm-p003-review.png](issue7816-indent_align_matrix-wasm-p003-review.png) / SHA-256 `00bf63c65c42f6e2f1447cda76ebb87f06fc40ab7439b32b2d15c46a56e3b4a4`
- [issue7816-indent_align_matrix-wasm-p003-overlay.png](issue7816-indent_align_matrix-wasm-p003-overlay.png) / SHA-256 `f0b255cf4ff729ae5908f3bd4dd6686361dcb6a0a94b965a73cc3d80f7d1e709`

### issue7816-odd_indent_ladder

- 입력: `samples/issue7816/odd_indent_ladder.hwpx` / SHA-256 `ba5c6c63bece099ac84295dd35c79e8d0f6be3d7a333936374715d2151f80d8a`
- 독립 한컴 Print: `pdf/issue7816/odd_indent_ladder-2020.pdf` / SHA-256 `d31386ce63b36915e61d4cb7b72d5307dcd3c9a43e9a28c2ce41712cb5cc2417`
- native 전쪽 TSV SHA-256: `c487bfd42105fcd304117c1821052750ea6d4ef60fa594044ff92d3e720827a6`
- wasm 전쪽 TSV SHA-256: `00a4a89e77567056cc3045c22aaed9643a6786ab4c86833c3a2ae3d5656c45d7`
- [issue7816-odd_indent_ladder-native-p003-review.png](issue7816-odd_indent_ladder-native-p003-review.png) / SHA-256 `112939aa919ca6ad7d0b7e03e8bf2c7f5079e8a4afc1b8ba0ab77054a890a147`
- [issue7816-odd_indent_ladder-native-p003-overlay.png](issue7816-odd_indent_ladder-native-p003-overlay.png) / SHA-256 `45850880800153ab1e0978aa3ea4815d8f8df3b51a02e8835de0a0a4041a0d2b`
- [issue7816-odd_indent_ladder-wasm-p003-review.png](issue7816-odd_indent_ladder-wasm-p003-review.png) / SHA-256 `f17b9f83e23af87772885c0ca37b7b5468fc3f0def3663437c47cb4025fca007`
- [issue7816-odd_indent_ladder-wasm-p003-overlay.png](issue7816-odd_indent_ladder-wasm-p003-overlay.png) / SHA-256 `3651b848e4ea2093a9905131521b67455565d5794fe39617ad51d4c786dd5e61`

### task1749-original-hwp

- 입력: `samples/task1749/saved_bounds_cumulative_page_break.hwp` / SHA-256 `49478ec2a01315594bcc5bf0bbbdbc43b86c134cefba3c979d617180bfd43f81`
- 독립 한컴 Print: `pdf/task1749/saved_bounds_cumulative_page_break-hwp-2020.pdf` / SHA-256 `0b219907dc9fcaf34175ee70b59c3124c1fcd76bd0613e49fcf7529cec6e6dc9`
- native 전쪽 TSV SHA-256: `9124328713e9d3ebe382d5645c33e5a199bb23101b1ec6bf0f833bb5c82b90d1`
- wasm 전쪽 TSV SHA-256: `7cde32ef5f56cf8e7a5312f95741a6018f4229cc3660c5d38c99afebd702afd5`
- [task1749-original-hwp-native-p001-review.png](task1749-original-hwp-native-p001-review.png) / SHA-256 `55b605de32da5d75057590cf9afeb8c5f4a443789cc1a73d4c7798cd0e36a5e0`
- [task1749-original-hwp-native-p001-overlay.png](task1749-original-hwp-native-p001-overlay.png) / SHA-256 `5d5e53e6b3448bdad0a1df665d3db6c5ce0a9c1a2a9e9d25560c3d9b09ceb2e2`
- [task1749-original-hwp-wasm-p001-review.png](task1749-original-hwp-wasm-p001-review.png) / SHA-256 `cc2c73a34224b04ba1789d4798c58a38e1423c5d2cd8734c82401245e42fa894`
- [task1749-original-hwp-wasm-p001-overlay.png](task1749-original-hwp-wasm-p001-overlay.png) / SHA-256 `06ea53e4503c91da99059cd71240306299eac2b9e8adf9895782f4eb9b24f3b0`
- [task1749-original-hwp-native-p002-review.png](task1749-original-hwp-native-p002-review.png) / SHA-256 `8f3b01873f12e3b90860cfc125cc0fa36e6eb657dd57ee64fe76d5a623774384`
- [task1749-original-hwp-native-p002-overlay.png](task1749-original-hwp-native-p002-overlay.png) / SHA-256 `72c43d21c35b900c9fc72b0969ad0cf096f32b87a1ad874170e07b3b0f57c69a`
- [task1749-original-hwp-wasm-p002-review.png](task1749-original-hwp-wasm-p002-review.png) / SHA-256 `648888f1b189fca4961020ea78d239dc1d26df3232da0ed4176459c14fdc50a2`
- [task1749-original-hwp-wasm-p002-overlay.png](task1749-original-hwp-wasm-p002-overlay.png) / SHA-256 `99805c744532e0b4722e6963943b96a40d3e7a163c6ab42dcb6faac9570f16a8`
- [task1749-original-hwp-native-p005-review.png](task1749-original-hwp-native-p005-review.png) / SHA-256 `853ed2e57e7ef805c15762f5bba58f9d5b1bd701d3e843ab4341dace8d27461c`
- [task1749-original-hwp-native-p005-overlay.png](task1749-original-hwp-native-p005-overlay.png) / SHA-256 `0b27c76e7a502a5a6ce9d7e96e5a7fe233e621cc93284af77d13ff986f283e66`
- [task1749-original-hwp-wasm-p005-review.png](task1749-original-hwp-wasm-p005-review.png) / SHA-256 `45fc8048233a6da41617b37f6c9335c2232e1d6cea5210957c1fa3a32bf88b66`
- [task1749-original-hwp-wasm-p005-overlay.png](task1749-original-hwp-wasm-p005-overlay.png) / SHA-256 `3307b7ca805a7dd5811e7b69a18b17473a67ecf73763d050f7e3027ff1b303f4`

### remaining-franchise

- 입력: `samples/issue5966/1130000-202100008_franchise_review_report.hwp` / SHA-256 `a1b8299b2554a83873b780474defb02efa949b43c86c63bcc164967f96091e55`
- 독립 한컴 Print: `pdf/pr_6088_6144/hancom2020/pr_6088_6144_issue5966_franchise_review_report_1130000-202100008_franchise_review_report-2020.pdf` / SHA-256 `4cf22800e29baddb9b84023462c75252efd4efb04d22e9e64a8e3d9313076009`
- native 전쪽 TSV SHA-256: `989f4578538426660304cfcc54054b991d6472c6fd14ed670a7276eca7550271`
- wasm 전쪽 TSV SHA-256: `a7221bb20326a580d5e03004f49a4525010d078ec37b44031d54af5618d8ed56`
- [remaining-franchise-native-p057-review.png](remaining-franchise-native-p057-review.png) / SHA-256 `82e5c2c107bf7eeab474d42ce6d91d6974546d14d5812bb67c2716d4fcc5fc67`
- [remaining-franchise-native-p057-overlay.png](remaining-franchise-native-p057-overlay.png) / SHA-256 `9c71e9f45214ddbc8ba6f8197f6fe21d2c2592b1ca91c4537ce464a9257b732d`
- [remaining-franchise-wasm-p057-review.png](remaining-franchise-wasm-p057-review.png) / SHA-256 `986388f0a334bd53f963e4b34da1a18eac5fd702856c6d8865f58bc37a87a411`
- [remaining-franchise-wasm-p057-overlay.png](remaining-franchise-wasm-p057-overlay.png) / SHA-256 `6648c7b75064e79bb10b974f96f88733ab2313c039bb9cae9049aff5c046fd60`
- [remaining-franchise-native-p071-review.png](remaining-franchise-native-p071-review.png) / SHA-256 `23a640649ea79b109a456da73670484dcc6a00ea6e7a38da7eab609c333765cc`
- [remaining-franchise-native-p071-overlay.png](remaining-franchise-native-p071-overlay.png) / SHA-256 `05b32100de971d8361a5349d0c8ca040e42381f0bdf1e1766f819a8bee2aa904`
- [remaining-franchise-wasm-p071-review.png](remaining-franchise-wasm-p071-review.png) / SHA-256 `7f64110af05a5854d7dceaa5da764b8cec2a1a13ad4961880f24c144d0df0702`
- [remaining-franchise-wasm-p071-overlay.png](remaining-franchise-wasm-p071-overlay.png) / SHA-256 `fb87d23d2598291d7057957591bb4660795cb04464bb0f28f2148cddff09b69f`
