# Task #7957 11단계 — text paint style 보존과 최종 제출 검증

Runtime source는 `9c6c48c091e0780f805d91a045561b5994c7a6aa`로 고정한다.
10단계의 가림·단어 간격 보정은 유지하며, 다른 style을 한 text box로 합쳐 링크 색을
잃은 회귀를 먼저 바로잡았다. 기존 정확한 기대값과 시각 gate는 완화하지 않았다.

## 위반 규칙과 실제 소비 경로

PDF의 연속 text paint는 글꼴·크기·굵기·italic·색상·간격·scale·rise·방향이 각각 다를 수 있다.
현재 positional lowering은 span 하나를 한 text box와 한 CharShape로 내린다. 서로 다른
style을 합치면 첫 span의 속성으로 나머지를 덮어쓰므로 동일 paint group이라는 조건만으로
merge를 허용할 수 없다. 독립 Poppler SVG의 URI RGB `(5,99,193)`과 실제 이전 출력의
검정색이 이 위반의 반례다. 특정 URI나 좌표를 production 조건으로 사용하지 않는다.

`TextExtractor`의 text state → `TextSpan` style → 비텍스트 mark/Form으로 구분한 paint group과
`same_painted_span_style` → 같은 style에서만 단어 간격 복원 → 첫 원본 source order →
`src/parser/pdf/lowering.rs`의 한 CharShape/text box → RenderTree/Native SVG·Skia·WASM Canvas의
실제 text paint를 대조한다. 검색용 reading-order의 기존 glue 정책에는 이 gate를 적용하지 않는다.
같은 style의 0.25em 단어 간격은 기존 정식 검사로 계속 보호하며 가림·Form 반례도 유지한다.

## 우선 경계와 Native producer

기존 단어 간격 검사가 새 source에서 PASS한 뒤 그 nextest가 생성한 CLI를 보존했다.
관련 source bytes를 commit과 대조했고 CLI SHA-256은
`f7ab7531f2beab29f3a28f5f50631479db2a4984dfbe3c1da4b3e2774df5503f`다.
새 Native SVG에서 `Website:`는 `#000000`, URI는 `#0563c1`이며 실제 PNG 확대 판독에서도
파랑이 복원됐다. 같은 색으로 보이는 실루엣만으로 완료를 판정하지 않았다.

영향 문서·정상 경계를 먼저 산출한 뒤 Native 360쪽을 완료했다. 자원 정책 전용 1쪽을 제외한
359쪽은 전쪽 raw 90% 이상이고 공통 대조군의 0.01%p 이상 하락이 없다. 16 MiB 생성본의
Native 73.54167%는 사용자 승인에 따른 자원 전용 범위이며 Helvetica 차이는 백로그 B1이다.
동일 source의 fresh Docker WASM도 360쪽을 완료했다. 시각 범위 359쪽은 Native 최저
90.14458%, WASM 최저 90.16683%이며 쪽수·누락·공통 대조군 하락이 없다. 자원 전용
생성본의 WASM 78.19253%는 미달 그대로 보존한다.

Docker 빌드는 종료 코드 0으로 끝났으며 WASM SHA-256은
`01f4831e486ef0cb7f195d4b0fae3c17ee6e90848409e6b607d94d3df67ba962`다.
Studio 7797의 실제 HTTP 응답 bytes도 이 hash와 같고 19쪽을 표시했다. 6쪽 연결선·가림,
7쪽 글꼴·그림 경계, operators 12쪽 간격, image 1쪽 검정 배경과 검정 label/파란 URI를
독립 PDF·Native·WASM에서 같은 영역으로 확대 판독했다. 최신 source에 대한 에이전트
직접 판독이며 이전 사용자의 시각 판정을 새 source 판정으로 재인용하지 않는다.

