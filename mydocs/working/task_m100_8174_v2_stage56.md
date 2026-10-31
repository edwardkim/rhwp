# #8174 v2-S56 — Paginator 상태와 공통 소유 경로의 요청 예산

2026-10-27, 기준 `c912d35464`(S55). 기존 통합 승인과 남은 처리 진행 지시에 따라
T04의 Paginator 페이지/단·분할 상태와 실제 소비하는 공통 producer를 연결한다.
작업 증거는 ignored `output/pr-review/issue8174/v2-s56-final-20261027/`에 보존한다.
이번 절편의 구현·검증을 완료했다. T04 전체나 PR 준비 완료는 아니다.

`PdfImportSession::open → from_parsed_document_with_storage → paginate_pass_with_storage →
Paginator::*_with_storage → PaginationState::*_with_storage`가 같은 요청을 소비한다.
정상 TypesetEngine의 알고리즘을 바꾸거나 새로운 backend/형식별 계산을 만들지 않는다.

- 페이지·단·항목·줄 경계/강제 분할 배열·행 높이 fallback, layout 복사와 방어 횟수/감춤 집합을
  실제 증가 전에 예약한다. 문단/표 fit, 컷·이월·원점·현재 높이·각주 공간·방어 상한은 유지한다.
- 공통 PageLayoutInfo의 단 영역과 Core 사전 측정의 단 영역도 같은 계산과 Storage를 사용한다.
  같은 너비 산술·용지/좌우 여백·쪽/단 방향을 유지한다.
- 페이지 번호 이벤트의 본문 소유 색인·컨트롤 위치·이벤트 배열과 소비 집합, 머리말/꼬리말의
  선택·복사를 연결한다. 원본 줄에 따른 실제 물리 쪽, 번호 발화 순서와 적용 규칙은 유지한다.
- 표 안 머리말·각주/미주의 경로 배열과 모음 backing을 연결한다. 원본 셀/control 순서와
  경로의 마지막 control-index 갱신을 유지한다.
- 인라인 개체의 기하 우선 판정과 원시 UTF-16 슬롯 fallback은 같은 helper를 공유하며,
  요청 실패를 정상 None으로 바꾸지 않는다.
- KoPub 이름 판정은 기존 `hwpx_stored_layout` 조건 안에서만 계산해 PDF 경로의 불필요한
  lowercase 문자열 생성을 피한다. 그 조건이 참인 기존 판정과 조판 계산은 유지한다.

일반 API는 같은 구현의 Unbounded wrapper다. 기존 `RHWP_USE_PAGINATOR=1`은 별도
프로세스에서 실행해 전후 출력·예산 실패·복구를 대조한다. 다른 테스트의 환경변수를
바꾸지 않는다. 테스트/기대값·지원 기능·기존 자원 한도와 host 감독 책임을 유지한다.


## 최종 source와 중간 실행 구분

각주 참조에 경로를 복사하는 마지막 producer도 같은 Storage로 연결했다. 그 전 Native
중간 실행은 `v2-s56-20261027/`에 보존하고, 해당 실행의 소유 프로세스만 종료했다.
최종 source를 `v2-s56-final-20261027/production-freeze.json`으로 새로 고정하고 모든
제품·검사를 다시 실행했다. 중간 바이너리를 최종 head 검증으로 사용하지 않는다.

## 검증 결과

- 같은 검사 source·Native64 cap에서 **S55 2 FAIL / 후보 2 PASS**. 기존 환경변수를
  부모 프로세스에서 바꾸지 않고 자식 프로세스의 실제 fallback을 검사한다.
- 공개 요청/저장/wire **41 PASS**(XML 2개 포함), Core/측정/정규화·Paginator/Typeset과
  기존 page-layout/page-number **644 PASS / 기존 8 ignored**, composer/float/model
  **244**, 텍스트 측정 **38**, serializer/adapter **662**, Allocation **9 PASS**.
