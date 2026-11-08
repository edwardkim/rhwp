# PR #7494·#7751 15단계 Native/fresh WASM 시각 증거

셀 안 도형의 문단 기준 가로 프레임을 보정한 소스에서 전383쪽을 새로 출력했다.
compat2024·Print·96dpi이며 Native는 실제 Skia PNG, fresh WASM은 실제 Canvas PNG다.
대표 PNG도 이 새 출력으로 만들었다. SVG 브라우저 대조와 실제 Direct PDF는 별도 경로다.

- 원본: `samples/2025 행정업무운영 편람(최종).hwp`, SHA-256 `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f`.
- 독립 한컴2024+KoPub Print: `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf`, SHA-256 `3cdf329004cffad0ebacd122586e5e1702aff36b3015cbcb401d038857019593`,383쪽. 기존 정상 Print를 재사용했다.
- WASM 실제 생산 source: `542a8bcbfe00247f554706306f930a16c04a589f`. Native는 `cef058e45`+기록한 fmt patch에서 생산됐으며, 실제 생산 파일 SHA가 같은 source와 일치한다. 이후 정식 회귀·문서 변경은 제품 변경과 구분한다.
- 공용 `target/pr-review`·jobs8, fresh WASM codegen16·Docker 부재 호스트의 `--no-opt` 대체 경로다. 시스템 원본 글꼴을 경로·SHA로 공급했고 글꼴 프로그램을 PR에 복사하지 않았다.
- Native binary SHA-256: `fba2b98ebef7c82dff6f2e84873a342486a12833ebe06916f3b3eca29eef2e0a`.
- fresh WASM SHA-256: `a92b93cdfd7672d5d5d197380ee3c5e9bee505dae326fb76d7973ff8e17aee85`.

고정 devel은 `b7a5388034ee66f58a7e1d5363974e83b6374f57`, 기준 산출물의 실제 생산은
`552514f423ec91f1b7171f6e6b0882a4efe034f2`다. 두 SHA 사이 renderer/paint/model/document_core/
serializer와 HWP parser 소스가 같음을 확인했다. PDF 가져오기 차이와 실제 기준 실행 SHA는 구분한다.

## 전쪽 결과와 직접 판독

전383쪽 raw 최저는 Native93.07066%, fresh WASM92.98362%다. 두 경로 모두90% 미만·누락·
고정 devel 대비0.01%p 이상 하락은0쪽이다. 판정은 색상 경계 보정 전 `silhouette_raw_match_percent`다.

14단계에서 raw99.6%에도289쪽 그룹 화살표가26px 왼쪽인 것을 확대 판독으로 찾아 PR을 보류했다.
셀 안 문단 기준 원점에 원본 문단 여백이 빠진 원인을 보정해 두 그룹의 x 오차가0.37/0.33px로 줄었다.
같은 규칙으로135쪽 일반 강조 사각형도 독립 Print 괘선과 약0.1px 차이로 맞았다.
두 쪽의 Native/fresh WASM overlay를 직접 열어 화살표·강조 영역·기존 일반 선의 보존을 확인했다.
두 경로 각각 나머지381쪽 PNG는14단계와 byte-identical이므로 이전 확대 판독 증거도 유지한다.
새 코드의 변경 범위를 점수만으로 승인하지 않았으며 PR은 최종 필수 회귀·lint 결과와 구분한다.

전체11개 노드의 x 외에 세로 좌표·크기·문구·소유·순서 변화는 없다. Native/WASM 전383쪽의
내용·소유 감사도 내부64/32비트 usize sentinel 대응 외에는 차이가 없고11쪽 각주1개를 유지한다.

## 대표 페이지

