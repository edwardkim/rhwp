# OSS 결과보고서의 끝 표 커서 보정

원본 `samples/2026_oss_rst.hwp` SHA256
`b3a43578fc441d9668d06cd55c0cd267332ecb087e47ada4c5ac456af2afc78b`.
독립 한컴2020 Print `pdf/hwpx/2026_oss_rst.pdf`는6쪽이며 SHA256
`bec53a601cc7714a40ca340d26f971d1ab49eeb355682fc9db7b15cd5e04c86e`.

기준 base4a07 Native6쪽 최저96.34424%. 후보270 Native6쪽 최저95.88324%,
fresh WASM273 Canvas2D Print6쪽 최저96.56689%, 전쪽90미달0.
3쪽은 양쪽99.99680%. Native1/4쪽은 base 대비0.68234/0.15572%p 하락이
기존247에서도 동일하여 별도 보정이 필요하다. 이 자료를 전체 PR 수용으로 쓰지 않는다.

192DPI, compat2022, Print, 현재 canonical PDFium smooth_rgb_v1, raw2px90%를 유지했다.
3/4쪽 review와 overlay를 직접 열어 마지막 세 행·표 종료·바로 뒤 SBOM 구역을 확인했다.
옅은 본문/AA 차이가 남는다. before247의4쪽은 항목 없는 빈 쪽이며 쪽수7/6 불일치
실패 증거다. 후보의 내용을 앞으로 옮겨 비교한 수치를 정식 same-page 통과로 쓰지 않는다.

Native producer270 CLI SHA256
`e3ca21c51179d5f4045ed4753e7f3282a19b1c089ae39ea06173a1e15b35239c`,
fresh WASM273 SHA256
`4ea927016261d9e3d4ad99d2472e018d50b84eed7102ce71d2d4229ede661b91`.
WASM은 Mac locked `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며
Docker 최적화 결과가 아니다. producer/source/실제 초기 응답 hash와 로그는 ignored
`output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair270*`,
`repair273*`, `repair280*`에 있다.

## 원본 바이트 감사

| 파일 | SHA256 |
| --- | --- |
| [base-native.tsv](base-native.tsv) | `67d7243e4b8e93f992677cfd7eddbc5480315d27979f8b4121db8a85c5251924` |
| [candidate-native.tsv](candidate-native.tsv) | `1745cce690ae782587846206f300c77afde42f517204e512df01ec4509ecf2b3` |
| [candidate-fresh-wasm.tsv](candidate-fresh-wasm.tsv) | `71345c81edd450c84944684333ade1202d1fd39b895ac9aeb087485818b68280` |
| [native-p003-review.png](native-p003-review.png) | `17e1b73b22d810f734005c6aa073a5529255a9b2a1c1c55fb658b9ba082275ca` |
| [native-p003-overlay.png](native-p003-overlay.png) | `a1c0a922692d932e254b0d25e36608bc87b22e44761ebb1eb0f8dc4c1ab7af78` |
| [native-p004-review.png](native-p004-review.png) | `67fd820a78dc1e3eaebd5bb2712d47fbaa9c8a948568409116b25c7d8cdbe652` |
| [native-p004-overlay.png](native-p004-overlay.png) | `e381fa1cffbac1bc0d1880670d3edd5f867280a021f5631cc6b402d2b41157fa` |
| [wasm-p003-review.png](wasm-p003-review.png) | `13e8ef49904f4f8f41b66626d46cd821ff5b5df58b1e6fe885d81fb398423d2d` |
| [wasm-p003-overlay.png](wasm-p003-overlay.png) | `c7268221bdc182e72caf7d4e36ba9b2b3b89fe331453b2deee7582aaed9f9659` |
| [wasm-p004-review.png](wasm-p004-review.png) | `ae2aadcf09355b6f54f984ee550d02754e50757371069daf0091f4dbcd5d2c9d` |
| [wasm-p004-overlay.png](wasm-p004-overlay.png) | `7369c8e2eda128fd6fb883fc31177dde170f46b5abf437414ddcffb9d99c7edf` |
| [before-native-p004-review.png](before-native-p004-review.png) | `fb5fe887764bfc331a7ef55df649e3408af5d29e3fe9b6df0267eca6344d5163` |
| [before-native-p004-overlay.png](before-native-p004-overlay.png) | `b08b68f3a6694f0057eb2ef0dee8991288a1d092e1cbf004e4b92721ab298b79` |
