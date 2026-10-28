# #1892 원본 도형·공개 저장의 좌표와 표시 의미

원본은 `samples/issue1892_hwp3_drawing_group_roundtrip.hwp`다. 이전 후보와 고정 devel,
최종 후보의 전2쪽 TSV를 원문 그대로 보존했다. 대표 이미지는 내용이 있는1쪽의 review와
standalone overlay다. 두 번째 빈 쪽100%를 핵심 위치 보호용 페이지로 고르지 않았다.
수치는 색상 경계 보정 전2px raw 실루엣이며 엄격 RGB 일치율이 아니다.

## 실행 조건·출처

- 고정 base: `4a07d3ae4dd3255a57fb0166b2b2a2dc335a61fa`.
- 이전 후보: `93d9cd8e596f27c6ce170c2ce55ddf45b6ed1b62`, Native361/fresh WASM362.
- 최종 source: 같은 parent에 이 커밋의 `shape_mapping.rs`와 `shape_layout.rs` 변경을 적용한
  Native393/fresh WASM394. runtime source patch SHA-256: `362a6267baba2a217610c88c8ab20fde29809ebb84cb95b07f1e60ba84aa9050`.
- canonical `visual_sweep.py`, PDFium smooth_rgb_v1,96DPI CSS px,compat2022,Print profile,
  capture 최대8. Native는 실제 Skia PNG, WASM은 실제 fresh Canvas2D PNG다.
- Mac WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
  Docker 최적화 결과가 아니다. 실제 root package 재컴파일과 root pkg/Studio public hash를 확인했다.
- Native CLI SHA-256: `4a2f63a7423c3f0c87c715af569a9a9f4788ca3386cd7bca1446a94bbf9bc0a0`; WASM SHA-256: `174ef06e842525682c71e2368c755f327890cd9899964c16ddef6effd4aa4aa2`.
- Studio7736 초기화의 실제 JS/WASM 응답도 같은 frozen 빌드로 확인했다. 초기화 확인은
  문서 paint·CanvasKit 준비·actual36의 수용 결과가 아니다.
  JS 응답은 Vite URL rewrite를 검증한 일치이며 raw 파일과 응답 hash의 동일을 뜻하지 않는다.
  WASM 응답은 frozen 파일과 SHA-256이 동일하다.
- HWP3 원문 HFT 출처에 대해 기존 public Batang/독립 ASCII 공급과 NGULIM/사용자 폰트를
  같은 순서로 공급했다. 전체 파일 hash는 ignored producer에 고정했다.
  TrueType78494의 별도 글꼴 선택 계약에 이 HFT-only ASCII 프로그램을 적용하지 않았다.
- fixed WASM387은 현재 실제 `fontEnvironment:null`과 같은 조건을 명시했다. 구형 base에 없는
  자동 선택 API의 실패384/386 WASM 산출물은 사용하지 않는다. base Native384는 유효하다.

## 전쪽 결과

| 역할 | 쪽수 | 1쪽 raw % | 2쪽 raw % | 90% 미만 |
| --- | --- | --- | --- | --- |
| before-native | 2/2 | 98.83706 | 100.00000 | 0 |
| before-wasm | 2/2 | 98.83714 | 100.00000 | 0 |
| fixed-native | 2/2 | 99.38299 | 100.00000 | 0 |
| fixed-wasm | 2/2 | 99.38301 | 100.00000 | 0 |
| after-native-original | 2/2 | 99.98735 | 100.00000 | 0 |
| after-native-public-saved | 2/2 | 100.00000 | 100.00000 | 0 |
| after-native-none-curved | 2/2 | 100.00000 | 100.00000 | 0 |
| after-native-visible-curved | 2/2 | 99.94939 | 100.00000 | 0 |
| after-wasm-original | 2/2 | 99.98735 | 100.00000 | 0 |
| after-wasm-public-saved | 2/2 | 100.00000 | 100.00000 | 0 |
| after-wasm-none-curved | 2/2 | 100.00000 | 100.00000 | 0 |
| after-wasm-visible-curved | 2/2 | 99.94939 | 100.00000 | 0 |

original의 최종 후보와 fixed devel은 같은 원본/PDF·글꼴·도구·DPI로 페이지별 대조했다.
어느 쪽도0.1%p 이상 후퇴하지 않았다. 원본→저장본의 입력 hash가 바뀐 사실은 아래에 공개한다.
public-saved/none-curved/visible-curved는 새 정확한 입력마다 독립 한컴2020 Print PDF를 받았으며,
기존 원본 PDF로 수동 편집본의 정답을 주장하지 않는다.

