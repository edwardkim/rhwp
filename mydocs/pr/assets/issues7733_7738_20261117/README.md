# #7733·#7735~#7738 원본 조판 보정 증적

현재 검증 소스는 `8771cdcbc48cc324bddc790b32bf8516b52b6379`, 고정 비교 base는
`08b38c12a75fa5cabfe5fbbfde9388aceb3fcca3`다. 최신 Native·WASM 빌드의 제품 소스는
`0fee3421a4ca0fd6e9484b67e4da0b2f4c5e44b0`이다.
[제품 소스 동일성 대조](hy-hanyang-font-identity/test-only-render-reuse.json)와
[이전 리베이스의 렌더링 재사용 감사](rebase-08b38-rendering-reuse.json)에 각 산출물의 생산 소스를 기록했다.
[다섯 원본의 전쪽 TSV·대표 판독·보호 등록](final-efe954/README.md)을 추가했다.
최종 로컬 보호 검증은 46개 데이터셋 4,046쪽에서 하락·누락 0쪽이다. 편람 Native 필수
게이트의 383쪽도 모두 통과했다. [보호 검증 결과](hy-hanyang-font-identity/protected-validation.json) ·
[편람 필수 게이트](hy-hanyang-font-identity/required-handbook/validation.json)

## 원본과 변경 범위

| 이슈 | 원본 문서 | 쪽수 |
| --- | --- | ---: |
| #7733 | 화학제품 표시 기준 `samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp` | 103 |
| #7735 | `samples/한글문서파일형식_5.0_revision1.3.hwp` | 71 |
| #7736 | `samples/pr4093/outline_navigation_panel_demo.hwpx` | 3 |
| #7737 | `samples/issue6776/78494-virtual-convergence-industry-decree.hwpx` | 74 |
| #7738 | `samples/pr4093/outline_navigation_table_cell_number.hwpx` | 1 |

저장된 글줄·셀의 첫 프레임과 이어받기·종료 간격, 원본 글꼴 프로그램의 실제 윤곽과 전진,
글머리표·첨자·작은 기호·수식 공급자 및 원점을 공통 측정·배치·표시 경로에서 보정한다.
[원 해결 이력·원인별 분석](analysis.md)에서 각 단계의 독립 Print 대조와 정상 대조군을 연결한다.

원본 및 독립 기준 PDF를 변경하지 않는다. 공식 사양 PDF
`pdf/한글문서파일형식_5.0_revision1.3-hancom-official.pdf`는 재산출하지 않고 그대로 사용한다.
다른 기준은 데스크톱 한컴의 Hancom PDF Print이며 웹한글기안기·Microsoft Print to PDF와 구분한다.
[현재 커밋과 실제 검증 입력 52개 동일성](committed-input-audit-current.json)을 확인했다.
기존 base의 대용량 PDF는 동일 Git blob을 재사용하며 이번 PR에 새로 추가하거나 수정하지 않는다.

## 현재 완료한 추가 경계

- [sample4 26쪽 회색 배경 하단](sample4-gray-tail/cause.md): 본문에 수용한 마지막 줄의 후행 채움만
  보존한다. 페이지 배분과 외곽 테두리 clip은 유지한다. 새 반례는 보정 전 1 FAIL, 모듈은 보정 후
  6 PASS다. Native/fresh WASM 25쪽은 100.00%를 유지하고 26쪽은 각각 99.37→100.00%다.
  [실제 결과](sample4-gray-tail/validation.json),
  [WASM review](sample4-gray-tail/wasm/review-026.png)·
  [overlay](sample4-gray-tail/wasm/overlay-026.png)를 함께 보존한다.
- [원본 글꼴 우선 보호 조건](priority-original-font-qualification/README.md): nested 8쪽, 네 글꼴
  스타일 대조군 1쪽, sample16 HWP2022 38·43쪽, 영월 서식 1쪽을 Native/fresh WASM과 base/후보에서
  측정한 20쪽 모두 100.00%다. 후보의 같은 페이지 review·overlay 10쌍을 직접 판독했다.
  이는 18조건 전쪽 검증의 완료를 대신하지 않는다.
- Studio 분할 프로그램의 준비와 실제 CanvasKit 소비 경로는
  [수집 원인](split-host-font-preparation/cause.md)과
  [소비 경로](split-host-font-consumer/actual-controls/validation.json)에 구분한다.
  해당 실제 소비 증거의 원 생산 커밋을 최신 paint 코드로 재명명하지 않는다.

이 수치의 100%는 기존 **2px 허용 raw 실루엣** 판정이며 픽셀 동일이 아니다. sample4 하단 약 1px,
보호 대조군의 얇은 선·잉크·회색 밴드의 미세 차이를 직접 판독 기록과 분리하지 않는다.
Visual Sweep은 원본 글꼴을 공급하는 정상 환경에서 실행하고, Rust/Studio 회귀는 별도로 검증한
macOS 최소 글꼴 sandbox에서 실행한다. 비공개 원본 글꼴 바이너리는 이 증적에 포함하지 않는다.

