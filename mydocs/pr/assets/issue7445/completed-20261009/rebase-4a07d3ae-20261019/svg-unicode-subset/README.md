# SVG 부분 글꼴의 Unicode·웹 소비 보존

2026-10-20 KST, C435. Parent `f8af48bdecf9b77166757b9bd47e0abe762ccadf` + runtime patch
SHA `c803ddeb62446e228110bd3ea511ce39c6aade947a2ace7f326a9cddb542ba08`.
Native96 CLI SHA `d7fb0a267e903923372ee85082c4588c3ada5a2fb922750276381fbf0586a87a`,
lib SHA `be4a5331b5b23ea1194a7fcbbbb05e167da0a342d79e526ce37962231241b14a`.
runtime2199파일과 정식 검사 source hash를 대조했다. 글꼴 바이너리는 포함하지 않는다.

## 입력과 독립 Print

- 정식 `issue_6520_svg_layer_contract::subset_font_bytes_are_deterministic_for_multiple_codepoints`와
  같은 public blank → insert `가나다ABC123` → font 등록/서식 적용 → HWP 저장/재열기 입력이다.
  기존 para-001의 문단만 바꿔 다른 저장 줄 폭을 남기던 합성 방법을 사용하지 않는다.
- 입력은 로컬 ignored `repair89-svg-webfont-public-edit-control/svg-webfont-public-edit.hwp`,
  SHA `9cb5b230e785c9f101dc9d55357ccfb089c1809f11e09788b7368279dbc4e36a`다.
  `rhwp info`는 hancom-office-2020/11.0.0.2129, 1쪽이다.
- 독립 Print는 `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/temporary-artifacts-20261020/pdf/issue7445-svg-webfont-public-edit-20261020-2020.pdf`,
  SHA `91601e7fc918209c9bf7cffd58875d43df925210c99cb6d34c38989d6ea6889a`다.
  MCP job `b20d7b2a-9861-487b-9330-1a92ba6acf84`, engine2020/11.0.0.9136,
  method0/hancom2020_pdf_driver_one_up, 전1쪽이며 실제 PDF face는 Haansoft Dotum이다.

## 현재 코드의 전쪽 결과와 직접 판독

Print/compat2022/192DPI, raw2px 실루엣, Native/fresh WASM 각각 전1쪽이다.
canonical PDFium과 별도 pdftoppm audit를 혼합하지 않았다.

| 실제 경로·공급 | Native canonical | fresh WASM canonical | 별도 pdftoppm audit 양쪽 |
| --- | ---: | ---: | ---: |
| 직접 PNG, 사용자 글꼴+public 경로 | 100.00000% | 100.00000% | 100.00000% |
| SVG webfont, 정식 검사와 같은 public-only 경로 | 97.85523% | 97.85523% | 98.62920% |

모든 쪽수는1/1, 미달·누락·중복0이다. 직접 PNG100%를 public SVG의98% 수용으로
인용하지 않는다. public SVG에는 자형·미세 래스터 차이가 남으며 절대 px 핀을 추가하지 않았다.
현재 재출력의 review/overlay8장은 직접 판독한 이전90/92 패널과 byte동일함도 검증했다.
TSV는 최종96/97 producer에서 다시 산출했다.8개 완료 manifest와 PNG8쌍 hash를 감사했다.

- 직접 PNG: [Native review](direct-native-review.png), [overlay](direct-native-overlay.png),
  [WASM review](direct-wasm-review.png), [overlay](direct-wasm-overlay.png).
- public SVG: [Native review](svg-public-native-review.png), [overlay](svg-public-native-overlay.png),
  [WASM review](svg-public-wasm-review.png), [overlay](svg-public-wasm-overlay.png).
- `*-canonical.tsv`4개가 판정 원본이고 `*-pdftoppm-audit.tsv`4개는 별도 대조다.

## 실제 부분 글꼴과 회귀 검사

PDF용 subsetter는 cmap/OS2를 제거해 브라우저가 거부했다. Native SVG의 부분집합을
Unicode cmap·GSUB closure·GPOS·이름·힌팅을 보존하는 skera 경로로 바꿨다.
분할 금지 표시, 지원하지 않는 CFF 단일 face/legacy kern 등의 경우 선택한 전체 face를
보존한다. 이 세 대조군은 원본/내장 byte동일과 Chrome 로드를 최종96에서 확인했다.
일반·실제 Bold·문서 내장 글꼴의 기존 선택 경로는 유지했다.

정식 public-only 출력의 실제 공급은 NotoSansKR-ExtraLight.ttf,
SHA `67b4003e2be99ea44a7c957e4b35acde1b5e6e82a54fe195c5598c3c617bc2e3`다.
전체2,817,876bytes→부분4,128bytes이며 실제 내장 face를 쓰는 Chrome146에서 Full/Subset
SVG PNG byte동일을 확인했다. 정식 검사는 기존8회 결정성뿐 아니라 Unicode cmap/OS2,
em/수직 메트릭,9자 glyph 존재·전진폭·외곽 상자, 저장 텍스트의 순서·누락·중복을 검사한다.
보강된 검사는 이전68 library에서 **cmap 누락1FAIL→최종96에서1PASS**다.
prepare 후 재산출한 suite015의25건 모두 PASS,243건 선택 제외,
test1.891초/전체135.495초, exit0. NativeSkia lib Clippy도30.787초 exit0이다.

독립 cmap glyph가 없는 VS를 입력에서 제거하지 않도록 보정했다. 로컬 합성 cmap14의
A+VS→B를 원 프로그램과 같은 윤곽·hmtx로 유지하며 실제 FontFace의 폭/PNG도 원본과
동일해졌다. 보정 전에는 A로 돌아갔다. glyph ID 재배정은 윤곽 hash와 메트릭으로 정규화했다.
이는 자원 형식·소비 증거이며 독립 Print 문서의 VS 조판 수용으로 인용하지 않는다.

## 빌드·로그와 남은 범위

원 증거 prefix는 main checkout의 ignored
`output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/`다.
최종 producer96/97, 정식98, 직접99/public SVG100, 초기 응답101,
실제 Subset102, fallback104, 최종 감사105를 연결한다. `.log`·JSON·중간 SVG는 커밋하지 않는다.

Mac locked web/pkg/no-opt wrapper97을 실행했다. VS의 추가 변경은 Native 전용 helper
한 파일뿐이므로 WASM target은 재컴파일하지 않고 이전82의 byte동일 package를 재사용했다.
JS SHA `23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591`,
WASM SHA `f2c5aee17ff0d52dd698b4a936742f6231839ae9348f640ad6af21fd4c728480`.
root pkg/public과 Studio7736 실제 초기화 HTTP 응답을 대조했다. Docker 최적화 통과가 아니다.

첫 수동 모델 입력은 Print face가 바탕으로 달랐고70.99664/63.74932%였다. 그 자료를 새 입력의
개선율로 인용하지 않는다. raw 스트림 잔재만으로 구체적인 저장 차이 원인을 확정하지도 않는다.
잘못된 compat·PDFium 없는 기본 Python·standalone extern 누락·기본 Vite7700 접속 실패는
각 로그를 보존하고 올바른 환경/의존성/7736으로 재실행했다. 최종 증거에 섞지 않았다.
VS의 원 glyph 이름과 재배정 이름을 같다고 요구한 최초 진단도 윤곽·메트릭 대조로 정정했다.

전체 nextest·남은 표/수식 회귀·전체 문서/actual36는 미완료다. 이 집중 결과로 PR 준비 완료를
주장하지 않는다. PR은 최종 준비 결과를 보고한 뒤 사용자 승인 후 진행한다.
