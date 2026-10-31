# #8174 v2-S55 — 첫 저장의 보존 메타데이터·손실 보고 예산

2026-10-27, 기준 `ba75e1aa8c`(S54). 기존 통합 처리 승인에 따라 T04의 첫 저장 내부
생산자와 content-loss 수집을 보정한다. 원시 근거는 ignored
`output/pr-review/issue8174/v2-s55-20261027/`에 보존한다. Native·최적화 fresh WASM·실제 Worker/Studio 검증과
전쪽 TSV·대표 PNG 직접 판독을 완료했다. T04 전체나 PR 준비 완료를 뜻하지 않는다.

## 실제 생산·소비 경로

`PdfImportSession::export_hwp/export_hwpx → 같은 Allocation → CFB/ZIP writer →
보존 메타데이터 encoder·ContentLossReport`를 연결했다. 공개 일반 저장은 같은 구현의
Unbounded 호출을 사용하며 편집한 live IR을 교체하지 않는다.

| 경계 | 변경과 유지한 계약 |
| --- | --- |
| 손실 보고 | metadata·BinData·차트 손실의 경로 문자열과 임시/최종 목록 backing을 같은 Storage로 예약한다. 실패하면 저장 오류로 전파한다. 기존 code·subject·reason·path와 보고서/산출물의 수명은 유지한다. |
| 텍스트 클립 | owner count·중복 ID set·기록 배열과 JSON 출력 backing을 예약한다. 경로는 원본을 빌려 직렬화하고 fingerprint는 고정 배열로 만들어 불필요한 깊은 복사를 없앴다. 64단계·4,096건·4 MiB와 소유/유효성 판정을 유지한다. 클립 명령의 숫자 유효성 검사는 같은 조건을 스택에서 확인한다. |
| 링크 원래 서식 | 기존 순서/범위/Command hash를 유지한다. 복사하던 서식과 fingerprint를 borrowed record·고정 배열로 직렬화하고 기록/JSON backing을 예약한다. 공개 Option API의 일반 호출과 bounded 저장의 오류 전파를 구분한다. |
| 원 글꼴 프로그램 | 기존 7개 언어 슬롯·빈 슬롯·header SHA·프로그램 정보를 유지하고 digest/슬롯 벡터·JSON backing을 예약한다. |
| 테두리 그림 소유 | owner count·기록·PNG hash memo와 출력 backing을 예약한다. 재귀 순회는 실패 즉시 반환하며 resource proof의 정상 None과 예약 실패를 구분한다. 같은 JSON 바이트를 digest에 직접 써서 셀마다 만들던 임시 JSON Vec을 없앴다. 원 소유 검증·프레임/pen/compound/raster flag·출력 순서·크기 상한은 유지한다. |
| 컨테이너 연결 | HWP의 위 4개 보조 스트림 경로/목록과 HWPX의 OLE prefix 재부착 backing도 같은 요청으로 예약한다. ZIP/CFB 형식과 payload는 유지한다. |

일반 encoder의 정상 None을 bounded 요청 오류의 fallback으로 쓰지 않는다. encoder의
상위 저장 호출은 Result를 소비하고 실패한 session은 재사용하지 않는다. 내용 손실을
숨기거나 지원 기능·기본 코덱·host 한도를 바꾸지 않았다.

## 새 공개 요청 검사와 보존

같은 검사 source·Native64 cap으로 **S54 2 FAIL / 후보 2 PASS**를 확인했다. 정상 import 후
trusted host가 유효한 텍스트 클립을 추가하고 첫 HWP/HWPX 저장을 실행한다. 이는 합성 입력의
배치 golden이 아니라 공개 저장 수명·예산 경계 검사다. 새 레이아웃 fixture·좌표 핀이나
회귀 기대값 변경은 없다.

| 형식 | S54 전체 저장 decoded | cap(S54 + 64) | 후보 전체 저장 decoded | 산출물 |
| --- | ---: | ---: | ---: | ---: |
| HWP | 1,690,710 | 1,690,774 | 1,692,721 | 4,096 bytes |
| HWPX | 2,049,973 | 2,050,037 | 2,051,778 | 7,419 bytes |

cap은 입력 정책이며 현재 counter의 정확한 기대값을 고정하지 않는다. 제한 요청의 저장
거부·실패 유지·재사용 거절·live 문서/보고서 보존과 새 기본 요청의 같은 바이트 복구를
확인했다. 기본 저장의 손실 JSON은 이전과 같고 텍스트 클립의 한컴 기본 표현 제한도 그대로
보고한다. 수치는 논리적 payload 작업량이며 heap/RSS·실행 시간 증가율이 아니다.

| Native 검증 | 결과 |
| --- | --- |
| 공개 요청·저장·wire | **39 PASS**(XML 2개 포함) |
| 기존 보존 메타데이터 | 텍스트 클립·compound/raster/outside 테두리·소유·링크 편집 **52개가 S54와 후보 모두 PASS**. 실제 편집/되돌리기·양쪽 저장/재열기·유효/손상 소유 반례 |
| Core·측정·typeset·pagination·정규화 | **629 PASS / 기존 8 ignored** |
| composer/float·model, 텍스트 측정 | **244 + 38 PASS** |
| serializer/adapter, Allocation | **662 + 9 PASS** |
| 기존 frame integration | **17 PASS** |
| 실제 header/ZIP | 기존 11개 문서의 정상/거절 **22개 파일 bytes 보존** |
| PDF 저장/재열기 | **17개·49쪽**, 양쪽 34개 저장/98쪽 재열기·상세 3개·선택 4개·CLI 실패/복구 6개. **저장 37개 파일이 S54와 bytes 동일** |
| lint/구성 | Native/WASM Clippy `-D warnings`·feature-off·fmt·절편 unit 정책·suite 계약 등 **12개 실행 PASS** |
| Native 전쪽 | **17개·49쪽**, 최저 raw **98.34664%**. 실제 PNG **49개 전부 S54와 bytes 동일** |