## 진단 소유권 보정

[후행 배경과 실제 넘침을 구분하는 공통 계약](overflow-fill-ownership/cause.md)을 보정했다.
새 반례는 보정 전 6 PASS·1 FAIL, 보정 후 7 PASS다. 기존 본문 넘침 16분할, CLI·off-canvas·빈 글줄·
글리프 경계까지 [집중 66 PASS·0 FAIL](overflow-fill-ownership/validation.json)을 확인했다.
원장 상한·허용 오차·원본 입력은 유지한다. 최신 코드의 전쪽 결과가 끝나기 전까지 이전 그림의
source SHA를 최신 것으로 바꾸거나 전체 검증 완료로 쓰지 않는다.

## 최신 전체 회귀

`8771cdcbc48cc324bddc790b32bf8516b52b6379`의 최소 글꼴 전체 nextest는
**13,781건 통과·실패 0건·제외 45건**, 테스트 873.621초다. #6171의 기존 네 검사도
통과했다. Native·WASM은 제품 소스가 같은 `0fee3421a4`의 빌드를 재사용했다.
[전체 회귀 영수증](hy-hanyang-font-identity/test-expectation-validation.json) ·
[제품 소스와 빌드 해시 대조](hy-hanyang-font-identity/test-only-render-reuse.json)

포맷·Clippy·workspace build·수집 manifest·Native Skia를 포함한 Rust 검증 21단계가 통과했다.
[단계별 실행 결과](hy-hanyang-font-identity/rust-validation.json)

### 전쪽 비교 자료

다섯 원본의 최신 Native와 fresh WASM을 각각 전 252쪽 산출했다. 대표 판독 쪽의 현재 출력과
기준 PDF 픽셀이 이전 직접 판독 자료와 일치했다. [Native 27쪽](hy-hanyang-font-identity/native-direct-reading.json)과
[fresh WASM 26쪽](hy-hanyang-font-identity/wasm-direct-reading.json)의 대조에 이전 review·overlay를 연결했다.
원본별 전체 TSV와 페이지별 고정 base 대비 차이는 [전쪽 비교 기록](hy-hanyang-font-identity/scope-whole-validation.json)에
있다. 고정 base 비교에 사용한 Native·WASM의 생산 SHA와 재사용 범위는
[base 산출물 신원 대조](hy-hanyang-font-identity/fixed-base-render-reuse.json)에 기록했다.

| 원본 | 쪽수 | 최신 Native 최저 | 최신 fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| 화학물질 표시 기준 | 103 | 99.75% | 99.71% |
| 한글 문서 파일 형식 5.0 개정 1.3 | 71 | 99.69% | 99.67% |
| 개요 탐색 패널 | 3 | 100.00% | 100.00% |
| 가상융합산업 시행령 | 74 | 99.74% | 99.73% |
| 표 셀 개요 번호 | 1 | 100.00% | 100.00% |

별도 대조군인 원본 #4138 분할 저장본은 두 경로에서 197쪽을 유지했다. 최저값은 Native
99.28%, fresh WASM 99.36%다. sample4는 두 경로에서 전 36쪽을 비교했고, 사용자가 지적한
회색 배경 경계의 25·26쪽은 양쪽 모두 100.00%다. 두 쪽의 출력 PNG와 기준 PDF PNG는 각각
이전 판독에 사용한 파일과 같은 해시다. [sample4 직접 판독 대조](hy-hanyang-font-identity/gray-tail-direct-reading.json)

### 최신 Studio 실제 출력

최신 WASM으로 15쪽의 실제 출력을 45건 캡처했다. 직접 요청한 Canvas2D 15건, CanvasKit 준비 실패로
Canvas2D로 전환된 26건, 실제 CanvasKit 4건을 확인했다. raw 실루엣 최저는 98.63244%다.
45건 모두 이전과 디코딩한 RGB 픽셀·기준 PDF PNG·실제 백엔드가 일치하여 직접 판독한 17쌍을
재사용했다. 15개 캡처 묶음의 실제 초기화 JS·WASM 응답과 브라우저 E2E 3건을 함께 기록했다.
[최신 출력과 직접 판독 대조](hy-hanyang-font-identity/actual-browser-validation.json) ·
[Studio 배포 빌드](hy-hanyang-font-identity/studio-production-build.json)

최신 Native `export-svg`의 브라우저 래스터 대표 7쪽은 이전 직접 판독본의 출력·기준 PDF 픽셀과
일치했다. [SVG 캡처와 판독 대조](hy-hanyang-font-identity/svg-direct-reading.json)

최신 Native와 WASM에서 [공통 공개 4문서·8경로](hy-hanyang-font-identity/fresh-public-validation.json)가
모두 통과했다. 기준 PDF와 출력 PNG의 해시가 일치하여 같은 쪽의 기존 직접 판독을 연결했다.

### sample16 보호 비교의 글꼴 공급

