# #8309 메인터너 보정 검증 증적

## 대상과 원인

원 기여 `a9836a959015e9f4709d1b659994b7316f812289`의 저장 문제를 최신 devel
`c88657d9c30b57d8481de40f74fd76fd3e2a0af7`에서 재현했다. 호환 merge
`dda384eadc4148ff4c939efdf1a184aacd0e59fa`는 #8174/#8360의 Storage writer를 보존하며,
제품 보정 `01e9f4ff012acceaa477c7f4f0a085b86747ef42`는 그 writer에서 정렬 비트를 생성한다.
원 기여 commit은 재작성하지 않았다.

생성·속성 편집의 `TextBox.vertical_align` → HWP `write_text_box_header`의 raw 비트 →
재열기 파서의 `vertical_align` → `shape_layout`의 자식 텍스트 정렬이 실제 소비 경로다.
구현은 세로 정렬 bit 5–6만 갱신한다. 다른 raw 비트, 추가 13바이트, payload 측정·쓰기의
같은 writer, Storage 한도 및 I/O 오류 전달은 유지한다. 셀/캡션의 offset 2 기준 bit 21–22와
글상자의 현재 내부 offset 4 기준 bit 5–6을 혼동하지 않는다. 새 사양 정오표를 주장하지 않는다.
측정·paint·페이지 분할·이어받기 알고리즘은 변경하지 않았다.

`base-probe.json`은 실제 기존 WASM에서 기본 Top·편집 Bottom·재편집 Top이 HWP 저장 후
Center로 바뀌고 HWPX는 보존됨을 기록한다. `native-before-result.json`은 runtime source를
동일한 devel 바이트로 만든 Native에서 원 회귀 3건이 모두 FAIL한 실행의 provenance다.
원 테스트 assertion은 그대로 두고 후보에서 3/3 PASS했다. 첫머리 주석만 추가했다.
빈 글상자의 API/저장 계약 검사를 아래 글자 있는 문서의 시각 측정과 구분한다.

## 입력과 독립 기준

공개 합성 입력은 RHWP의 실제 API로 생성했다. 18000×12000 HWPUNIT 글상자에
`Text box alignment ABC 123`을 넣고 기본 Top → Center → Bottom → Top 순서로 편집했다.
한컴이 생성한 원본이라고 표시하지 않는다. 입력 생성은 `reproduce.mjs`로 재현할 수 있다.

- `tests/fixtures/pr8309/textbox-reference-{top,center,bottom}.hwpx`: 수정 전 WASM의 올바른
  HWPX 저장 경로에서 확보한 기준 입력.
- `tests/fixtures/pr8309/textbox-saved-{default,center,bottom,top}.{hwp,hwpx}`: fresh 후보 WASM의
  실제 저장 바이트. HWPX는 각각 기준 입력과 바이트가 같고, 마지막 Top은 default와 두 형식
  모두 바이트가 같다. `saved-roundtrip.json`에 각 action·정렬·쪽수·SHA-256을 기록했다.
- `pdf/pr8309/textbox-reference-*-2024.pdf` 및 `textbox-saved-*-2024.pdf`: 위 기준 HWPX 3개와
  후보 HWP 3개 **각각**의 독립 한컴 2024 Print. HWP의 기준을 HWPX PDF로 대신하지 않았다.
  전부 실제 한컴 13.0.0.3901, PrintMethod 0, `frame_print_to_pdf_ex_one_up`, 1쪽이다.
  `print-execution.json`과 `saved-print-execution.json`에 실제 실행과 입력·PDF 해시를 보존했다.
  6건의 `font_scope=verified` 확인은 `print-font-scope.json`에 있다.
- 기존 #8134/#8393 대조군은 `samples/issue8122/textbox_valign_no_lineseg.hwp`와
  `pdf/issue8122/textbox_valign_no_lineseg-webhwp-print.pdf`다. 기존 Print를 재사용한 사실을
  새 Print 6건과 구분한다. 해당 문서의 관계형 회귀는 변경하지 않았다.

`inputs.json`은 새 입력·기준 PDF와 기존 대조군의 바이트 수·해시다. 폰트 프로그램·WASM·로그·서버 주소·인증
설정·변환 capability는 커밋하지 않는다. 생성 문서만 공개하며 개인 corpus를 사용하지 않았다.

## 전쪽 시각 결과

측정일은 2026-11-02다. Native Skia와 최적화 fresh WASM Canvas2D에서 직접 PNG를 출력했다.
96dpi, print profile, raw 2px 실루엣, `threshold_boundary_color_support_v1`을 사용했다.
새 문서는 `compat=2024`, 기존 #8122는 `compat=2022`다. 색 경계 보정 전 수치를 판정에 썼다.

