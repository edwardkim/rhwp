# PUA 점선의 흐름 판정과 실제 표시 분리

원본 접수증은 `samples/issue7017/2528375-science-invention-contest-receipt.hwp`, 독립 Print는
`pdf/2528375-science-invention-contest-receipt-2020.pdf`(SHA256
`ee0f1312284ce04c683b69cbdfd70157760ff42ea57a2a864a1d3aaf93ff8e3b`)다.
대조군 `samples/복학원서.hwp`와 `pdf/복학원서-hwp-2020.pdf`도 원본 바이트를 보존했다.
각 문서는 전1쪽이며 before270/273과 after291/293 Native/실제 WASM Canvas2D를 비교했다.
192DPI·compat2022·Print·canonical PDFium smooth_rgb_v1·raw2px90%를 유지했다.

| 문서/엔진 | 수정 전 raw | 수정 후 raw | 변화(%p) | 쪽수 |
| --- | ---: | ---: | ---: | --- |
| 접수증 Native |97.75291|99.35164|+1.59873|1/1|
| 접수증 fresh WASM |97.78710|99.37179|+1.58469|1/1|
| 복학원서 Native |96.94293|96.92177|-0.02116|1/1|
| 복학원서 fresh WASM |97.70069|97.68584|-0.01485|1/1|

전쪽90미달0이며 대조군 하락은0.1%p 이내다. 모든 after review/overlay를 직접 읽었다.
접수증의60개 점선과 절·취·선 중심을 복구했고 표·본문·원문 소속을 보존했다.
접수증 Native 변화4222픽셀은192DPI `[291,859,1311,888]`에만 있다. 복학원서는
121픽셀 `[113,2062,154,2065]`만 달라지고 몸통/서명/표는 그대로다. 이 변화 상자는
진단이며 회귀 기대값으로 고정하지 않았다. 복학원서는 왼쪽 위 로고 등 기존 독립 PDF
차이가 남으므로 내용·픽셀 완전 일치나 새 px 조건 충족을 주장하지 않는다.
색상 경계 보정값98.01169%를 raw97.68584% 대신 쓰지 않는다.

접수증만 양쪽98% 이상·전쪽90%·직접 판독을 충족하여 기존7017 검사의 원문·순서·점선60개·
쪽 여백·가운데 정렬 관계를 유지하고 독립 PDF 라벨 x/기준선±2CSSpx를 추가했다.
PDF의 `/INPILL+Batang` Tm x2087/2435/2784, y2763과 device CTM
(0.119935, -0.119869)를 pt→96CSSpx로 변환한 기대값이다. 현재 출력 좌표를 복제하지 않는다.

Native291 CLI SHA256 `01a06bb072784d9ba41d612813fd667034146fc2417ad678deaf022a0b50516c`,
WASM293 SHA256 `1099f24c5b3956ac4669f4ee6413ea8d0589efbcd46ab8e96e58b406eee293eb`.
Mac locked `--target web --out-dir pkg --no-opt` 로컬 대체 빌드256.345초/exit0이며
Docker 최적화 결과가 아니다. root pkg/Studio public/실제7736 초기 JS·WASM 응답 hash와
초기화도297에서 확인했다. 문서 CanvasKit/actual36 검증 완료를 대신하지 않는다.

정본 producer/로그/원본 PNG·JSON·SVG는 ignored
`output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair289*`~`repair300*`에
있다. 재사용한 이전 head의 통과가 아니라 실제291/293 출력을 새로 산출한 결과다.
그림·서식의 잔여 AA·미세한 자형 차이와 전체 회귀/다른 문서 미해결은 남는다.

## 원본 바이트 감사

| 파일 | SHA256 |
| --- | --- |
| [before-native-receipt7017.tsv](before-native-receipt7017.tsv) | `26886d27caef32d05a87bc7cab63a16a5fd0cfa7ad3d542fc698dd00c219e1fd` |
| [before-native-receipt7017-review.png](before-native-receipt7017-review.png) | `5d3c57f632b036f86e16c461d7536b00dbeb0cd333e72a1e623573fe6a2a742c` |
| [before-native-receipt7017-overlay.png](before-native-receipt7017-overlay.png) | `70225845395a66b7b8a51ab4d2fe163cc25791c4be571bcb7f52708da898c363` |
| [before-native-return937.tsv](before-native-return937.tsv) | `d0d4ae7b4b99ea6cf08699c6131cb3e84c248da99663b66819ddc38b06be047d` |
| [before-wasm-receipt7017.tsv](before-wasm-receipt7017.tsv) | `e5020930789764d9615c8b9b50d4a0f1e287a1c7a760b01b0f2decd21ddeb0b2` |
| [before-wasm-receipt7017-review.png](before-wasm-receipt7017-review.png) | `93da972f53ff360fef5ae98630731178e6ecb56b0b7555bdceafe91012ba0724` |
| [before-wasm-receipt7017-overlay.png](before-wasm-receipt7017-overlay.png) | `2bf9f3932b4fb267c9cbf751b7fe2dc54b8b939b9570014a255b6d6e4e5d3d2f` |
| [before-wasm-return937.tsv](before-wasm-return937.tsv) | `1e4544a0746f50e0b88fcb1a26d0918c1deb2544f06a312c57f594392b0533e2` |
| [after-native-receipt7017.tsv](after-native-receipt7017.tsv) | `7579789c6f2d41760f17a50173b0e72080d1c4693a98fbcacb850b0819d2c1b6` |
| [after-native-receipt7017-review.png](after-native-receipt7017-review.png) | `5a57dc9bbe7c1953e0f8eb0d52dd0d39ed9468bc312e59ee3632a72c6e1ab888` |
| [after-native-receipt7017-overlay.png](after-native-receipt7017-overlay.png) | `eeb1eedf45f4deeff31a958cc30f33617675d29f4219b36b5e72dcf1e316a4fe` |
| [after-native-return937.tsv](after-native-return937.tsv) | `cbde07b171dc7eb18de85634f96771228d74805ef6a23542f88a4fa6a52c0463` |
| [after-native-return937-review.png](after-native-return937-review.png) | `206486f78fa59cbe5d55dcb5a393b0d432bd5b284d2833b232a9276cdfeaf404` |
| [after-native-return937-overlay.png](after-native-return937-overlay.png) | `e82cd0ac3bc4b1e75d938f2bb63c68114796ba1baa897a7e43e78a51e7afbddb` |
| [after-wasm-receipt7017.tsv](after-wasm-receipt7017.tsv) | `0c3acf6192dcaa09b173277ba8cc6519e75687e396b03a3e8a8cf8ba353f4c10` |
| [after-wasm-receipt7017-review.png](after-wasm-receipt7017-review.png) | `f474f7712bb26afbf20d6c09a10055901c01d3123abfc0035d96576e7df88904` |
| [after-wasm-receipt7017-overlay.png](after-wasm-receipt7017-overlay.png) | `03c1addd1b8b9393eb9340a4f5fe2743c2ebf4380361f03743ec55e57e1c758f` |
| [after-wasm-return937.tsv](after-wasm-return937.tsv) | `494bbcb21dba686e7a2031048afa1371e345a4ec2ee4dc308ade2e53c1cddc7f` |
| [after-wasm-return937-review.png](after-wasm-return937-review.png) | `62fdb5a402846d6f0b348ede30b3d507b7a7688a944132f415da1dd1739a98c4` |
| [after-wasm-return937-overlay.png](after-wasm-return937-overlay.png) | `91ace02be40fd0fd43de95a3e0b1ff96b723127677c1a26f7c5fc1eada1db015` |