| 쪽 | Native raw | fresh WASM raw | Native review | WASM review |
| ---: | ---: | ---: | --- | --- |
| 3 | 100.00000 | 100.00000 | [PNG](native_review_003.png) | [PNG](wasm_review_003.png) |
| 90 | 93.07066 | 92.98362 | [PNG](native_review_090.png) | [PNG](wasm_review_090.png) |
| 130 | 98.69152 | 98.45828 | [PNG](native_review_130.png) | [PNG](wasm_review_130.png) |
| 135 | 98.05498 | 98.03088 | [PNG](native_review_135.png) | [PNG](wasm_review_135.png) |
| 224 | 99.77352 | 99.75272 | [PNG](native_review_224.png) | [PNG](wasm_review_224.png) |
| 226 | 99.79541 | 99.81998 | [PNG](native_review_226.png) | [PNG](wasm_review_226.png) |
| 241 | 98.23048 | 98.27806 | [PNG](native_review_241.png) | [PNG](wasm_review_241.png) |
| 289 | 99.74756 | 99.72209 | [PNG](native_review_289.png) | [PNG](wasm_review_289.png) |
| 290 | 99.45606 | 99.44102 | [PNG](native_review_290.png) | [PNG](wasm_review_290.png) |
| 297 | 99.23751 | 99.24374 | [PNG](native_review_297.png) | [PNG](wasm_review_297.png) |
| 375 | 99.37858 | 99.42850 | [PNG](native_review_375.png) | [PNG](wasm_review_375.png) |
| 383 | 100.00000 | 100.00000 | [PNG](native_review_383.png) | [PNG](wasm_review_383.png) |

130·289·375쪽은 Native/fresh WASM standalone overlay도 보존한다. 파랑은 RHWP만,
빨강은 PDF만, 주황은 양쪽 색 차이, 회색은 일치다. 전쪽 TSV 원본은 PR 본문에 직접 수록한다.

## 회귀·독립 기대값

3·130·135·241·289·290·297·375쪽에서 양쪽 raw98% 이상과 직접 판독 후 독립 좌표±2px를
관계 검사와 함께 보호한다. 같은 동작은100%→99%대→98%대를 우선한다. 그룹은99%대289쪽,
일반 사각형의 문단Center/도형Left 경계는 고유135쪽, WMF crop은 고유130쪽에서 검사한다.

135쪽 빨간 Print S 경로의 bbox는(112.198667,358.365333,66.968,43.836)px다. 289쪽은
독립 Print B 경로의 CTM을 적용해8화살표의 시작 좌표를 각각 검사하며 일반 화살표2개의 위치도
유지한다. 소유·누락·중복·원본 그림 안 포함을 함께 확인한다. 동일 정식 파일은 수정 전4PASS/2FAIL,
수정 후6PASS였다. [15단계 기록](../../../working/task_m100_7751_stage15.md)에 원인·호출 경로·명령을 연결한다.

## 색조와 실제 브라우저·PDF

실제 한컴 저장 입력5개와 독립 Print의 전쪽 비교는 Native/fresh WASM/실제 Direct PDF 모두 raw100%다.
입력·PDF·독립 RGB는 [색조 기준 원장](../../../../pdf/issue7751/README.md)에 고정했다.
새 source에서 Canvas550·Studio DOM screenshot550 패치를 각각 독립 Print와 비교한 브라우저8건도 통과했다.
[DOM 이전·Print·보정 비교](dom_color_review.png)는13단계 실제 screenshot이며 이번8건의 관측값과 같다.
밝기20·대비0의 이전(96,120,144)을 독립 Print(131,151,171)와 같은 resolved PNG로 소비한다.

