---
kind: plan
status: active
canonical: mydocs/plans/issue_7639_remaining_performance.md
last_verified: 2026-10-13
---

# #7639 남은 성능 개선

Issue: #7639

## 기준과 승인 범위

작업지시자가 #7639 남은 성능 개선 처리를 지시했다. base는 최신 devel `22317614b03722e95349a1d3d71c86923363c6f5`다. 원 기여자의 두 반복 검사 개선은 #7877에 반영됐고 추가 열린 관련 PR은 없었다. 담당자를 메인테이너로 지정하고 별도 local worktree에서 진행한다. 원격 코드 push/PR/comment는 아직 승인·실행 범위에 포함하지 않는다.

## 독립 계약과 구현 순서

1. 현행 코드를 실제 제품 진입점으로 측정하고 전쪽 SVG·쪽 소속을 보존한다. CLI open과 DocumentCore open/셀 한 글자 삽입은 별도 시간이며 동일 환경·입력·profile·횟수로 재측정한다.
2. `typeset/table/block/prepare.rs`의 rowspan_touched를 행마다 전체 셀 탐색하는 O(rows*cells)에서 셀의 시작/끝 이벤트와 행 prefix로 계산하는 O(rows+cells)로 바꾼다. 기존 `row_span>1 && start<=row && row<start+span` 구간을 정확히 보존하고 queried row_count는 MeasuredTable 범위를 사용한다. 빈/0span/범위 밖/겹치는 span도 기존 predicate와 같아야 한다. 이 단계에는 재사용 캐시가 없다.
3. 행 시작 셀·단일 행 컷 슬롯(안정적 col 순서)·rowspan 후보의 원본 source 순서를 구분한 읽기용 색인을 구성한다. 무효 선언·중복 col·원본 cell index도 기존 탐색의 의미를 유지해야 한다. 모든 span을 모든 행에 복제해 메모리를 키우지 않는다.
4. raw row heights와 rowspan shrink 재계산을 해당 불변 source snapshot 범위에서 공유한다. float 변환/합산 순서는 그대로 유지한다. 이후 줄/내용 조건은 원본 문단과 실제 normalization을 계속 읽는다.
5. 포인터 주소만으로 일시 clone/정규화 표를 섞지 않는다. source owner와 immutable snapshot의 실제 생명주기를 확인한 뒤 cache 경계를 정한다. 초기/편집 재조판, 문서 교체, 구조 편집, 설정 변경, Undo/Redo의 무효화 경로를 검증한다.

## 호출 경로

- rowspan_touched producer → 온전한 행 높이 선택·rowspan 행 보호 → 블록 준비/scan 예약 → 실제 분할/paint. 값과 배치 정책은 바꾸지 않는다.
- row_cut_cell_order → RowCut 슬롯 소속 → 컷 측정·실제 셀 배치·저장 프레임 writeback. col 정렬의 안정성과 원본 셀 인덱스를 보존한다.
- whole_fragment_row_uses_measured_height / row_uses_reflow_physical_frame → 예약 높이 선택·physical frame → scan/emit/layout. 새로운 예외나 좌표 보정 없이 탐색 범위만 좁힌다.
- get_raw_row_heights / rowspan_declared_overflow_shrink → native saved frame·source prefix·straddle continuation 수요 → 실제 cut/배치. helper의 조건·반환값·부동소수 합산 순서를 변경하지 않는다.

## 검증과 알려진 차이

초기 공개 샘플 `samples/issue2063_huge_cellbreak_table.hwp`는 current161쪽, 기존 독립 PDF 두 개는162쪽이다. 성능 결과와 한컴 조판 완료를 구분하며 사용자에게 기존 조판 차이의 처리 범위를 문의했다. 샘플 경로를 안내했고 아직 범위 선택은 받지 않았다. 이 차이를 허용치/기대 쪽수로 숨기지 않는다.

동일 전쪽 출력 비교는 성능 최적화의 의미 보존 증거이고 한컴 일치 증거를 대신하지 않는다. 원본·독립 Print·Native/fresh Docker WASM gate와 확대 판독, cache 반례, formal 회귀·세 Clippy 등 제출 gate는 별도로 충족한다. 현재 측정·로그·프로브는 기본 작업공간 ignored `output/pr-review/issue7639-perf-20261013/`에 보존한다.

## 첫 보정 관측과 색인 수명 계약

