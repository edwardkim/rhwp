# #8174 v2-S57 — TypesetEngine 구역 준비와 쪽 확정의 요청 전파

2026-10-27, 기준 `4a2fe9e2dc`(S56). 기존 통합 승인과 다음 작업 진행 지시에 따라
T04의 주 조판 엔진 진입·확정 경계를 처리했다. 이번 절편의 구현·검증을 완료했으며 T04 전체나
PR 준비 완료는 아니다. 증거는 ignored `output/pr-review/issue8174/v2-s57-20261027/`에 둔다.

실제 `Core::paginate_pass_with_storage`의 기본 경로가 TypesetEngine의 일반 API를 호출해
구역 단 영역·용지 기준 float 증거·머리말/꼬리말과 확정 쪽의 소유 이벤트를 요청 밖에서
만드는 것을 확인했다. `구역 준비 → 원래 format/fit/place → 페이지 확정 → Core 결과`의
호출 순서를 유지하며 같은 Storage를 진입·확정에 전달한다. 원본 컷·소유 유닛·요구/누적
높이·이월·paint 계산은 변경하지 않는다.

머리말/꼬리말 수집은 기존 Paginator와 TypesetEngine이 공통 producer를 사용하게 한다.
페이지 번호·본문 소유 색인·컨트롤 위치와 선택된 중첩 머리말/꼬리말 경로 복사는 S56의
공통 fallible helper를 소비한다. 일반 API는 동일 구현의 Unbounded wrapper로 유지한다.
기본 PDF 요청의 작은 한도 거절·실패 유지·새 요청 복구와 기존 정상 Native/fresh WASM
출력·저장 bytes를 실제 실행해 검증한다. 새 조판 golden이나 기존 기대값 변경은 하지 않는다.

주 엔진의 중간 배치 상태·셀 유닛/색인 캐시 및 일부 일반 helper는 계속 추적할 잔여다.
이번 진입·확정 보정을 전체 내부 할당의 증명으로 바꾸지 않는다. 기존 한도와 host 책임,
`parserAllocationBoundsProven=false`, T05–T09의 최종 통합/PR 게이트를 유지한다.


## Native 검증

같은 Native64 정책·검사 source에서 **S56 2 FAIL / 후보 2 PASS**다. 기본 경로의 CFF와
중첩 셀 요청을 S56 초기 작업량 + 512 bytes로 제한했다. CFF에는 용지 float/머리말/중첩
소유 경로가 없어 추가 구역 단 영역 32 bytes를 지난 뒤 쪽 소유 이벤트에 도달한다.
두 요청의 거절·실패 유지·재사용 거절과 새 기본 요청의 같은 쪽수·저장 bytes 복구를 확인했다.

| 입력 | S56 초기 작업량 | S57 초기 작업량 | 새 제한 |
| --- | ---: | ---: | ---: |
| CFF | 1,626,940 | 1,627,804 | 1,627,452 |
| cell-nested | 1,824,986 | 1,826,906 | 1,825,498 |
| multiline-cell | 7,607,519 | 7,608,959 | 해당 없음 |
| paragraph-origins | 7,641,371 | 7,642,715 | 해당 없음 |

이는 논리적 backing 작업량이며 heap/RSS나 실행 시간 증가율이 아니다. 기존 한도를 올리거나
완료 후 합산으로 예약을 대신하지 않았다. Core는 이미 보유한 normalization Arc를 생성자에
바로 전달해 곧 교체할 빈 overlay의 불필요한 Arc 할당도 만들지 않는다.

공개 요청/저장/wire **43 PASS**(XML 2개 포함), Core/측정/정규화·두 엔진과 page-layout/
page-number **644 PASS / 기존 8 ignored**, composer/float/model **244**, 텍스트 측정
**38**, serializer/adapter **662**, Allocation **9**, 기존 frame integration **17 PASS**다.
메타데이터의 기존 **52개도 후보에서 PASS**다. 입력·기대값을 바꾸지 않았다. S56의 같은
52개 실행과 저장 bytes 증거를 기준으로 사용하며 이를 새 baseline 실행으로 표시하지 않는다.
`librhwp-before.rlib`의 hash가 S56의 실제 Core 측정에 사용한 library hash와 같은 것도 확인했다.

Native 전쪽 **17개·49쪽**, 최저 raw **98.34664%**, 실제 PNG **49개 전부가 S56과
bytes 동일**하다. HWP/HWPX 저장·재열기·선택 쪽·CLI 실패/복구도 통과했고 저장 **37개**와
header/ZIP 정상/거절 **22개** 파일 bytes를 유지했다. CFF·Type3 3쪽/11쪽·중첩 셀·여러 줄
셀의 새 review 5개와 Type3 11쪽 standalone overlay를 직접 판독했다. 기호/줄 간격·도해/
캡션·표/셀의 소속과 뒤 본문을 확인했으며 새 배치 차이는 관측하지 않았다. 기존 글자 획·
그림 세선의 래스터 차이는 남는다. 검증용 Native 제품은 opt-level 1이며 T08 성능 제품이 아니다.

## 셀 유닛·색인 캐시의 실제 도달 조사

