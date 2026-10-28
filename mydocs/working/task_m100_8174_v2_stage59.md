# #8174 v2-S59 — 문단 서식·줄 메트릭의 요청 예산

2026-10-27, 기준 `6ffd00bc6f`(S58). 기존 통합 승인과 다음 작업 진행 지시에 따라
T04의 문단 서식 producer를 처리했다. 증거는 ignored
`output/pr-review/issue8174/v2-s59-20261027/`에 둔다. 이번 절편의 구현·검증을 완료했으며
T04 전체나 PR 준비 완료는 아니다.

구역의 표 없는 문단 경로가 같은 Storage를 문단 재구성·줄 높이/간격 계산과 파생 구성
복사에 전달하도록 한다. 같은 서식 계산을 일반 API의 Unbounded wrapper에서도 소비한다.
원래 줄 소유·계산 순서·수식 배정·개체 줄 그룹·leading을 보존하며, 예약 실패를 빈 줄이나
정상 반환으로 바꾸지 않는다. 다음 저장 줄 vpos의 임시 관측 상태는 성공·실패 모두 해제한다.

입력 → 구역 조정자 → 문단 서식 → 공통 composer 재구성/줄 메트릭 → 기존 fit/split/place
순서를 따른다. 이번 변경은 컷·수용 높이·페이지 이월 규칙을 새로 계산하지 않는다. 동일한
서식 결과를 기존 측정/배치 소비 지점에 전달하는 연결을 실제 출력·저장으로 검증한다.

문단 배치/표/미주 내부의 일반 State 호출과 항목 producer, cell-unit/index cache,
일부 일반 커닝/shaping·frame helper와 T05–T09는 남는다. host 책임과 논리 작업량/실제
heap·시간 구분, `parserAllocationBoundsProven=false`를 유지한다. 새 조판 golden이나
기대값 완화는 없다. 최종 source의 Native·fresh 최적화 WASM·실제 Worker/Studio·전쪽
시각/직접 판독·저장 보존 결과를 아래에 기록했다.

## 구현 연결과 적용 경계

`section.rs:362 → section/flow.rs:10 → typeset.rs:3853/3896 → paragraph/format.rs:21`
순서로 같은 요청을 전달한다. `recompose_for_flow_with_storage(:302)`는 S52–S54의
공통 composer를 소비하고, `resolve_line_metrics_with_storage(:359)`는 TAC 투영·수식
배정/행 packing·그림 stack과 줄 높이/간격·fallback 배열을 증가 전에 예약한다. 파생
ComposedParagraph와 host 줄 사본·마지막 줄 폭의 스타일 복사도 같은 요청을 쓴다.

서식 결과의 `total_height`·`height_for_fit`·줄 높이/간격은 기존
`typeset_paragraph → paragraph/flow::place`와 인라인 경로에 그대로 전달된다. 이후
fit·split·이월·실제 배치에서 원점을 고르는 조건과 덮어쓰기는 바꾸지 않았다. 이번 수정은
분할/이어받기 정책 변경이 아니며, 모든 분할 분기의 한도 실패를 새로 검증했다고 주장하지
않는다. 저장 줄과 재조판 선택 조건·UTF-16 슬롯 소유·빈 guide 처리도 유지한다.

표 문단의 서식 호출은 아직 일반 wrapper를 사용한다. 표 없는 문단의 서식 계산 이후
배치 상태 producer와 공통 frame·커닝/shaping 등 하위 잔여도 남으며, 이 연결을 전체
문단/표 경로의 할당 상한 증명으로 확대하지 않는다.

## 요청 한도·실패·복구

같은 검사 source·Native64 정책에서 **S58 1 FAIL / 후보 1 PASS**다. S58 초기
작업량에 16 bytes를 더한 CFF `1,633,740`을 입력 정책으로 고정했다. 새로 계상한 문단
작업 때문에 한도가 거절되고, 실패 유지·재사용 거절·새 요청의 같은 쪽수와 HWPX bytes
복구를 통과했다. 최종 decoded 수치나 거절 지점을 기대값으로 고정한 검사는 아니다.

첫 진단은 CFF와 여러 줄 셀에 512-byte 여유를 둬 S58과 후보 모두 성공했고, 거절을
기대한 새 검사가 양쪽에서 실패했다. 실제 측정에서 CFF의 추가 계상은 48 bytes였으며
여러 줄 셀은 이번 일반 문단 서식을 쓰지 않아 차이가 없었다. 이 결과를 보존하고 새 검사
입력 정책을 위 CFF 경계로 정했다. 기존 회귀 기대값·한도·조판 baseline은 바꾸지 않았다.

| 입력 | S58 초기 작업량 | S59 초기 작업량 | 새 제한 |
| --- | ---: | ---: | ---: |
| CFF | 1,633,724 | 1,633,772 | 1,633,740 |
| cell-nested | 1,832,826 | 1,832,826 | 해당 없음 |
| multiline-cell | 7,614,879 | 7,614,879 | 해당 없음 |
| paragraph-origins | 7,648,635 | 7,648,635 | 해당 없음 |