| 입력 | HWP SHA-256 | 독립 Print PDF SHA-256 |
| --- | --- | --- |
| original | `ab6bb4e80c4b0f4228e62bbb3d180229e5cef5f52b20fddee8849bf1c7b4c438` | `bde778b26c91635f14bf14272eca8b9199da7be7f9d4e0750b7402768b640bc9` |
| public-saved | `d067d6b0eecafd2c28d1a1c622b7fad7cd67c45a92d95c3b9228a1ace4d52757` | `f53352f3203754fbc2914c6b89b4b330b081ccafec499fec613bec8d6a9f764c` |
| none-curved | `73aa516280ce21a7a85f1694ec63a282f328b03bb252ed4383a8a1d4ccaa6bcf` | `9c2833d3dd5f527838967edf6a099a32c96280ffd91a5bc2609d45b7d85ffbb5` |
| visible-curved | `8644cdffbec7ff73857aa1918dcb2843af6bcfbfba60a80c64255773f0e2997e` | `603a22a977afc88b22ab684127a5d16abd28e8bb81c405050ae4783c618f9628` |

추가3개 PDF는 `pdf/issue1892-controls-20261020/`의 `source-list-attr-c451` 이름으로 보존했다.
engine2020,Hancom11.0.0.9136,direct DLL host,preprocess none,Print method0,각2쪽이다.
공개 저장본은 원본의 `export_hwp_with_adapter()`로, 두 연결선 대조군은 정식 테스트의
`curved_visibility_variant()`와 같은 source-owned 편집으로 생성했다. 이전 raw serializer
진단 입력의 한컴 개방 거부를 우회하지 않았으며 해당 실패 입력은 새 기준에서 제외했다.

## 직접 판독·회귀 계약

원본/저장본과 선 없음/가시 연결선의 Native/fresh WASM1쪽을 실제 열었다.
248mm 치수선의 길이·끝점, 비표시 동반 개체의 소유 보존, 글상자 가운데 정렬과 단색 면,
가시 연결선의 원본 SPEAR/OpenArrow와 시작 화살촉 부재를 확인했다. 이전 후보에서 보인
중복 축척·수평 hatch 띠·임의 Triangle 화살촉을 보정했다. 가는 획·화살촉 끝의 작은
래스터 경계와 글자 획 차이는 남아 있으며 RGB 동일이라고 보고하지 않는다.
모든 입력의2쪽은 실제 PNG/PDF 잉크가 없는 쪽이다. 쪽수·본문/표 순서·누락·중복은
정식 관계 검사와 함께 확인한다.

원 이슈1892→PR1905(489bd3829f/merge23aea574ac)→2993cd35ed/0bbb7c366c 및
현재5c2dd53c64의 처리 의도를 대조했다. metadata offset을 rendering translation으로
승격하지 않는 계약을 유지한다. 누적 offset을 실제 구조 묶음 원점으로 재요구하던
기존 기대값은 독립 재저장과 충돌해, source offset/affine/깊이/순서·실제 출력의 관계로 고쳤다.
248mm 선은 PDF path의 실제 화살촉 끝점을96DPI CSS px로 환산한 독립 기대값 ±2px로 보호한다.
같은 기능의 public-saved1쪽은 양쪽100%, 원본1쪽은 양쪽99%대이며 두 경로를 모두 검사한다.
빈2쪽보다 실제 도형이 있는1쪽을 선택했다. 기타 HWP3 탭/널 확장 표식 계약은 유지한다.

이 자료는 #1892 경계 보정 증거다. 전체 회귀/시각 corpus·유입 PR 기능·actual36 및 다른
미해결 항목의 PR 완료를 의미하지 않는다. PR 제출은 사용자 별도 승인 후 진행한다.

## 산출물 SHA-256