이번 source unit 증가는 없다. 위 절편 정책 통과는 T08의 이전 누적 source unit 이관이나
최신 devel 비교를 대신하지 않는다. 기존 제외/ignored·기대값은 유지했다.

## 남은 호출 경로의 도달 조건 조사

초기 bounded Core 생성의 실제 caller는 `PdfImportSession::open_inner`이며 PDF lowerer가
만드는 IR을 소비한다. lowerer의 font/substitute는 embedded/resolved BinData를 만들지 않고,
초기 `rebuild_embedded_exact_font_sources`가 exact context를 비운 뒤 그 조건에서 반환한다.
커닝은 context를 얻은 뒤 projection을 만들고 horizontal shaping도 두 context를 먼저
요구한다. 목록 마커는 실제 참조한 head type의 None에서 문자열 생성 전에 반환한다.
본문 쪽번호 수렴은 Page/TotalPage AutoNumber와 이전 layout 값이 없으면 반환한다.

이 조건을 기존 **17개·49쪽**의 실제 Core에서 별도로 점검했다. 참조한 목록 head·embedded
exact font·본문 쪽번호·layout 값은 모두 0이다. 템플릿의 **참조하지 않는 번호 스타일**을
실행 경로로 오인하지 않으며 최초 진단의 그 가정은 실행 로그에 보존했다. 실행 관측과
lowerer/guard의 코드 근거를 구분한다. 이것만으로 임의의 host 편집 뒤 하위 helper 전체의
예산을 증명했다고 판정하지 않는다. 기존 공개 편집 수명 검사는 계속 통과한다.

본문 page-field 재분할은 이번에 **현행 bounded 제품 경로의 비도달로 감사 완료**한다.
초기 lowerer는 Page/TotalPage 컨트롤이나 layout 값을 생성하지 않아 `fields → empty return`을
소비한다. 두 첫 저장은 snapshot/serializer를 호출하며 `settle_body_page_fields`나
`paginate_with_storage`로 재진입하지 않는다. host의 일반 편집/재조판은 별도 수명이며,
기존 공개 검사 `initialization_uses_parser_budget_and_editing_owns_its_lifetime`가 이를
확인한다. 따라서 사용하지 않는 HWP 쪽번호 수렴 전체를 바꾸지 않는다. 입력 정책·지원
기능을 축소하거나 필수 실패를 다른 이슈로 넘긴 판정이 아니다. lowerer 또는 bounded
호출자가 향후 이 경계를 열면 같은 감사 계약을 다시 확인해야 한다.

## 후속 판정

root의 locked wrapper·release/codegen-units 16·`wasm-opt`로 최적화 제품을 새로 만들었다.
실제 Worker **249 assertions / 일반 11 + 메모리 실패 3 = 14 Workers**와 Studio의
오류/취소 시 기존 문서/draft/recents 보존·복구, 사용자 PDF 15쪽의 가져오기/표시·양쪽
저장을 통과했다. Worker/Studio와 전쪽 sweep이 받은 WASM hash도 새 제품과 같다.

fresh WASM도 **17개·49쪽**, 최저 raw **98.79246%**이며 **실제 PNG 49개 전부가 S54와
bytes 동일**하다. Native와 함께 누락·측정 불가·90% 미만·쪽수 불일치는 없다. 각 runtime의
CFF 1쪽, Type3 최저 3쪽/대표 11쪽, 중첩 셀·여러 줄 셀의 새 review 5개와 Type3 11쪽의
standalone overlay를 직접 판독했다. 기호·간격·행/열/셀 소속·그림/캡션과 앞뒤 내용에 새
누락·겹침·배치 차이는 관측하지 않았다. 기존 문자 획·그림 세선의 래스터 차이는 남는다.
원시 TSV는 `visual/candidate/{native,wasm}/<문서>/silhouette.tsv`, 새 PNG는
`full-page-reviews/`에 있다. `visual-evidence-audit.json`, 실제 직접 판독 JSON과
`final-evidence-check.log`로 source/product/test와 전쪽 PNG hash를 연결했다. 자동 TSV의
`not_evaluated`나 Worker의 고정 시각 판정 문구를 직접 판독으로 사용하지 않았다.

새 WASM은 **25,338,137 bytes**로 S54의 25,211,976 bytes보다 **126,161 bytes /
0.50040%** 크다. JS ABI glue는 bytes 동일하다. 이는 제품 크기이며 로딩/편집 시간이나
RSS 증가율이 아니다. 실행 시간·메모리 비용은 최종 T08 source에서 확정한다.

일부 일반 helper/커닝·shaping/frame metric 하위 경로와
TypesetEngine/Paginator state·cell-unit/index cache는 T04에 남는다. content-loss/보존 메타데이터는
이번에 보정했고 본문 page-field는 위 도달 조건으로 감사 완료했다.

T05–T09와 최종 전체 회귀·성능·최신 base 기능 보존·포장/소비를 그대로 유지한다.
`parserAllocationBoundsProven=false`와 host 메모리/시간 감독 책임은 바꾸지 않는다.
원격 push·PR 생성·registry 게시·토큰 열람은 하지 않았다.
