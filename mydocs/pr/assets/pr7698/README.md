# PR #7698 비교 이미지

검증 code: `c2f0bad49959b853a100458b3d74810c37d0b385`. Native/fresh Docker WASM의 HWPX 2·3·4쪽 review 및 standalone overlay다. 원본은 공개 #5169 HWP를 한컴에서 저장한 HWPX이며 제공본과 header/section 바이트가 동일하다.

입력: `tests/fixtures/issue5169_tracked_visibility/issue5169_viewtext_changetracking.hwpx` (SHA-256 `1c822a1c9962eadf9ae4a68206051c7d1474f811d5401074fbb05de309f36b26`). 기준: `pdf/issue7674/issue5169_viewtext_changetracking-hwpx-2020.pdf`, 한컴 2020 Print job `9d83c530-a806-405a-8a10-216a591b8d26`, method 0 / build 11.0.0.9136.

전쪽 TSV/로그는 ignored `output/pr-review/pr7698-20261011/maintainer/compatible-{native,wasm}/`에 보존했다. 두 경로 모두 6쪽이며 최저 90.87964%; 누락·90% 미달 없음. 대표 gate는 passed. 메인터너는 Studio 직접 시각 판정 통과를 제공했다. 글자 폭·자형·미세 자간과 괘선 모양의 차이는 남아 있어 픽셀 동일로 주장하지 않는다.

WASM SHA-256: `3988720b9da0d2fac1a3a59d612a8c291950d0e2a6669aa008cdbd70166e3a4a`. 원본 폰트·임베딩 SVG는 공개 자산에 포함하지 않는다.

| 파일 | SHA-256 |
| --- | --- |
| hwpx-native-overlay-002.png | `248241126919fdbc24f9ccb443141c66f44ae1c5e130205a5b966283a55fa68b` |
| hwpx-native-overlay-003.png | `dafa426df4110a4bc36676b487629bab5d28113908335a154e0f116a0f822ce4` |
| hwpx-native-overlay-004.png | `736df417d6cbe5a804fe800d54d9e0bfa029416c4b00a3d0755e9cee6e37df12` |
| hwpx-native-review-002.png | `2491c848fe1416e71fed0b6c3c95532cf455b60f1293706a4f4abd4d09027dff` |
| hwpx-native-review-003.png | `9da26715bd86886de3a60ca72b43e1ce5bbb1d3ab518c411f184858c89aa7a21` |
| hwpx-native-review-004.png | `b5eac24cac66fccfcf9e25d2bbc008ec11349254a62a47aa334f76a343a3f220` |
| hwpx-wasm-overlay-002.png | `2fc5fa12a6030e749e01a0c648c61c2b8cc1a28ec4aae396620915019b11eec9` |
| hwpx-wasm-overlay-003.png | `4075e42b514a69adfe8af0fe0352aee8d381d6163c86ac6918f600a45fb813cc` |
| hwpx-wasm-overlay-004.png | `7eb3dc1d59e79fe78530adc195dc45af9b0a4c56229eab1ef8c9a782493850f0` |
| hwpx-wasm-review-002.png | `116712a65b7fc4033641f5c8be3d5de79ca09c42346ff7c56000535e110abf18` |
| hwpx-wasm-review-003.png | `3b88c34c52db986a97d1d91a0d00e5d6d0673167ccd19354666f239d3c10d6b3` |
| hwpx-wasm-review-004.png | `89cb50627a7d70481297f0b67367720e1cab2441a7210e679769b2f5dccd36b4` |