정식 공개 WASM/Worker 검사는 95 assertion·실제 Worker 6개가 통과했다.
`formal-public-api-worker.log`, `formal-public-api-worker/browser-result.json`에 실제 실행을
보존한다. 새 Rust 2건은 IR의 유효 CharShape·SVG 색상과 HWP/HWPX 저장 후 재열기,
CLI 경로에서 같은 원본의 각 색상이 보존되는지 검사한다. 실제 이전 CLI `c9670eaf…`는
동일 CLI 검사에서 URI 색상 손실로 FAIL하고 현재 producer는 PASS한다.
`style-cli-negative.json`, `style-focused.log`에 연결하며 빌드 실패를 반례로 세지 않는다.

Native Skia lib 4,043 PASS/12 ignored, placeholder 2건·direct PDF 4건, workspace build,
fmt·Native/WASM/workspace all-target Clippy, manifest·unit-tier 검사가 통과했다.
runtime·font rule·binding을 바꾸지 않은 뒤의 test·문서·PNG commit은 이 producer를 재사용한다.
전체 Rust 회귀의 최종 결과는 아래에 기록한다.

## 증거·유보 범위

실제 이전 WASM `276acabf…`를 새 style 진단에 사용하면 URI 색상 검사에서 FAIL한다.
이는 구조·빌드 실패가 아니며 `wasm-before-style.log`의 실제 assertion으로 남긴다.
현재 source 결과와 이전 결과를 같은 실행으로 합치지 않는다. 새 rendering 회귀는 같은
원본 4쪽의 Native/fresh WASM gate와 실제 색상을 확인한 뒤 정식 suite에 포함한다.

Ignored 증거 경로는 `output/pr-review/issue7957/paint-style-20261017/`다. 원본은 기존에
보존한 `samples/issue7959/bounded-import-path.pdf`를 재사용한다. 원본·독립 PDF·font bytes를
다른 입력으로 바꾸지 않는다. `parserAllocationBoundsProven=false`, Type3 원 glyph·논리 문단의
미완료 범위와 자원 전용 B1은 유지한다. 원격 push/Open PR 승인은 이미 받았으며,
그 전 필수 검증·최종 base 확인이 끝날 때까지 등록 완료로 보고하지 않는다.


## 필수 로컬 검증 완료

`cargo nextest run --locked --cargo-profile release-test --target-dir
/home/edward/mygithub/rhwp/target/pr-review --tests --no-fail-fast`는 exit 0,
**10,620 PASS / 45 skip**으로 끝났다(실제 nextest 실행 376.349초, slow 8건).
이번 이미지 배치 3·affine clip 2·닫힌 선 3·그리기 순서 2·글자색 2건이 모두 정식
회귀에서 실행돼 PASS했다. 10단계의 중단된 실행을 합산하지 않는다.
`validation-phase2.log`, `full-regression.log`, `validation-results.json`에 명령·종료 코드·
source SHA를 보존했다. 모든 실행된 필수 단계 11개가 exit 0이다.

전 단계 font lifecycle/projection Node 49건은 동일 데이터·테스트 bytes를 확인해 재사용한다.
공개 Rust projection/trace 3건은 해당 테스트 보정 뒤의 동일 bytes 결과를 재사용한다.
Studio source·의존성 변경이 없으므로 이전 TypeScript·Studio 2,024 PASS/2 skip은 재사용하고,
바뀐 제품 WASM은 위의 공식 95 assertion/6 Worker와 실제 Studio 응답으로 새로 검증했다.

고정 `upstream/devel`은 최종 fetch에서도 `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`이며
병합 시뮬레이션은 exit 0이다. 최종 제출은 runtime을 바꾸지 않은 test·문서·증거 PNG를
같이 commit하고 승인된 upstream push/Open PR을 진행한다. PR 번호를 예측한 review·오늘할일
문서는 만들지 않는다. 정확한 code head의 원격 CI 성공 뒤 해당 기록을 시작한다.
원격 CI·self-review·병합·이슈 종료는 이 로컬 통과와 별개인 남은 절차다.