첫 rowspan event 보정은 span 순수 계약 2건과 전체 161쪽 SVG 바이트 동일성을 통과했다. API baseline(워밍업 제외 2회)의 open 10.5399–10.6043초 / insert 8.3774–8.4745초 대비 첫 보정은 open 10.5559–10.5647초 / insert 8.1658–8.1845초로 큰 개선을 입증하지 못했다.

색인은 RowMembers의 원본 순서와 안정적 col 정렬을 공유한다. 캐시에는 Weak만 두고, strong 소유자는 항상 해당 불변 Table borrow를 함께 보유하는 TableRows guard로 제한한다. 따라서 원래 Table이 소멸·이동·변경 가능해진 뒤에는 색인 strong 소유자도 없어 주소 재사용이 hit가 되지 않는다. Arc raw-height 값을 외부에 복제해도 원래 RowIndex를 소유하지 않는다. scope 밖은 원래 탐색으로 fallback한다. 별도의 주소 지문·문서 ID·첫/끝 셀 속성으로 같은 표를 추측하지 않는다.

scope는 블록 prepare, continuation 동기 drain과 각 step, 온전한 표 paint와 partial-table 실제 row-domain 선택 뒤에 연결한다. 임시 정규화·중첩 표의 기하 대상이 원본과 다르면 각 실제 Table에 별도 scope를 둔다. raw heights와 declared shrink는 원래 Table 메서드를 동일한 불변 borrow에서 lazy 1회 호출하며 pixel 변환·합산·정책 판정은 기존 consumer에 남긴다.

## 행 색인 보정의 관측

행 색인과 블록 준비의 행별 검사 두 곳을 연결한 phase3 Native build는 성공했다. 동일 입력·환경·release-test profile에서 워밍업 제외 2회 측정은 open 5.4427–5.4487초 / insert 4.1007–4.1080초다. baseline 대비 약 48% / 51% 감소지만 두 번의 국소 측정이므로 다른 기기·문서의 일반 성능을 보장하지 않는다. 삽입 시간은 DocumentCore 셀 편집과 재조판이며 저장·브라우저 paint 시간은 포함하지 않는다.

phase3의 전체 161쪽 SVG는 baseline과 페이지별 SHA-256 및 파일 집합이 모두 같다. 이는 기존 출력 보존 증거이며 독립 한컴 162쪽 출력과의 일치를 뜻하지 않는다. 증적은 ignored `output/pr-review/issue7639-perf-20261013/phase3/`의 `source-hashes.json`, `provenance.json`, `edit-probe.jsonl`, `svg-parity.json`에 보존한다. rowspan 순수 계약과 #6123·#6756·#7114 focused 검사 파일 4개는 모두 통과했다. 구조 편집·문서 교체·Undo/Redo, 전체 회귀, 세 Clippy, Docker WASM 및 독립 Print 시각 게이트는 아직 완료하지 않았다.

## Studio 지연과 수명 계약 보강

사용자가 요청한 devel `22317614b`의 Native Skia / Docker WASM 빌드는 별도로 성공했고 기본 작업공간에서 유지한다. 이 WASM을 Chrome 155 CDP의 실제 Studio에서 공개 샘플로 확인했을 때, helper의 문서 로드부터 Canvas 표시까지 23.908초였다. 두 번째 상세 계측은 22.094초이며 문서 로드 10.872초, 글꼴 대기 0.190초, Canvas view 준비 10.993초였다. Canvas 내부에서 요청 수집 0.188초, 응답 등록 0.0001초, `selectCanvasMetrics`의 재조판 9.445초, 실제 5회 layer tree 조회 합계 1.089초를 관측했다. 다운로드나 CDN 지연으로 오인하지 않는다. 이 helper 계측은 메뉴·대화상자의 모든 시간을 포함한 실제 사용자 클릭 벤치마크는 아니다.

경로는 `WasmBridge.loadDocument → HwpDocument(data) → 최초 조판`과 `CanvasMetricSession.prepare → selectCanvasMetrics → DocumentCore.rebuild_derived_state → Canvas 조판`이다. backend별 메트릭을 보존해야 하므로 두 번째 조판을 임의로 생략하지 않는다. 행 색인 보정은 두 경로가 공유하는 반복 탐색에 적용한다. 첫 측정·상세 측정 및 PNG는 ignored `output/pr-review/issue7639-perf-20261013/browser/`에 보존한다.

