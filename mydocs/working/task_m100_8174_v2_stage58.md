# #8174 v2-S58 — 주 조판 상태의 페이지 생성·단 확정·이월 예산

2026-10-27, 기준 `e5a3027a6b`(S57). 기존 통합 승인과 다음 작업 진행 지시에 따라
T04의 페이지/단 상태 전환 producer를 처리했다. 이번 절편의 구현·검증을 완료했으며 T04 전체나
PR 준비 완료는 아니다. 증거는 ignored `output/pr-review/issue8174/v2-s58-20261027/`에 둔다.

S57의 구역 진입·쪽 번호 확정 사이에서 페이지/단 배열, 앞에 발행할 개체와 현재 항목의
합성 배열, zone/page layout 복사와 이월 개체/각주 적용이 요청 밖에서 할당됨을 확인했다.
기존 상태 전환에 fallible 공통 경로를 마련하고 구역 조정자의 문단 진입·종료와 직접 이월이
같은 Storage를 전달하게 한다. 일반 API는 같은 구현의 Unbounded wrapper로 유지한다.

같은 컷·요구/누적 높이·단/쪽 전환 순서와 정해진 개체/각주 소유를 보존한다. 새 예약 실패는
Result로 전파하고 정상 빈 반환·clamp·출력 생략으로 바꾸지 않는다. 구역의 동일 원점·높이와
최종 출력/저장 보존, 실제 작은 한도의 거절·실패 유지·새 요청 복구를 검증한다.

아직 연결하지 않은 문단/표/미주 내부의 상태 호출과 항목·메트릭 producer, 셀 유닛/색인 캐시를
완료로 표시하지 않는다. 전체 T04·T05–T09 및 최종 source 통합/PR 게이트와 기존 한도·host
책임·`parserAllocationBoundsProven=false`를 유지한다. 새 조판 golden이나 기대값 완화는 없다.


## 실제 상태 전환과 적용 경계

`section.rs:357 ensure_page_with_storage → transition.rs:842 → new_page_content_with_storage
(:1116)`에서 같은 layout 복사와 pages backing을 예약한다. 구역의 종료도
`section.rs:452 → flush_column_always_with_storage(:601) → take_current_page_items_with_storage
(:556)`로 같은 요청을 소비한다. 앞에 발행할 Square 그림·저장 개체, 현재 항목의 순서와
zone/page layout, wrap·inline·origin·overlay 소유 필드는 원래 순서대로 이동한다.
단 배열·쪽 배열·사다리 band 배열은 실제 증가 전에 예약한다.

직접 이월은 `section.rs:251 → advance_column_or_new_page_with_storage(:764)`로 들어가
`flush_column_with_storage(:477) → 단 전환 또는 push_new_page_with_storage(:861)`를
소비한다. 강제 새 쪽·이월 저장 프레임·지연 열 도형/각주 배출도 같은 fallible helper를
호출한다. 시작/끝 컷을 새로 계산하거나 row 요구·수용 높이를 바꾸지 않는다. 원래 동기화·
높이 리셋·overlay 예약 max·각주 구분선과 본문 점유 규칙은 유지한다.

이는 **구역 조정자가 직접 호출하는 상태 전환과 그 하위 producer의 연결**이다. 문단/표/
미주 내부의 일반 API 호출까지 요청 전달을 마친 것은 아니다. 기존 일반 API는 같은
공통 구현에 Unbounded를 전달하며 기존 합성 경계 검사의 입력·기대값은 유지한다.
현재 요청에서 도달한 페이지 생성/종료를 실제 한도로 실행한 증거와 일반 wrapper의 기존
분할·이월 검사를 구분한다. 모든 분할 분기의 한도 실패를 새로 검증했다고 주장하지 않는다.

## Native 한도·복구와 출력 보존

같은 검사 source·Native64 정책에서 **S57 2 FAIL / 후보 2 PASS**다. CFF와 중첩 셀의
S57 초기 작업량 + 512 bytes를 입력 정책으로 고정해, 이전에 통과하던 요청이 새 상태
backing을 예약할 때 거절되는지 검증했다. 거절 뒤 실패 유지·재사용 거절, 새 기본 요청의
같은 쪽수/저장 bytes 복구까지 통과했다. 앞 절편의 한도 값도 그대로 유지했다. 새 producer가
추가되면서 더 앞에서 거절될 수 있음을 기존 검사 주석에 명시했고 기대값을 완화하지 않았다.

| 입력 | S57 초기 작업량 | S58 초기 작업량 | 새 제한 |
| --- | ---: | ---: | ---: |
| CFF | 1,627,804 | 1,633,724 | 1,628,316 |
| cell-nested | 1,826,906 | 1,832,826 | 1,827,418 |
| multiline-cell | 7,608,959 | 7,614,879 | 해당 없음 |
| paragraph-origins | 7,642,715 | 7,648,635 | 해당 없음 |