- Native/WASM Clippy `-D warnings`, feature-off, fmt·절편 unit 정책·suite 계약 등을 포함한
  **12개 실행 PASS**. 새 source unit·fixture/golden·기대값 변경은 없다.
- Native 기본 **17개·49쪽**, 최저 raw **98.34664%**, 실제 PNG **49개 전부가 S55와
  bytes 동일**하다. 저장 37개·header/ZIP 22개 파일도 bytes를 유지한다.
- 실제 Native fallback은 CFF·일반 본문·다단·중첩 셀·여러 줄 셀 **5개·5쪽 모두 100%**이며,
  실제 PNG 5개가 S55와 bytes 동일하다. 새 review/overlay를 별도로 생성해 직접 판독했다.

Native64 논리적 초기 작업량은 아래와 같다. cap은 S55의 실제 fallback 측정 + 512 bytes로,
추가 Core 단 영역의 32 bytes보다 넉넉하므로 실제 Paginator 내부에 진입한다. cap을 현재
counter의 정확한 기대값으로 고정하지 않으며 시간·실제 heap/RSS 증가율로 해석하지 않는다.

| 입력 | S55 기본/fallback | 후보 기본 | 후보 fallback | 새 cap |
| --- | ---: | ---: | ---: | ---: |
| CFF | 1,626,908 | 1,626,940 | 1,633,740 | 1,627,420 |
| cell-nested | 1,824,954 | 1,824,986 | 1,832,834 | 1,825,466 |
| multiline-cell | 7,607,487 | 7,607,519 | 7,614,887 | 해당 없음 |
| paragraph-origins | 7,641,339 | 7,641,371 | 7,648,643 | 해당 없음 |

두 제한 요청의 거절·실패 유지·재사용 거절과 새 기본 요청의 같은 쪽수/저장 bytes 복구를
확인했다. 최적화 fresh WASM·실제 Worker/Studio·전쪽/대표 직접 판독도 완료했다.

## 분할·확정 경로의 보존

할당 보정은 기존 경로 안에서 수행한다. `paginate_table_control_with_storage`가 기존
MeasuredSection/MeasuredTable을 읽고 fit이면 `place_table_fits_with_storage`, 이월이면
`split_table_rows_with_storage`(engine.rs:2675)를 호출한다. 후자는 기존 row geometry와
보호 블록·행 높이·셀 간격, caption/host/offset을 같은 순서로 예산에 반영한다.
`cursor_row/end_row`, `content_offset/split_end_limit`의 전진·마지막 캡션 처리와 기존
PartialTable 필드는 바꾸지 않는다. 상태는 `advance_column_or_new_page_with_storage →
flush_column_with_storage → new_page_content_with_storage`를 통해 확정된다. layout/zone
복사와 항목·단·쪽의 backing 예약 실패는 상위 Result로 돌아간다.

일반/강제 줄 경계와 다단 경로도 같은 fallible 상태를 호출한다. 쪽 확정은
`finalize_pages_with_storage`(engine.rs:3138)에서 같은 소유 이벤트와 활성 머리말/꼬리말을
소비한다. 기존 Paginator fallback은 TypesetEngine의 cell-unit 컷/overlay continuation을
생성하지 않는다. 이를 새로 구현했거나 두 엔진의 모든 분할을 같게 만들었다고 주장하지 않는다.

기존 Paginator의 `test_table_page_split`, `test_table_split_with_repeat_header`,
`test_partial_paragraph_split`, `test_table_split_10rows_at_page_bottom`,
`test_table_split_50rows_multi_page`, `test_table_split_with_nested_table`를 같은 공통 구현의
일반 wrapper에서 실제 실행해 통과했다. 해당 source unit의 입력·기대값은 바꾸지 않았다.
`test_table_cell_break_intra_row_split`과 `test_table_cell_break_multi_page_row`는
기존 Task #993의 「fallback이 cell-unit 컷을 생성하지 않음」 사유로 ignored이며 PASS나
실행 증거로 세지 않는다. ignored 상태/기대값도 바꾸지 않았다. 이번에 해당 미지원 컷을
구현했다고 주장하지 않는다. 합성 상태의 계약 검사와 원본 PDF 전쪽/실제 fallback 시각
증거를 구분한다.