코드 검토에서 Drop glue만 있는 guard는 NLL이 마지막 필드 사용 뒤 Table borrow를 끝내도록 허용하여, Arc 소유자가 남아 있는 동안 수정하는 코드가 컴파일될 수 있음을 확인했다. 현행 호출은 불변 Table 인자를 동기적으로 사용하지만 제시한 일반 수명 계약에는 이 보호가 부족했다. TableRows에 명시적인 Drop을 추가해 strong owner의 종료까지 drop checking이 borrow를 유지하도록 보강했다. 실제 table_rows 모듈을 사용하는 독립 컴파일 프로브에서 기존 구조는 수정 허용, 보강 구조는 E0502, 명시적 guard drop 뒤 수정은 허용이었다. 이는 수명 계약 검사이고 조판 결함의 수정 전 FAIL 증거로 보고하지 않는다.

보강을 위해 첫 candidate Docker 컴파일을 직접 중지했다(exit 137; OOM 실패가 아님). 해당 로그는 phase3의 `lifetime-review-aborted-*`로 보존했다. 보강 후 Native Clippy는 통과했으며 phase4 Native·Docker WASM 재빌드와 성능·출력 보존 재확인을 진행한다. phase3 시각·시간 결과를 phase4의 완료 증거로 그대로 적용하지 않는다.

## 보강본의 실제 WASM 관측

phase4 Native / Docker WASM 빌드는 성공했다. 브라우저가 실제 받은 WASM 해시는 devel `7a12583cc041376ee2dcf8986dab451059e52a3669c5a47df4eb669233cccc32`, 보강본 `f97bae32f29de71797743d3961f96068098602541dbeca3cccae379cdc6527dd`이며 각 빌드 증적과 일치한다. 기본 작업공간 devel과 7700 서버는 유지하고 보강본은 별도 7796 Studio에서 실행했다.

| 관측 경로 | devel | phase4 보강본 |
| --- | --- | --- |
| 같은 Chrome 155 / Canvas2D에서 첫 표시(각 2회) | 23.775–23.910초 | 11.369–12.906초 |
| 첫 조판을 포함한 문서 로드 | 12.165–12.199초 | 5.593–6.722초 |
| Canvas 메트릭 선택의 재조판 | 9.478–9.493초 | 4.173–4.262초 |
| 같은 셀에 한 글자 삽입 API(1회) | 8.219초 | 3.935초 |
| 해당 API 삽입부터 view 재표시까지(1회) | 9.645초 | 5.336초 |

서로 다른 두 Native 실행의 API open 5.523–5.863초 / insert 4.051–4.167초도 다시 확인했다. 이는 사용자 메뉴·키보드 입력·IME의 모든 경로를 포함한 일반 성능 보장은 아니다. 소수 회 측정의 환경·JIT·font callback 편차를 고려하며, `fonts_ms`에는 글꼴 준비 중 일어난 Canvas 재조판 callback 시간이 포함될 수 있어 CDN 지연으로 해석하지 않는다. 메트릭 선택을 생략하거나 표시 backend를 바꾸지 않았다.

phase4의 전체 Native 161쪽 SVG 파일 집합과 페이지별 바이트는 baseline과 같다. 같은 원본 첫 페이지의 실제 Canvas 버퍼(1123×794)는 devel / phase4의 픽셀이 완전히 같았다. 앞서 element screenshot의 차이 6,352px은 문서 내용이 아닌 왼쪽 8열의 전체 높이 capture band에 한정됐으며 raw Canvas 비교로 구분했다. 이 결과는 출력 보존 증거이며 독립 한컴 162쪽 문서의 일치나 전쪽 Canvas 검증을 대체하지 않는다.

phase4에서 기존 focused 검사 4개 파일 / 10건이 모두 통과했다. 증적은 `output/pr-review/issue7639-perf-20261013/phase4/`와 `browser/`에 있으며 `build-provenance.json`, `source-hashes.json`, `svg-parity.json`, `focused-tests.json`, `*-open.json`, `*-edit.json`, `raw-canvas-parity.json`으로 연결한다. 새 원본 확인 탭은 `http://localhost:7796/?review=issue7639-candidate-review`다.

완료하지 않은 범위는 전체 Rust 회귀, 전체 Rust lint 묶음, 구조 편집·문서 교체·Undo/Redo의 추가 실행, 독립 Print에 대한 전체 Native/fresh WASM TSV 및 확대 판독이다. 기존 161/162쪽 차이의 처리 범위도 아직 확정되지 않았다. 이 상태에서 PR 생성·push·이슈 종료를 실행하지 않는다.