추가된 논리 작업량은 네 입력에서 각각 **5,920 bytes**다. 기존 페이지/단 backing의 예약을
이제 같은 요청으로 계상한 결과이며, 실제 heap/RSS나 실행 시간의 증가량으로 해석하지 않는다.

공개 요청/저장/wire **45 PASS**(XML 2개 포함), Core/측정/정규화·두 엔진과 page-layout/
page-number **644 PASS / 기존 8 ignored**, composer/float/model **244**, 텍스트 측정
**38**, serializer/adapter **662**, Allocation **9**, frame integration **17 PASS**다.
메타데이터 기존 **52개도 후보에서 PASS**다. 같은 source의 S57 실행을 기준으로 사용하며
이를 새 baseline 실행으로 표시하지 않는다. 저장 **37개**·header/ZIP 정상/거절 **22개**
파일이 S57과 bytes 동일하다. 전체/선택 쪽 저장·재열기와 CLI 오류/복구도 통과했다.

Native 전쪽 **17개·49쪽**, 최저 raw **98.34664%**, 실제 PNG **49개 모두 S57과 bytes
동일**하다. 현재 Native의 CFF·Type3 3쪽/11쪽·중첩 셀·여러 줄 셀 review 5개와 Type3
11쪽 standalone overlay를 직접 판독했다. 기호/줄 간격·도해/캡션·표/셀의 소속·뒤 본문을
확인했으며 새 누락·겹침·배치 차이는 관측하지 않았다. 기존 글자 획과 세선의 래스터 차이는
남는다. Native 제품은 root opt-level 1의 기능 검증용이며 T08 성능 제품이 아니다.

Native/WASM Clippy `-D warnings`, feature-off, fmt·절편 unit 정책·suite 계약 등을 포함한
**12개 실행이 PASS**다. 새 source unit·fixture/golden·조판 기대값 변경은 없다. 전체
workspace/Native Skia·성능·cached devel 대비 기존 unit 정책의 잔여와 최신 devel 기능
보존 게이트는 T08/T09에서 계속 처리한다. 최적화 fresh WASM의 실행·시각 검증도 완료했다.


## 최종 제품과 실제 브라우저

고정 wrapper·codegen-units 16·wasm-opt의 fresh WASM은 **25,438,577 bytes**,
SHA-256 `2a28268776ee52c40450a07041037efece465ba135b6ed41c4dd955f9b9017ba`다. S57 대비
**+99,338 bytes / +0.39203%**다. JS glue bytes는 같고 TypeScript는
InitOutput 멤버의 선언 순서만 바뀌며 서명은 같다. root pkg와 Studio public의 실행용
JS/WASM bytes도 같다. 크기 변화로 로딩·편집 시간이나 실제 heap/RSS의 변화를 주장하지
않는다. 최종 비용 확정은 T08에서 한다.

실제 Worker **249 assertions / 14 Workers**와 Studio의 활성 문서·draft·recents 보존,
오류/취소 후 복구, 15쪽 문서의 표시·HWP/HWPX 저장도 통과했다. fresh WASM도 전쪽
**17개·49쪽**, 최저 raw **98.79246%**, 실제 PNG **49개 모두 S57과 bytes 동일**하다.
양쪽 출력 모두 누락·측정 불가·90% 미만·쪽수 불일치가 없다. 현재 WASM의 CFF·Type3
3쪽/11쪽·중첩 셀·여러 줄 셀 review 5개와 Type3 11쪽 standalone overlay도 직접 판독했다.
기호/간격·도해/캡션·셀 소속과 전후 본문을 확인했고 새 누락·겹침·배치 차이는 관측하지
않았다. 기존 글자 획과 세선의 래스터 차이는 남는다.


`verify_evidence.py`로 현재 Rust source·검사 source·Native/fresh WASM 제품 hash와
전쪽 TSV/쪽수·직접 판독·저장 증거를 대조했고 통과했다. 최종 증거 hash는
`evidence-manifest.json`에 고정했다. 문서 링크와 diff 검사도 통과했다. 원격 push·PR 생성·
registry 업로드·태그·토큰 열람은 수행하지 않았다.

문단/표/미주 내부의 상태 API 호출과 항목·메트릭 producer, cell-unit/index cache와 일부
일반 커닝/shaping·frame helper는 다음 T04 대상으로 남는다. 이번 구역 직접 호출의 보정을
전체 상태 할당의 완료로 확대하지 않는다. T05–T09와 최종 source 통합·devel 기능 보존
게이트도 남으며 기존 한도·host 책임과 `parserAllocationBoundsProven=false`를 유지한다.