## 보존 메타데이터와 첫 저장 검증

S55에서 보정한 텍스트 클립·compound/raster/outside 테두리·소유·링크 편집의 기존
**52개가 S55와 후보 모두 PASS**다. 기존 frame integration **17 PASS**, PDF 17개·49쪽의
양쪽 34개 저장/98쪽 재열기·상세 3개·선택 4개와 CLI 실패/복구 6개도 통과했다.
저장 37개·header/ZIP 정상/거절 22개 파일이 S55와 bytes 동일하다.

Native 대표 CFF·Type3 최저 3쪽/대표 11쪽·중첩 셀·여러 줄 셀의 새 review와 Type3 11쪽
standalone overlay를 직접 판독했다. fallback의 일반 본문·다단·중첩 셀 review와 중첩 셀
standalone overlay도 직접 판독했다. fallback CFF·여러 줄 셀의 새 review는 직접 판독한
현재 Native 패널과 bytes 동일함을 따로 기록했다. 줄 간격·기호·그림/캡션과 표/셀/단의
소속·전후 내용을 확인했으며 새 누락·겹침·배치 차이는 관측하지 않았다. 기존 문자 획과
그림 세선의 래스터 차이는 남는다.


## fresh WASM과 실제 Studio

고정 wrapper로 codegen-units 16과 wasm-opt를 적용한 fresh 제품을 새로 빌드했다. 실제
Worker **249 assertions / 14 Workers**와 Studio의 활성 문서·draft·recents 보존,
오류/취소 후 복구, 15쪽 문서의 표시와 HWP/HWPX 저장이 통과했다.
WASM 전쪽도 **17개·49쪽**, 최저 raw **98.79246%**, 실제 PNG **49개 모두 S55와
bytes 동일**하다. 누락·측정 불가·90% 미만·쪽수 불일치는 Native와 WASM 모두 없다.
현재 WASM의 같은 대표 review 5개와 Type3 11쪽 standalone overlay를 직접 판독해
CFF 기호·줄 간격, Type3 도해/캡션·26–30행·후속 본문, 중첩 셀과 여러 줄 셀의 소속·
간격을 확인했다. 새로운 배치 차이를 관측하지 않았다. 기존 문자 획·그림 세선의 래스터
차이는 남는다.

최적화 WASM **25,361,505 bytes**, SHA-256
`ccc420ef1d296a0677f32c10d11a4145982398b076c775129023f98196f374d7`이다.
S55 대비 **+23,368 bytes / +0.09222%**이며 JS glue는 bytes 동일하다. TypeScript 선언의
멤버 순서만 달라졌고 선언과 서명은 같다. 실행에 사용하는 root pkg와 Studio public의
JS/WASM bytes가 같다. 제품 크기를 로딩·편집 시간이나 heap/RSS 변화율로 해석하지 않는다.
Native 제품은 root opt-level 1의 기능 검증용이며 최종 T08 성능 제품이 아니다.

`verify_evidence.py`로 최종 Rust source·검사 source·제품 hash와 전쪽 TSV/실행/직접 판독
기록을 대조했고 통과했다. `evidence-manifest.json`에 최종 증거 hash를 고정했다. 중간
실행과 최종 source를 섞지 않았다. 원격 변경·PR 생성·게시·태그·토큰 열람은 수행하지 않았다.

TypesetEngine 내부 state·cell-unit/index cache와 일부 일반 커닝/shaping·frame metric/helper는
다음 T04 보정 대상으로 남는다. T05–T09와 최신 devel 기능 보존·전체 통합 게이트도 남는다.
기존 한도와 host 감독 책임·`parserAllocationBoundsProven=false`를 유지한다.