## 스크롤과 캐럿 클릭 지연의 재현·보정

사용자는 7796의 첫 표시가 빨라졌지만 마우스 스크롤과 캐럿 클릭이 오래 기다린다고 확인했다. 실제 Chrome에서 표 안 첫 클릭이 51.676초를 기다렸고, 그중 `getTableCellBboxes` 한 번이 51.597초였다. 이는 최초 조판과 별개인 메인 스레드 정지다. 기존 `page_hint`는 정확한 한 페이지가 아니라 탐색 시작점이므로, hover가 161쪽 표의 미방문 조각까지 렌더 트리를 준비했다.

계약은 조회 대상과 실제 소비 범위를 일치시키는 것이다. `table.rs`의 셀 수집 결과 → 페이지 단위 Native/WASM API → `WasmBridge.getTableCellBboxesAtPage` → `ensureTableCellBboxCache` → 현재 쪽 `pageBboxes`를 소비하는 hover/리사이즈 경로로 연결한다. 기존 전체 표 API와 페이지 API는 같은 `find_table_cells`를 사용해 모델 셀 인덱스, 조각의 페이지 소속, 좌표와 직렬화 순서를 보존한다. 다른 페이지로 이동하면 그 페이지 조각을 조회한다. 빈 결과·오류를 전체 표 조회로 확대하지 않는다.

F5 하이라이트·Tab 읽기 순서·표 전체 선택은 기존 전체 표 API를 유지한다. 중첩 표의 ByPath API도 유지하며 해당 경로의 성능 개선은 이번 보정으로 입증하지 않는다. 기존 WASM과의 호환 fallback은 명시적으로 남기되 실제 보정 검증은 새 Docker WASM에서 새 API 호출을 확인한다. `getPageControlLayout`의 child 순번을 원본 셀 인덱스로 추측해 대체하지 않는다.

스크롤의 별도 병목은 Canvas font setter에서 같은 CSS font를 그릴 때마다 fallback 목록을 다시 해석하는 비용이었다. `CanvasFontResolutionScope`는 정확한 font 문자열을 키로 **동기 paint callback 내부**에서만 기존 해석 결과를 재사용한다. 다음 paint·중첩 host/portable scope·await 뒤에는 새로운 정책을 읽는다. 실제 native font setter, 글꼴 준비와 Canvas 메트릭 재조판은 생략하지 않는다. TS 보정의 예비 페이지 이동 5회에서 paint 최댓값은 135.9ms → 13.9ms, 최대 RAF 간격은 187.6ms → 70.8ms였다. 이 실행의 트리는 이미 준비된 상태였으므로 미방문 페이지의 전체 비용으로 보고하지 않는다. 새 문서의 미방문 쪽 이동을 별도로 확인한다.

캐시 계약 검사 2건은 기존 구현에서 실제 assertion 실패, 보정 구현에서 통과했다. 새 font scope 계약 4건과 기존 관련 검사를 합친 focused 33건은 통과했다. Studio 전체는 1,840건 통과·실패 0·기존 skip 2였다. Native Clippy와 WASM lib Clippy는 통과했다. 새 export의 TypeScript 확인은 새 Docker build의 타입 생성 뒤 수행한다. 첫 클릭 지연, 전쪽 Canvas 버퍼 보존, 페이지 조회와 기존 전체 조회를 필터한 결과의 일치는 새 artifact에서 이어서 검증한다. 증적은 기존 ignored 경로의 `browser/scroll/`와 `phase5/`에 둔다.


phase5 Docker build는 성공했고 실제 Chrome의 WASM SHA는 `03e7e558007c93eaa87777467012455e83416b8ea2d18bc99ee9c8227885a89e`다. root pkg/public 해시도 같으며 TypeScript 검사와 세 Clippy, workspace build, manifest base 비교, fmt가 모두 통과했다. 같은 첫 표 클릭은 51.676초 → 78ms이고, 해당 조회는 전체 표 대신 페이지 API 1회·0.3ms였다. 캐럿이 실제 cell 27로 이동했음을 확인했다. 이후 두 표 안 클릭은 61ms와 59ms였다(다른 한 번은 페이지 밖이므로 셀 클릭의 증거로 세지 않는다).

