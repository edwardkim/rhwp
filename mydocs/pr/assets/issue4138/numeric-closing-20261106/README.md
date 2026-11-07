# #4138 숫자·섭씨 금칙의 줄 소속과 위치

2026-11-06 코드 `fa0436f4951db3bcafa2403c05e202cb425355fe` / base `45684640be5eddcf5972b063260250345b34d928`.
원본은 `mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp`,
기준은 `pdf/issue4138-ascii-split-20261016-2024.pdf`다. 입력을 재생성하거나 기준 PDF를 교체하지 않았다.
전체 SHA·변환기·실행 파일·페이지별 결과는 validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue4138/numeric-closing-20261106/validation.json`)에 기록했다.
현행 Visual Sweep·PDFium·192DPI·2024 compat·색상 경계 보정 전2px raw다.

| 경로 | 수정 전90쪽 | 후보90쪽 | 후보 전197쪽 최저 | 98% 미달 |
| --- | --- | --- | --- | --- |
| Native | 89.85554% | 99.99501% | 91.17674% | 7쪽 |
| fresh WASM | 90.20277% | 100.00000% | 91.29951% | 7쪽 |

위 수정 전90쪽은 직전 구현의 전쪽 결과이며, 새 base4568의 수치는 별도 base TSV에서 확인한다.
21·30·44·120·129·130·189쪽은 여전히98% 미만이다. 문서 전쪽 완료로 판정하지 않는다.
[Native review](candidate-native-p090-review.png)·[overlay](candidate-native-p090-overlay.png),
[WASM review](candidate-wasm-p090-review.png)·[overlay](candidate-wasm-p090-overlay.png)를 직접 열어
`14 | 0℃` 줄 소속, 앞뒤 본문과180℃, 행 시작·종료·간격·이어받기를 확인했다. 섭씨 기호의 미세 차이는 남는다.

`issue_4138_numeric_closing_units`는 줄 소속·내용 순서·누락·중복 방지의 관계형이다.
`issue_4138_print_temperature_position`는 독립 PDF x=111.1797409pt를 실제192DPI ±2px로 유지하는 고정형이다.
API의96DPI 논리 좌표에 출력 배율2를 적용한다. 조판 자체의DPI를 바꾸지 않는다.
새 고정형은 base FAIL → 후보 PASS를 확인했다. 원본115쪽·저장197쪽 등 기존 검사도 유지한다.
Mac fresh WASM은 locked wrapper의 web/pkg/--no-opt 로컬 대체 빌드이며 Docker 최적화 통과가 아니다.