고정 WASM 글꼴 목록에 Mac에 설치된 굴림체 TTC가 빠져 있었다. 같은 파일과 컬렉션 face를
공급한 집중 비교에서 HWPX 18쪽은 77.59066%에서 99.99067%, 35쪽은 82.46843%에서
100.00%로 복원됐다. 실제 WASM 공급 기록에서 SHA-256과 face 1을 확인했다. 요청한 151개
문자의 윤곽·가로 메트릭은 독립 Windows 설치본과 일치했다.
[공급·판독 기록](hy-hanyang-font-identity/sample16-gulim/validation.json)

sample10의 원래 WASM 보호 검증에는 수동 글꼴 환경이 없었다. 같은 입력·PDF PNG·WASM·글꼴
목록으로 대조한 결과, 수동 글꼴 환경이 있을 때 9쪽은 99.96%였고 자동 선택에서는 GPU와
소프트웨어 렌더링 모두 100.00%였다. [자동 선택과 수동 매핑 대조](hy-hanyang-font-identity/sample10-automatic-font-validation.json)

### 글꼴 신원 보정 전 회귀 이력

`efe9540bd4f1cbaddd89bf018771ef1de20389c8`의 최소 글꼴 전체 nextest는
**13,780 PASS·0 FAIL·45 skip**, 테스트 823.941초다.
[실제 결과](final-efe954/full-nextest-result.json)와
[명시적 요약](final-efe954/rust-gates/full-nextest-summary.txt)을 보존한다.

새 Native/fresh WASM의 [기여자 공통 8경로](final-efe954/fresh-public-validation.json)는
모두 통과했고 각 review·overlay를 직접 판독했다. 같은 fresh WASM 컴파일 산출물에서
만든 Node 바인딩의 Studio 단위 검사 **2,814 PASS·0 FAIL**과 TypeScript·production build·
E2E manifest·문서 링크·Chrome 옵션 검사는
[frontend 검증](final-efe954/fresh-frontend-validation.json)에 기록했다.
남은 Rust 게이트·보호문서 전쪽 비교의 완료를 뜻하지 않는다.

### 리베이스 전 회귀 이력

리베이스 전 소스 `e63992fb2f4fbedb9c58569232b176ae91b98331`의 최소 글꼴 전체 nextest는 **13,774 PASS·0 FAIL·45 skip**, 실행 672.944초다.
이전 실패 세 건을 포함한 전체 검사 결과이며, 아래 전쪽·보호·실제 소비 경로 수용과 수집 복원 뒤의
재검증을 대신하지 않는다.

## 제출 전 남은 검증

다섯 문서 전 252쪽과 원본 #4138 197쪽의 Native/fresh WASM 전쪽 산출을 완료했다.
다섯 문서 504행을 정본에 추가하여 총 23문서·4,046행으로 보호한다. 기존 보호 18조건
1,771쪽씩의 하락 금지 검증과 sample4 전 36쪽 기능 보존, 필수 편람 HWPX Native
383쪽 검사 등 최종 게이트는 각 실행 상태와 완료 결과로 별도 확정한다.
편람 전용 `--handbook-gate`는 Native 383쪽 전부 통과했다. 최저 raw는 99.96503%이고
보호 기준보다 하락한 쪽은 0쪽이다. [별도 편람 검사 결과](hy-hanyang-font-identity/required-handbook/validation.json)

`jongbu_law.hwpx`의 WASM 1쪽은 현재 Mac의 GPU 경로에서 base와 후보 모두 99.96967%였다.
기존 수용 기록의 소프트웨어 Chrome 조건으로 같은 두 빌드의 전 6쪽을 다시 비교한 결과 1쪽은
둘 다 99.98059%로 정본 표시값 99.98%를 유지했다. 입력·기준 PDF·공급 글꼴·render tree가
같음을 확인했다. 기존 보호 18문서의 WASM 전쪽은 이 소프트웨어 조건으로 비교한다.
[GPU와 소프트웨어 실행 대조](hy-hanyang-font-identity/issue7905-raster-condition.json)

| WASM 1쪽 | review | overlay |
| --- | --- | --- |
| GPU 99.97% | [review](hy-hanyang-font-identity/issue7905-raster/gpu/review-001.png) | [overlay](hy-hanyang-font-identity/issue7905-raster/gpu/overlay-001.png) |
| 소프트웨어 99.98% | [review](hy-hanyang-font-identity/issue7905-raster/software/review-001.png) | [overlay](hy-hanyang-font-identity/issue7905-raster/software/overlay-001.png) |

최신 코드의 전체 nextest·lint·build·Native Skia와 실제 Studio Canvas2D·지원 CanvasKit·SVG,
초기화 JS/WASM 응답 신원과 직접 판독도 확인한다. fallback을 요청한 backend의 성공으로 합산하지 않는다.
완료 뒤에만 최종 TSV·생산 영수증·대표 raw/PDF/review/overlay 및 PR 본문의 결과를 확정한다.