전체 161쪽 Canvas raw RGBA SHA-256은 TS 보정 전과 모두 동일하다. 현재 쪽 0·80·130·마지막 쪽의 페이지 조회 결과는 기존 전체 조회를 같은 pageIndex로 필터한 결과와 모델 인덱스·rowspan·좌표·순서 모두 같았다. 존재하지 않는 표는 빈 배열, 범위 밖 페이지는 오류였다. `page-query-canvas-parity.json`, `page-query-contract.json`, `click-page-query.json`과 phase5 source/build provenance로 연결한다. 원본 모델 인덱스 0인 반복 제목 셀을 포함한 분할 조각도 그대로 유지한다.

실제 CDP 마우스 휠은 스크롤 위치를 전진·복귀시켰지만, 미방문 쪽 이동은 최대 1.157초·RAF 1.102초가 남았다. paint 최댓값은 13.7ms였고 `getCanvasPageLayerTree`의 cold build가 최대 405.5ms였다. 이를 완료로 보고하지 않는다. 별도 Native 프로브에서 원본의 5,277행·52,694셀에 대한 `invalid_declared_width_rows`만 335.6–375.9ms를 사용했다. 행마다 모든 셀을 다시 검색하는 순수 선언 검사이므로, 다음 단계에서 규칙·정렬·오류 판정은 유지하고 탐색을 개선한다. 이 단계는 아직 전체 Rust 회귀·Native Skia·독립 Print 시각 게이트를 완료하지 않았으며 원격 반영하지 않는다.


## 선언 너비 탐색과 최종 상호작용 관측

`Table::declared_width_rows_exceeding`은 단일행 셀을 원본 행별로 한 번 모은 뒤 기존 안정적 열 정렬과 완결성·허용 오차·정수 합산을 그대로 적용한다. 선언 영역 밖의 행·세로 병합·빈 행·겹친 열·0 span의 판정은 바꾸지 않는다. 이 결과를 소비하는 `compute_column_widths` / `LayoutEngine::resolve_column_widths`와 최종 partial/whole table 격자는 그대로 둔다. cache 수명이나 문서 ID 조건을 추가하지 않는다.

같은 공개 원본의 순수 너비 검사 5회는 335.6–375.9ms → 3.07–3.57ms였으며 invalid 행 집합은 동일했다. 새 source query 계약 2건은 허용 오차의 경계, 불완전·중복 열, 병합 열, 영역 밖 행, 세로 병합과 희소 행을 확인한다. 조판 위치·쪽수·golden을 고정하는 검사가 아니다. 수정 전의 같은 질의와 성능을 비교한 것이며 기존 코드의 의미 오류를 검출했다는 주장으로 바꾸지 않는다.

최종 Docker WASM SHA는 `c3934165ae0e58e3207acf00a354a376579536c30bb487dc876d040363869256`이며 실제 Chrome 응답, root pkg/public 일치를 확인했다. 같은 첫 표 클릭은 74ms(이후 표 안 클릭 60ms), 페이지 경계 조회는 0.5ms였다. 실제 CDP 마우스 휠 최대 RAF 간격은 직전 보정본 593.4ms → 93.7ms이고 100ms 초과가 없었다. 미방문 쪽 이동은 고정 200ms 대기와 두 RAF를 포함한 최대 1.157초 → 320.4ms, cold layer tree 최댓값은 405.5ms → 74ms였다. 한 번의 스크롤 측정이며 모든 장치의 상한을 뜻하지 않는다.

첫 표시 첫 측정은 Native 컴파일과 겹친 구간에서 19.648초(load 14.747초)였다. 컴파일 종료 후 재측정은 10.617초(load 5.981초, view 4.366초)다. JIT·메모리·실행 부하를 분리 측정하지 않았으므로 차이의 원인을 단정하지 않는다. 이 helper는 메뉴·대화상자 전체 시간을 포함하지 않는다. 메인터너는 준비된 7796 탭의 실제 마우스 휠·캐럿 클릭을 확인하고 **“지연이 개선되었습니다”**라고 판정했다. 이를 독립 Print의 시각 판정으로 해석하지 않는다.

최종 161쪽 Native SVG 파일 집합과 바이트, WASM Canvas raw RGBA 해시는 모두 변경 전과 같다. 페이지 조회는 기존 전체 조회를 같은 페이지로 필터한 배열과 같으며, 전체 조회도 47.090초 → 7.429초였다(후속 호출 순서에 따른 cache 상태가 있어 일반 상한으로 쓰지 않는다). `phase6/svg-parity.json`, `browser/scroll/row-width-canvas-parity.json`, `page-query-contract-row-width.json`, `click-row-width.json`, `real-wheel-row-width.json`, `cold-row-width.json`에 연결한다.