편람3쪽의 실제 Direct PDF는 raw100%다. [review](direct_pdf_review_003.png)와
[overlay](direct_pdf_overlay_003.png)를 보존한다. 전383쪽 Direct PDF 지원을 주장하지 않는다.
63쪽 Path 연결선(#7501),130쪽 SVG payload,289쪽 Line 화살표는 기존 손실 방지 guard로 명시적으로
거부됐다. unbaked/미지원 legacy 효과의 거부도 유지한다. 미세 글자 힌팅·raster 색 차이와 WMF
저색수 확대의 일반 팔레트 경계는 남으며100% 완전 호환으로 확대하지 않는다.

## 증적 보존

`stage15-*-provenance.json`, 전쪽 TSV·PNG·내용/소유 감사·before/after·브라우저 로그는 실행 중
ignored `output/pr-review/pr7494-20261016/`에 둔다. merge 후 핵심 증거를
`/home/tsjang/rhwp-review-evidence/pr7494-20261016/`에 해시 확인 후 이관하고 root output을 정리한다.

## PNG SHA-256

| 파일 | SHA-256 |
| --- | --- |
| direct_pdf_overlay_003.png | `734ee88c16a6bbb7cc48e7aeb93bfa3706c394c8bad114b28c761cdbacd1c7f0` |
| direct_pdf_review_003.png | `7fe8677c20e068c6db85ad773cc14a1731235f2574e1f5d5fb96a7e0b749b97e` |
| dom_color_review.png | `e32b21aebb8fa57f3ddaf51fcfd7b6225419d6e325fd765fc8aadc03759d1361` |
| native_overlay_130.png | `44bb001b395fc8fe4288bedf9f0033158caa4c851c033e59403de31c1ed6bf66` |
| native_overlay_289.png | `215b01bf523607ac3dbd15b83cc9a6475b552755f8a55d80d0a0c516865541c8` |
| native_overlay_375.png | `1406e515da28a0ad0348ba3be976376dac52fa7428fe6fe0cc85ca2b7136700a` |
| native_review_003.png | `d31eef9249a1b433004096394a194eda56e6b7450821955c7810ddec35158990` |
| native_review_090.png | `dac1f0354fce9a8f5883422dd5b426ed79542243a7a2f1bf1d1dbe071ffde8ec` |
| native_review_130.png | `c196082088b96e002fa00f464f5bda7f747053bc529aa7ded0c9cd400d60901f` |
| native_review_135.png | `7e583d72549be58e879e1f001034e91ec5c442aeaf5491c073c8d29368d48234` |
| native_review_224.png | `e20dbe004e8d3d75086af1df735040be77c8c35c5db0894ade7d441aeb7526b6` |
| native_review_226.png | `d8fff850e80708f6d09ff0916f74da33b33a9fe75bf88023408c7f342c410cca` |
| native_review_241.png | `4a26e9f0013d48ea24275856983fbed3a5429bf3528531a057e0aa2e33963995` |
| native_review_289.png | `8a42ae052ee833a2201b66b62cfc13db220f74e9604e4839f9d5bd5ea25086f9` |
| native_review_290.png | `e2f8c451cbd406bbaeaa2e8fa4608a5cc36c4065b9131e731cfb8eb2a3a9691c` |
| native_review_297.png | `ca49bf2fcbdf8cccd00d7afc6716ddeb5e20ac417b6a76c9e5a3ff85cdd02a43` |
| native_review_375.png | `47e4651e94ae87669251171ac171be1c7f48071f46b217b3ca22e7b415b92e01` |
| native_review_383.png | `1b548781d30ab8ab803381decb5830d1602f8b0d1d99afb1a3ce7fc6227677b8` |
| wasm_overlay_130.png | `36de77132a41b0685f207f4a4f4496b724c796ba0db62153a0001781ea97276c` |
| wasm_overlay_289.png | `791eafa5ab7d69af5c03a2839324c66337b005be18c185aba11e18463d4df488` |
| wasm_overlay_375.png | `99cd8e0db7d19b586b951b7237a457f9e38911cb8ee3a454876ba497b2b0f855` |
| wasm_review_003.png | `6009dcc964449b111cc78cfcf470cbc4814f7042093106ea21e97e13892f1e7c` |
| wasm_review_090.png | `eed38f796703de405337b16df621603e424a7c8b0e4294491cce1a6d2af63a1a` |
| wasm_review_130.png | `d69e8e046a7eeee83d7d3a64ac21d38fed866a57da113a42e7c76a72a13bb83e` |
| wasm_review_135.png | `2b7a1fa4a58ef26ff891f7df83a80aa3fee3168bca65b17f5c054b9bde9a17ec` |
| wasm_review_224.png | `6395aafe8e94f324cf52bbe08fed69b90549c11d9b41e7a92680d89d4e02dd22` |
| wasm_review_226.png | `bc6461db1b946995d69eaa4e927f29022e76b005bdcaf6a35130882bca88a7ed` |
| wasm_review_241.png | `a389343daed190d4f66d2b13f92ebe2b7a1fa162c9b324e7e186ed49058763d8` |
| wasm_review_289.png | `463c4e71fcceab6230be7eaa7837353d76fcdbe95de90249a7c0ab981daa0785` |
| wasm_review_290.png | `a50254b654484ba1362d4393f95595c3cc6133cdc03264e701de692646bab36b` |
| wasm_review_297.png | `e1bc2fdfefe2b4d2be1552aa01b6772c15e235a22d1bfa1abff51fa973a97bb8` |
| wasm_review_375.png | `c66b4e5541911850d1e2baf51f63940c5e224f6fdb4ce69772e118be5c966432` |
| wasm_review_383.png | `402b5da08027088c15bd70464ce14285d4d2d40b8d0b9f4be9805409215d6d02` |