- `before-native-whole.tsv`: `aedc673b938b89c55d708881dd70852fd399c454b4438ad881c39d9a7dc10b56`
- `before-native-p001-review.png`: `7118744c5e8981c4f787f0d03c49f96ea6e61863ce6c2b80646520d3d471a0ed`
- `before-native-p001-overlay.png`: `95512eed6ba4f162b9934832afaeca269caf73d9d2472508076f814bdf09bf2a`
- `before-wasm-whole.tsv`: `3392e6c2a458c6e960ae1ce7f298d56d2ff405bcf5c8ec9dcba9785bc35d1d84`
- `before-wasm-p001-review.png`: `20da4995b8f4181ff320cd0020193868e6553fba3e4bc993c996eb50aadee8f2`
- `before-wasm-p001-overlay.png`: `fcf572d4700b75c38e42b3cad76773054ec6d2b1c408afbb5135f7ad3e7769bf`
- `fixed-native-whole.tsv`: `c74ca803613645278eee1c995973bad40eb424766e1909096ae2270a8d71b023`
- `fixed-native-p001-review.png`: `ef2346d09ff695fe03c75fa17445374e56a1ac17f9337940f1bd51bf92f6d61c`
- `fixed-native-p001-overlay.png`: `f73dfc49d7597dd491667fbd02a90b9d0275dd28ffac7086b17bb9941978c711`
- `fixed-wasm-whole.tsv`: `f3575460ec50a6a2953c497ea51237d5490e283a962e0d0c5467d6e51b0fcf84`
- `fixed-wasm-p001-review.png`: `d85cd2764d58563a60fa07f1fe7ba6e261eafd97cbc4043ab52eef266592004e`
- `fixed-wasm-p001-overlay.png`: `80d0f54a852064c26dd04ef093a6785ee22e769c915d6d26ef9557fa43ba8ce8`
- `after-native-original-whole.tsv`: `fcc74d3b7d7584230d6402f045eb3d38604d83a3a2b504b60bcf786c39d591a5`
- `after-native-original-p001-review.png`: `746080e479c2bfa7a96c8b47c9e5b439883913577d7fa5ec690e165e3fa2cef8`
- `after-native-original-p001-overlay.png`: `90c208ec59ea7a1e4032ce4214ba5504e8010031849f7a80478d9fc2fbbf80ba`
- `after-native-public-saved-whole.tsv`: `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c`
- `after-native-public-saved-p001-review.png`: `532465647498e2835ecbc0398354445712672ec982491807def409f60d4df4a5`
- `after-native-public-saved-p001-overlay.png`: `eda9d38c17982cc5a2c50df2747ba1cfa256b510dd8fb0bf01180eb5e381a15e`
- `after-native-none-curved-whole.tsv`: `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c`
- `after-native-none-curved-p001-review.png`: `f73871f7ed156dd387aa65f24fecf01544484f7d75543b2cc8dcf4e7d28af619`
- `after-native-none-curved-p001-overlay.png`: `9e956fa8c7ece3cb21cf7f1febc5c3069d9950c981066860466b7717f768faf6`
- `after-native-visible-curved-whole.tsv`: `6320ec7d4da6e6c6930361502985a7154bf87de99f06a4697655ab5a8648f99d`
- `after-native-visible-curved-p001-review.png`: `ae36d96234443f779d5217bc82edd756b9700a6ced5ecd7e5d77f10bbe5b4ae9`
- `after-native-visible-curved-p001-overlay.png`: `ffdaaf86f8baccd3e5ccc239c0e0e4af1afc1c4f3ca07dc1111337d12bf2ecbc`
- `after-wasm-original-whole.tsv`: `fcc74d3b7d7584230d6402f045eb3d38604d83a3a2b504b60bcf786c39d591a5`
- `after-wasm-original-p001-review.png`: `c398b0872cb6ece817cca1443e3ddef07747b506b65e629c1916fbed79a4a14a`
- `after-wasm-original-p001-overlay.png`: `bcb7a245925a5464d43854fc7af112badb5ad65bc60910f8367679e5a8da307f`
- `after-wasm-public-saved-whole.tsv`: `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c`
- `after-wasm-public-saved-p001-review.png`: `9495fdb54de81fee59ea77581ee5e08cc835c27bbbdf35674bd9da2add8b451b`
- `after-wasm-public-saved-p001-overlay.png`: `be05dac4140948e10ce889a85889b85b178e52452f503cae01b7527bbb474e4e`
- `after-wasm-none-curved-whole.tsv`: `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c`
- `after-wasm-none-curved-p001-review.png`: `f738d46e3bc52cedd4af2191ce5c453c3b9d7458f503df47d114331a44369168`
- `after-wasm-none-curved-p001-overlay.png`: `98a6287e1cf3a93d1f9785f501b4715eb9e0f08fa8b33cdf6f8b1d524376b704`
- `after-wasm-visible-curved-whole.tsv`: `6320ec7d4da6e6c6930361502985a7154bf87de99f06a4697655ab5a8648f99d`
- `after-wasm-visible-curved-p001-review.png`: `2635f6f009e4633a69f987ba2ee1ac2969c92c01b4e0cceaaf1a1a70495ec54f`
- `after-wasm-visible-curved-p001-overlay.png`: `2f7092f242732e6fc9846c3b725138e4dce70714d86efc9e77da40712720b970`