이는 논리 작업량이며 heap/RSS·실행 시간의 증가량이 아니다. 표 경로의 무변동을 그 경로의
할당 보정 완료로 해석하지 않는다. 공개 요청/저장/wire **46 PASS**(XML 2개 포함)를
확인했다. 전체 제품·시각 검증 결과는 아래에 기록한다.

## Native 제품·저장·시각 검증

Core/두 엔진·정규화와 page-layout/page-number **644 PASS / 기존 8 ignored**,
composer/float/model **244**, 텍스트 측정 **38**, serializer/adapter **662**,
Allocation **9**, frame integration **17 PASS**다. 기존 메타데이터 **52개도 후보에서
PASS**다. 동일 source의 S58 실행 로그를 기준으로 대조했으며 이를 새 baseline 실행으로
표시하지 않는다. 저장 **37개**·header/ZIP **22개** 파일은 S58과 bytes 동일하다.
전체/선택 쪽 저장·재열기와 CLI 실패 시 기존 출력 보존·새 요청 복구도 통과했다.

Native 전쪽 **17개·49쪽**, 최저 raw **98.34664%**, 실제 PNG **49개 모두 S58과
bytes 동일**하다. 새 대표 review 5개와 Type3 11쪽 standalone overlay를 직접 판독했다.
CFF 기호·간격, Type3 3쪽 도해/캡션과 11쪽 26–30행·그림/뒤 본문, 중첩 셀 A/B/C/D와
전후 글줄·여러 줄 셀의 소속을 확인했고 새 누락·겹침·배치 차이를 관측하지 않았다. 기존
글자 획·세선의 래스터 차이는 남는다. 별도 Paginator 대조군도 **5개·5쪽 모두 100%**이며
실제 PNG 5개는 S57 실행과 bytes 동일하다. 이 기준은 S58에서 새로 실행한 자료가 아니다.
현재 fallback review 5개도 직접 판독했다.

Native/WASM Clippy `-D warnings`·feature-off·fmt·절편 unit 정책·suite 계약 등을 포함한
**12개 실행이 PASS**다. Native 제품은 root opt-level 1·codegen-units 64의 기능 검증용이며
T08 성능 제품이 아니다. 전체 workspace/Native Skia·성능·최신 devel 보존과 cached devel
대비 기존 source unit 정책의 잔여는 T08/T09에 남는다. 새 source unit·fixture/golden·
조판 기대값 변경은 없다. 최적화 fresh WASM 검증도 완료했다.

## 최적화 WASM과 실제 브라우저

고정 wrapper·codegen-units 16·wasm-opt의 fresh WASM은 **25,339,990 bytes**,
SHA-256 `671e091bddb18694f3be6a94d601a774cf3621b72b70b71cec41fc546314c78d`다.
S58보다 **98,587 bytes / 0.38755% 감소**했다. JS glue bytes는 같고 TypeScript는
InitOutput 멤버의 선언 순서만 바뀌며 서명은 같다. root pkg와 Studio public의 실행용
JS/WASM bytes도 같다. 제품 크기나 논리 작업량으로 로딩·편집 시간/heap 개선을 주장하지
않으며 최종 비용 검증은 T08에 남긴다.

실제 Worker **249 assertions / 14 Workers**, Studio의 활성 문서·draft·recents 보존,
오류/취소 후 복구와 15쪽 문서의 표시·HWP/HWPX 저장도 통과했다. fresh WASM 전쪽은
**17개·49쪽**, 최저 raw **98.79246%**, 실제 PNG **49개 모두 S58과 bytes 동일**하다.
양쪽 출력 모두 누락·측정 불가·90% 미만·쪽수 불일치가 없다. 현재 WASM의 CFF·Type3
3쪽/11쪽·중첩 셀·여러 줄 셀 review 5개와 Type3 11쪽 standalone overlay도 직접 판독했다.
기호/간격·도해/캡션·셀 소속과 전후 본문을 확인했고 새 누락·겹침·배치 차이를 관측하지
않았다. 기존 글자 획·세선의 래스터 차이는 남는다.

`verify_evidence.py`에서 현재 source·최종 검사 source·Native/fresh WASM 제품 hash와
전쪽 TSV/쪽수·직접 판독·저장 증거를 대조했고 통과했다. `evidence-manifest.json`에
증거 hash를 고정했다. 문서 링크·diff 검사도 통과했다. 원격 push·PR 생성·registry
업로드·태그·토큰 열람은 수행하지 않았다.

다음 T04는 표 문단의 인라인 plan/commit과 일반 State의 항목·원장 전달을 우선한다.
표/미주 서식과 셀 유닛/행 색인 캐시·일부 공통 frame/커닝/shaping도 남으며, 이번 서식
연결을 전체 조판 할당의 완료로 확대하지 않는다. T05–T09·최종 source 통합/devel 기능
보존 게이트와 host 책임, `parserAllocationBoundsProven=false`를 유지한다.