`hwp_table_test.hwp`에서 실제 file input 열기 → `executeOperation(snapshot)` 행 삽입 → `performUndo` → `performRedo` → file input 문서 교체를 실행했다. 모델 행수 4 → 5 → 4 → 5 → 4와 모델 셀 소속, 새 셀 보존을 확인했고 교체 후 UI bbox 캐시·표 참조·undo history는 초기화됐다. 실제 키보드 Ctrl+Z / IME를 검증했다는 의미는 아니다. 첫 background tab은 초기화 대기에서 timeout하여 실행 증거로 세지 않았고 foreground 재실행을 사용했다. 로그와 원문은 `cache-state-background-timeout.log`, `cache-state.json`에 보존했다.

새 integration source가 처음에는 파생 suite에 없는 상태여서 최초 fmt 적용이 그 파일을 놓쳤다. prepare 후 포맷을 보정했고, 그 바이트 변경으로 달라진 파생 배정을 다시 준비하여 fmt와 manifest를 함께 통과시켰다. 첫 실패 로그를 보존하며 generated suite·manifest는 source PR에 포함하지 않는다. production Rust/TS 해시는 재빌드·전쪽 출력 검사 때와 그대로다. 포맷 변화는 새 source query 검사 한 파일에 한정되므로 같은 Docker WASM을 재사용한다. 최종 세 Clippy·workspace build·정책 검사는 통과했고 전체 Rust/Native Skia 결과는 이어서 기록한다.


## 전체 검증과 남은 제출 조건

2026-10-14까지 같은 production source hash에서 전체 nextest는 **10,223건 통과·실패 0·skip 45**였다. compile 포함 1,403.9초, 실행 493.1초였으며 호스트 16 CPU / 31GiB 메모리와 다른 작업을 고려해 Cargo build jobs 2, test threads 4를 사용했다. Native Skia lib는 root 3,864건과 내부 crate 15·165·2건, 합계 **4,046건 통과·ignored 12**였다. placeholder 2건과 direct PDF 4건도 모두 통과했다. `phase6/validation-regenerated.json`과 각 로그를 보존한다. fmt·세 Clippy·workspace build·manifest 및 unit-tier base 비교, TypeScript·Studio 전체 검사도 통과했다. 앞서 실패한 fmt·manifest는 원본 형식 보정과 파생 재준비로 해결했으며 허용치·golden·검사 조건을 완화하지 않았다.