| 입력 (각 1/1쪽) | base Native | 후보 Native | base WASM | 후보 WASM |
| --- | ---: | ---: | ---: | ---: |
| 기준 HWPX Top | 100.00% | 100.00% | 100.00% | 100.00% |
| 기준 HWPX Center | 100.00% | 100.00% | 100.00% | 100.00% |
| 기준 HWPX Bottom | 100.00% | 100.00% | 100.00% | 100.00% |
| 후보 저장 HWP default/Top | 100.00% | 100.00% | 100.00% | 100.00% |
| 후보 저장 HWP Center | 100.00% | 100.00% | 100.00% | 100.00% |
| 후보 저장 HWP Bottom | 100.00% | 100.00% | 100.00% | 100.00% |
| 기존 #8122 | 99.55% | 99.55% | 99.42% | 99.42% |

7개 고유 입력, 4개 source/backend 조합의 전쪽 TSV 28개를 `tsv/`에 원문으로 보존했다.
`manifests/`는 해당 실행의 raw 측정 provenance이며 `--png-pair`는 이미 출력한 같은 PNG를
재측정한 경로다. 전체 쪽수는 원문 직접 출력과 PDF raster 각 1쪽임을 별도로 확인했다.
각 문서·경로의 metrics·PNG·TSV 연결은 `visual-results.json`에 있다. 미달·누락·제외 영역은 없다.
최저는 Native 99.55%, fresh WASM 99.42%다. 글꼴 예외를 사용하지 않았다.

동일 입력의 base/후보 PNG는 Native 7쌍과 WASM 7쌍 모두 바이트가 같다
(`same-input-preservation.json`). 이는 renderer가 보존된 증거이며, 잘못 저장한 이전 HWP가
개선된 증거와 혼동하지 않는다. 저장 개선은 위 수정 전 FAIL/수정 후 PASS, 실제 action 왕복,
후보 저장 HWP 자체의 Print·시각 대조로 확인했다.

raw 100%를 픽셀 완전 일치로 표시하지 않는다. 새 문서의 pixel match는 Native
99.88517–99.90223%, WASM 99.91458%다. 일반 ink 기반 proxy는 Native
59.35638–64.42114%, WASM 68.37074%다. `metrics/`에 일반 비교값을 따로 보존했다.

## 직접 판독과 글꼴 환경

`png/`의 원본 전쪽 PNG와 `saved-roundtrip-zoom.png`를 직접 비교했다. 같은 96dpi 영역을
두 배로 확대했고 원래 화소 비율을 유지했다. Top의 위 여백, Center의 중앙 정렬, Bottom의 아래
여백, 테두리·문자 간격·기준선과 `ABC 123`의 보존을 확인했다. 잘림·누락·중복이 없다.
합성 입력에는 수식·첨자가 없으므로 없는 수식의 통과를 주장하지 않는다.

기존 문서는 `legacy-control-detail.png` 및 전쪽 PNG에서 제목·본문 줄·도형·후속 설명의
소속과 간격을 대조했다. Print와 Native/WASM의 획 굵기·래스터 표현 차이는 남아 있으나,
이번 수정 전후 출력은 위 14쌍 전부 동일하며 정렬 기능의 후퇴는 없다.

시각 비교에는 새 문서의 함초롬바탕 1.980 Build 20140819, 기존 문서의 HBATANG·굴림 파일을
명시했다. 실제 파일 해시와 출처는 `producer.json`에 있다. 이 fidelity 환경과 Rust 회귀 환경은
다르다. Rust 검사는 bubblewrap에서 시스템·개인·fidelity 글꼴 경로를 숨기고 저장소의 공개
fallback 5개만 노출했다. 이번 실행에서 실제 파일과 해시를 다시 확인한 결과는
`font-isolation.json`이다.

## 실제 Studio와 소스 경계

별도 7706 Studio에서 초기 로딩이 소비한 WASM 응답 바이트를 CDP로 관측해 후보 파일의
SHA-256과 일치함을 확인했다. 새 요청으로 다시 받은 바이트로 대신하지 않았다.
실제 WasmBridge 속성 편집·HWP/HWPX export와 실제 파일 입력 재열기를 8회 확인했다.
글꼴 선택 창에 응답하고 로딩·busy 종료까지 기다렸다. 정렬·글자·1쪽 상태·Canvas를 확인했으며
page error는 없다. `studio-roundtrip.json`, `studio-roundtrip.png`는 이 UI 검증의 증거다.
Studio 스크린샷의 대체 글꼴 표시를 위 Print 시각 측정으로 대신 사용하지 않았다.

제품 source는 `01e9f4ff012acceaa477c7f4f0a085b86747ef42`에서 고정했다. 이후 변경은 테스트
첫머리 주석과 재현·증적 문서/데이터이며 제품 소스와 바인딩 표면을 바꾸지 않는다.
WASM 빌드·Native 실행 파일과 환경은 `producer.json`에 기록했다. 전체 Rust·Native 및
Studio gate 결과는 `rust-gates.json`, `studio-gates.json`에 연결한다.
