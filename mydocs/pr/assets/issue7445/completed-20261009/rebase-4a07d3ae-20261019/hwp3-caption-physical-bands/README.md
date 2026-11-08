# #6078 HWP3 빈 줄·캡션의 물리 점유 복원

Issue: #6078. C448 후보의 기준 parent는 `6f7980becc2ce64e8c9cd462f251d00ca802e595`다.
전체 PR 검증 완료 자료가 아니며, 이 문서의 실제 Native/fresh WASM 전1쪽 비교 자료다.

- [원본 HWP3](../../../../../../../samples/hwp3-table-caption.hwp), SHA-256 `e8ac29656b1ca41f96b1dd9f6ef88be294c61165e2daf40f75873814291795e1`
- [독립 한컴2020 Print PDF](../../../../pr_6191/hancom/issue_6078_hancom_2020.pdf), SHA-256 `48940a3579d0fb3a9545c68e0b3d00da1094c9eea67773a1d781b958c7319305`
- PDFium `smooth_rgb_v1`,192DPI, compat2022, 색상 경계 보정 전2px 실루엣,90% 문턱 그대로다.
- 폰트 공급과 원본·PDF는 전후 동일하다. 미등록 PUA·폰트·비교기 정책을 바꾸지 않는다.

| 출력 | 수정 전 | 수정 후 | 실제/기준 쪽수 |
| --- | ---: | ---: | --- |
| Native | 44.45712% | 97.76020% | 1/1 |
| fresh WASM | 43.02545% | 97.82998% | 1/1 |

수정 전은 frozen291/293, 후는310/312다. Native CLI SHA-256은
`7d71ddcc2f44b9c17eb630226269a46100056809d2a690bb3b0266b895bba2ae`,
fresh WASM은 `dfb91d538f8f67cd24076f8b924da6f588f8dcfc0e31a65c9e03ce19c8f8b191`다.
Mac locked `web/pkg/no-opt` 실제 재컴파일이며 Docker 최적화 통과가 아니다.
root pkg·Studio public·실제7736 초기화 응답의 JS/WASM hash를315에서 대조했다.
이는 실제 문서 Canvas/CanvasKit 또는 actual36 수용을 대신하지 않는다.

## 원인과 기존 검사 재검토

원 해결 #6078의 저장 글줄 조회와291eabcf5b의 캡션 점유 보정을 먼저 추적했다.
38804e80b4가 추가한 첫 줄간격0 기대는 원본과 다르다. HWP3 첫 줄 높이325,
다음 줄 pgy423(각1/1800inch)로 자기 간격은 `(423−325)×4=392HU`다.
공백 줄은 보이지 않아도 이 상자와 간격을 소유한다. 표 정보의 캡션 전체 높이는
`463×4=1852HU`, 캡션 글줄은1000HU이므로 남는 띠는852HU다.
기존 계산은 마지막 글줄간격300HU를 다시 더하여1152HU로 만들었다.

표 소유 줄만 호스트 글꼴 종료 간격을 소비하고, 다른 저장 줄은 유효한 같은 프레임의
pgy 차이 또는 자기 줄간격을 보존한다. reset·고정 간격·암호 인쇄 계약을 구분한다.
캡션의 전체 물리 높이에는 마지막 줄간격을 중복 합산하지 않는다. 공통 IR의 결과를
측정과 실제 배치가 같이 소비한다. 문서 ID·페이지 번호·픽셀 보정은 추가하지 않았다.

0 기대에 맞추기만 한 중간304의 기존1PASS는 최종 수용 근거가 아니다. 그 출력307은
89.58832%로 계속 미달이었다. 원본 gap만/캡션만 고친 frozen IR 대조도 각각 미달이며,
둘을 함께 고친 진단309와 실제 제품310/312의 결과를 구분한다.
실제 전쪽90%와 직접 판독 뒤에만 기존 검사를 원본 HU·점유 관계로 바로잡았다.
기존 한 쪽·세 체크박스·캡션/표/규격 문단 순서·포함 검사는 유지한다.
98% 미만이므로 절대 px 핀은 추가하지 않는다.

## 전쪽 TSV와 직접 판독

| 구분 | TSV | review | overlay |
| --- | --- | --- | --- |
| 수정 전 Native | [TSV](before-native/silhouette.tsv) | [review](before-native/review_001.png) | [overlay](before-native/overlay_001.png) |
| 수정 전 WASM | [TSV](before-wasm/silhouette.tsv) | [review](before-wasm/review_001.png) | [overlay](before-wasm/overlay_001.png) |
| 수정 후 Native | [TSV](after-native/silhouette.tsv) | [review](after-native/review_001.png) | [overlay](after-native/overlay_001.png) |
| 수정 후 WASM | [TSV](after-wasm/silhouette.tsv) | [review](after-wasm/review_001.png) | [overlay](after-wasm/overlay_001.png) |

후의 양쪽 review/overlay를 직접 읽어 표 위 캡션과 표·본문·후행 용지 규격의 순서,
겹침·내용 누락이 없는 것을 확인했다. 굵은 제목의 미세 자형/간격, 일부 가는 괘선의
래스터 차이 및 서명 PUA의 기존 공개 표시 `(인)`과 한컴의 원형 표시 차이는 남는다.
원형 PUA를 일반 괄호로 확장하는 기존 의미 대체를 완전한 glyph 일치라고 쓰지 않는다.

등록된 HWP3 10조건의 재귀 IR 비교311에서 이 원본의2문단만 변경됐고,
다른9조건은 줄 정보·내용·제어 수·소속 순서·쪽수가 모두 동일했다.
IR 대조는9조건의 새 전쪽 PNG 통과를 뜻하지 않는다. 정식 focused 결과와
고정 base 대조는 단계 보고서에 연결한다. 로그·중간 JSON·실행 파일은 ignored output에 보존한다.

## 고정 devel base 추가 대조

고정 `4a07d3ae4dd3255a57fb0166b2b2a2dc335a61fa` Native도 같은 원본·PDF·현재 공급 글꼴로
새 전1쪽을 산출했다.41.27698%,1/1이다. [TSV](base-native/silhouette.tsv),
[review](base-native/review_001.png), [overlay](base-native/overlay_001.png)를 보존하고 직접 읽었다.
기준선과 표 전체가 내려가는 기존 결함을 확인했다. base WASM은 새 공급 프로그램 선택 API가
없어 현재 캡처 도구가 거부했으며, 실제 결과가 없으므로 통과나 같은 프로그램 선택 증거로
기록하지 않는다. frozen base를 공유 target에 재빌드하거나 현재 guard를 제거하지 않았다.
고정 base WASM의 호환 캡처와 전체 corpus 비교는 PR 제출 전 남은 검증이다.