새 독립 정답지를 같은 원본에서 다시 생성했다. 현재 입력은 `hancom-office-2018` / `10.0.0.14727` 저장본이므로 MCP engine 2020을 명시했다. job `c660225b-da83-4358-b658-18296e0c36d8`의 결과는 한컴 `11.0.0.9136`, `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, `input_preprocess=none`, font scope verified였다. 새 PDF도 **162쪽**이며 SHA-256은 `32ecd01c0e82abb5d89b42edf29465f252d922ef78f98cd23a13edfd0b524ddf`다. ignored `oracle/pdf/issue2063_huge_cellbreak_table-2020.pdf`와 start/status/download 원문(접속 정보 제외)에 보존했다.

현재 Native는 Windows Fonts의 실제 함초롬바탕·돋움/돋움체·HY견명조 등을 공급해도 161쪽이었다. 전쪽 silhouette 실행은 `svg=161, tree=161, pdf=162`에서 중단됐고 TSV가 완성되지 않았다. 처음 WASM 실행은 Linux Chrome 경로 미지정으로 실행하지 못했으며 실제 설치된 Chrome 154 경로를 명시해 다시 전쪽 실행했다. 선택 1..161의 추가 진단은 페이지마다 전체 문서를 다시 조판하는 비용이 있고 162쪽 누락을 해결하지 못하므로 본 에이전트 소유 실행만 중지했다. 공통 161쪽 비교를 원문 전쪽의 통과로 바꾸어 보고하지 않는다.

따라서 본 단계의 **성능 보정과 변경 전후 출력 보존은 충족**, 전체 Rust/Skia/lint는 충족이며, **독립 Print 전쪽의 시각 게이트는 쪽수 불일치·TSV 미완성으로 미충족/미검증**이다. 메인터너의 “지연이 개선되었습니다”는 성능 판정이다. 신규 PR 생성·push·merge·이슈 종료를 진행하지 않는다. 현재 worktree와 ignored output의 소스·WASM·검증 자료를 유지하며, 공용 target과 원래 devel 7700 서버는 변경·삭제하지 않는다.


Chrome 154를 명시한 최종 fresh WASM 전쪽 Sweep도 `svg=161, tree=161, pdf=162`로 종료했다. Native와 WASM 모두 누락 쪽을 제외하거나 빈 쪽을 합성해 TSV gate를 통과시키지 않았다. `oracle/native-sweep.log`, `oracle/wasm-full-sweep.log`, 각각의 `run_manifest.json`과 WASM `manifest.json`에 전쪽 출력·실행 실패를 기록했다. `phase6/build-provenance.json`의 production 해시와 실제 Studio artifact가 일치한다. 검증 완료 소스는 성능 보정 local commit으로 보존하며 시각 게이트 해결 전 원격에 제출하지 않는다.


## 최신 devel 병합과 통합본 재검증 (2026-10-14)

사용자는 원격 변경 확인 뒤 “PR 생성 전 머지 후 진행합니다”라고 지시했다. 기본 작업공간의 clean한 로컬 devel은 `22317614b`에서 `07e7e813d06d3f8031e86eb7c27233b829964907`로 fast-forward했다. 같은 원격 SHA를 현재 성능 branch에 merge했고 commit은 `5eb992843`이다. 원격 push나 PR 생성은 수행하지 않았다.

충돌은 `src/renderer/typeset/table/block/prepare.rs`의 한 구간이다. 원격의 `remember_stored_frame_rows(row_geometry_table, &mt.row_heights)`를 유지하고, 그 앞에 불변 geometry/host Table의 행 색인 guard 두 개를 함께 둔다. 저장 프레임 계획은 원격의 측정 높이에서 생산되어 후속 행 컷 소비 경로가 읽고, 성능 색인은 같은 불변 Table의 원본 셀 순서·안정적 열 정렬·raw 높이만 재사용한다. 성능 색인이 원격의 저장 프레임 높이를 덮어쓰거나 다시 계산하지 않는다. 나머지 겹친 다섯 파일은 자동 병합된 diff를 원격 base와 대조했다.

기존 Studio 생성 JS는 경로 한정 stash로 먼저 보존했고 merge 뒤 적용했다. 새로운 root pkg/public은 통합 source에서 Docker의 locked wrapper로 재생성한다. 이 생성 JS와 ignored 산출물은 source commit에 섞지 않는다. `devel-merge-20261014/merge-provenance.json`에 기존/새 base, 양쪽 parent와 stash SHA를 보존했다.

통합본 TypeScript 검사와 Studio 전체 검사 **1,840 통과·실패 0·skip 2**는 완료했다. Native·Docker WASM 재빌드, 새 base를 고정한 Rust lint·회귀, 실제 브라우저 및 독립 Print 전쪽 확인은 진행 중이다. 이전 source의 완료 결과를 이 통합본의 결과로 대신하지 않는다. 증적은 ignored `output/pr-review/issue7639-perf-20261013/devel-merge-20261014/`에 둔다.


## 통합본의 출력 보존과 제출 예외 승인

통합 소스 `5eb992843`의 Native와 Docker WASM 빌드는 성공했다. root pkg/public의 WASM SHA-256은 `2615f003fe0c05bcfac7642dc9c8e19a4981a22eee34d0315649b24a0f7bd90a`, JS SHA-256은 `f377c6be818c64ffd20752e29271000ff5053ea8c7c984f65abb8fe74f17eddd`이며 실제 Chrome 응답도 같은 WASM이었다. 전체 161쪽 Native SVG는 기존 `22317614b` 출력과 바이트가 같고, 실제 Canvas 161쪽의 raw RGBA도 병합 전 `517b6a3b7`과 같다. `svg-parity.json`, `canvas-parity.json` 및 `browser/` 원문에 연결한다.

통합본 첫 표 안 캐럿 클릭은 71ms, 실제 휠 최대 RAF 간격은 93.6ms(100ms 초과 0회)였다. 셀 경계 page API는 기존 전체 API의 동일 쪽 필터와 같은 소유·순서·좌표를 반환했다. 실제 file input → 행 삽입 → Undo/Redo → 문서 교체도 통과했다. 첫 표시 helper는 14.724초였고 호스트 검증 컴파일과 겹쳐 실행되었으므로 이전의 한가한 환경 수치와 같은 조건의 반복 측정이라고 보고하지 않는다.

새 Native/fresh WASM 전쪽 Sweep은 모두 `svg=161, tree=161, pdf=162`로 종료했으며 전체 TSV는 완성되지 않았다. 메인터너는 이 결과를 확인한 뒤 2026-10-14에 **“성능 보정 범위의 시각 게이트 예외 수용”**을 명시했다. 자동 gate는 `re_review_required`인 채 보존하며, 성능 보정 제출에 한한 사용자 예외를 별도로 기록한다. 한컴 전쪽 조판 일치나 기존 쪽수 차이 해결은 주장하지 않는다. 허용치·baseline·기존 검사 기대값은 변경하지 않는다.

대표 1쪽을 추가 산출해 Native/fresh WASM review와 standalone overlay를 직접 열었다. 두 경로의 2px 실루엣 보조값은 각각 99.74962%, 엄격 픽셀 일치는 94.01578%였다. 표 행·열과 본문의 소속은 같고 획·얇은 괘선의 raster 차이가 보인다. 도구 라벨과 수치는 정상 판독된다. 대표 한 쪽의 수치를 전체 162쪽 통과로 확대하지 않는다. 원본 입력은 기존 tracked sample이며 SHA-256 `47de1ce82472050a4c5f33ccab2a3eac2c7e55d0e2a688febd085f0b4c8320ca`로 HEAD 바이트와 같다. 새 기준 Print PDF는 다른 tracked PDF와 해시가 달라 덮어쓰지 않고 [한컴 2020 Print](../working/assets/issue7639/hancom-2020-print.pdf)에 검증에 쓴 바이트 그대로 보존한다. Native/WASM 대표 PNG와 실제 Studio PNG도 같은 assets 디렉터리에 보존한다.

세 Clippy·workspace build·새 base의 manifest/unit-tier 검사와 focused 검사 12건은 통과했다. 전체 Rust와 Native Skia는 실행 중이다. 이 단계에는 GitHub push·PR 생성·이슈 종료를 수행하지 않았다.


## 통합본 필수 검증 완료와 제출 준비

통합본 전체 nextest는 **10,285 통과·실패 0·skip 45**였다. compile 포함 1,071.8초, 실행 476.7초다. Native Skia lib는 root 3,859건과 내부 crate 15·165·2건, 합계 **4,041 통과·ignored 12**였다. placeholder 2건과 direct PDF 4건도 통과했다. fmt·세 Clippy·workspace build·정책 검사·focused 12건·TypeScript·Studio 전체 1,840건, Native/Docker WASM 빌드와 실제 브라우저 검사 모두 완료했다. `devel-merge-20261014/validation.json` 및 각 로그로 재현한다. 코드·test 소스는 빌드 이후 변경이 없으며 `source-drift.json`은 빈 배열이다. 기준값이나 허용치를 바꾸지 않았다.

검증 중 원격은 `fea344f1fbed64737862e7f9f2c56287911d1256`로 전진했다(#7889·#7896). 두 Rust 파일의 바뀐 부분은 줄 주석 번역뿐이며, 주석을 제외한 줄 배열과 전체 줄수가 이전 base와 같다. 나머지는 문서 기록이다. 충돌 없는 현재-base merge-tree와 새 base의 manifest/unit-tier 검사를 추가 확인했다. 검증된 code head는 유지하고 기본 작업공간의 clean한 devel만 이 SHA로 fast-forward했다. `base-advance.json`, `latest-base-manifest.log`, `latest-base-unit-tier.log`에 연결한다. PR base는 현재 devel이며, GitHub merge tree에는 최신 주석·문서가 함께 보존된다.

제출 범위는 행·너비 탐색, 현재 페이지 hover 셀 조회, 동기 paint의 font 해석 비용 개선과 출력 보존이다. 161/162쪽 차이와 전체 TSV 미완성은 자동 gate 미충족으로 남기되, 메인터너가 승인한 성능 범위 예외로 제출한다. 독립 Print 조판 일치 완료를 주장하거나 기존 쪽수 검사 기대값을 바꾸지 않는다. 한국어 PR 본문 초안과 정확한 head로 고정할 Native/WASM review·overlay PNG, 검증에 쓴 원본·Print PDF를 준비했다. PR 등록 후 해당 code head의 CI와 self-review, 이후 merge 승인은 별도 단계다.