코드를 주입하거나 바꾸지 않은 실제 Native library에 GDB symbol breakpoint를 걸었다.
같은 공개 harness에서 17개 PDF·49쪽의 `Session::open → 첫 HWPX/HWP 저장`을 실행했다.
S56과 후보 양쪽에서 `TableRowIndexCache::scope`와 `cell_units_uncached` 심볼 진입점의 breakpoint hit는 각각 **0**이다.
정상 HWP 대조군에서는 각각 **4,378 / 4,210회**를 검출해 breakpoint가 실제 작동함을 확인했다. 인라인 호출까지 포착했다는 증거로 쓰지 않는다.
로그·symbol·probe/library hash는 `cache-route-{before,after}.json`에 보존했다.

이 관측만으로 해당 캐시의 보안 감사를 완료하지 않는다. lowerer는 `page_break=None`을
만들지만 일반 엔진의 극단적 높이 fallback은 `table/block/entry.rs` 끝에서 분할 준비로
이어질 수 있다. 현재 심볼 hit 0회를 모든 입력의 비도달 증명으로 바꾸지 않는다. 일반 문서
경로·공통 셀 유닛/색인 producer의 잔여를 유지한다. 이번 보정은 원래 분할·이월 경로와
컷/높이·원점·소유 규칙을 변경하지 않는다.


공통 producer를 이관한 실제 Native Paginator도 별도 프로세스에서 실행했다. CFF·본문·다단·
중첩 셀·여러 줄 셀 **5개·5쪽 모두 100%**이고 새 PNG **5개가 S56과 bytes 동일**하다.
본문/다단/중첩 셀의 새 review와 중첩 셀 standalone overlay를 직접 판독했다. CFF·여러 줄
셀은 직접 확인한 현재 source의 기본 Native review와 새 fallback review가 bytes 동일함을
확인해 연결했다. 이전 source의 캡처를 현재 판독으로 바꾸지 않았다.


## 실행 경로와 절편 검사 범위

`rendering.rs:5517 → typeset.rs:3706 → section.rs:22`로 같은 Storage가 들어간다.
구역 단 영역과 float/머리말 준비 뒤 중간 format/fit/place와 기존 상태 전환을 그대로
실행하고, `section.rs:475 → state/commands.rs:94 → state/finalize.rs:116`에서 같은
Storage의 쪽 소유 이벤트/번호/머리말 확정으로 돌아온다. 중간의 원래 `force_new_page`,
`flush_column`, `ensure_page` 등 모든 상태 할당까지 연결했다고 주장하지 않는다.

Native/WASM Clippy `-D warnings`, feature-off, fmt·절편 unit 정책·suite 계약 등을 포함한
**12개 실행이 PASS**다. 새 source unit·fixture/golden·기대값 변경은 없다. cached devel
대비 source unit 정책의 기존 잔여와 최종 전체 workspace/Native Skia·성능·devel 보존은
T08/T09에 남는다. 절편의 12개 검사를 그 전체 게이트로 바꾸지 않는다.


## 최적화 WASM과 실제 브라우저

고정 wrapper·codegen-units 16·wasm-opt의 fresh WASM은 **25,339,239 bytes**,
SHA-256 `3e9449df3aae1ec5f42adc8858ca5a36e7de09c72f999712b4c531ea7f263cf9`다. S56보다 **22,266 bytes / 0.08779% 감소**했다.
JS glue bytes는 동일하며 TypeScript는 InitOutput 선언 멤버의 순서만 달라지고 서명은 같다.
root pkg와 Studio public의 실행용 JS/WASM bytes도 같다. 제품 크기로 로딩·편집 시간이나
heap/RSS의 개선을 주장하지 않는다. 최종 비용 확정은 T08에서 한다.

실제 Worker **249 assertions / 14 Workers**와 Studio의 활성 문서·draft·recents 보존,
오류/취소 후 복구와 15쪽 문서의 표시·HWP/HWPX 저장이 통과했다. fresh WASM도 전쪽
**17개·49쪽**, 최저 raw **98.79246%**이며 실제 PNG **49개 모두 S56과 bytes 동일**하다.
Native/WASM 모두 누락·측정 불가·90% 미만·쪽수 불일치는 없다. 현재 WASM의 CFF·Type3
3쪽/11쪽·중첩 셀·여러 줄 셀 review 5개와 Type3 11쪽 standalone overlay도 직접 판독했다.
기호·줄 간격·도해/캡션·셀 소속과 전후 본문을 확인했고 새 누락·겹침·배치 차이는 관측하지
않았다. 기존 글자 획·그림 세선의 래스터 차이는 남는다.


`verify_evidence.py`에서 현재 Rust source·검사 source·제품 hash와 전쪽 TSV/쪽수·
직접 판독·기본/fallback 출력·저장 증거를 대조했고 통과했다. 최종 증거 hash는
`evidence-manifest.json`에 고정했다. 문서 링크와 diff 검사도 통과했다. 원격 push·PR 생성·
registry 업로드·태그·토큰 열람은 수행하지 않았다.

다음 T04 대상은 기존 주 엔진의 중간 `append_item`·페이지/단 flush·이월 상태 backing과
셀 유닛/행 색인 producer, 일부 일반 커닝/shaping·frame metric/helper다. 초기/첫 저장의
기존 비도달 감사 범위는 유지한다. 이번 진입·확정의 보정을 전체 상태 할당의 완료로 바꾸지
않는다. T05–T09와 최종 source 통합·devel 기능 보존 게이트도 남는다.
