# 이슈 #7642 Stage 52 — 미주 안의 흐름 표와 행 이어받기

Issue: #7642

## 독립 근거와 원본 계약

현재 source는 `1af0ae1790578ad04a0530cbb1d398e58255ded2`이며 Stage51의
TAC 문단 점유 보정 검증은 진행 중이다. 이번 분석은 별도 Table 항목으로
배치하는 non-TAC 표의 경로를 확인한다. 같은 범위의 최종 판정을 대신하지 않는다.

11월 구분선 없음 HWPX의 원본 본문 문단 162, 미주 19 내부 문단 3은
non-TAC·TopAndBottom·flow_with_text 표다. RowBreak이고 3행·6열이며
rowspan은 없다. 표 선언 높이는 1182 HWPUNIT, host 저장 줄 높이는 900이다.
이 값 하나로 표 전체의 물리 높이를 정할 수 없다.

독립 한컴 2024 Print 13쪽에는 앞 단 하단에 첫 두 행, 다음 단 상단에
마지막 행이 있다. Native는 다음 단에 세 행 전체를 배치한다. 이전 source의
13쪽 수치는 63.65353%다. 출력 위치는 ignored
`stage52-print-split-table-rows.json`의 PDF 해시와 괘선 관측으로 고정했다.
관측한 px는 진단용이며 코드·회귀 기대값으로 쓰지 않는다.

## 실제 호출 경로의 차이

미주 경로는 `paragraph.rs → query_rewind_tail →
no_separator_tail_table_starts_next_column → 단 이월 → emit_endnote_split →
Table`이다. 본문 흐름 표는 `format_table → typeset_block_table →
prepare_block_table_entry → scan/continuation → PartialTable`을 사용한다.
미주는 이 행 분할 경로를 호출하지 않는다. body의 측정 원장도 가상 미주
문단의 표를 포함하지 않아, 단순히 본문 함수를 호출하면 측정 없음에 따른
통째 강제 배치로 다시 빠진다.

## 연결할 공통 결과

- 원본 소유를 유지한 가상 미주 표를 같은 profile·styles·normalization으로
  한 번 측정하고, 페이지네이션과 실제 배치에 그 결과를 전달한다.
- 현재 단의 실제 흐름 원점과 표의 행 측정을 일반 표 포맷·분할 경로에 넘긴다.
  저장 줄 높이나 legacy 누계로 행 예산을 대신하지 않는다.
- 조각의 시작/끝 행·컷과 소유 유닛, 요구 높이, 누적 예약, 예산 실패 시
  이월, 최종 PartialTable 배치를 실제 경로에서 대조한다.
- non-TAC 흐름 표와 배경·전경·Square 어울림 개체를 구별한다. 별도 float
  배치 계약을 강제로 block 표로 바꾸지 않는다.
- 반복 머리행은 원본 header cell 속성을 확인한다. repeat_header 비트만으로
  모든 조각에 첫 행을 추가하지 않는다.

정상 본문 표의 분할 알고리즘을 복제하지 않고 기존 경로와 결과를 사용한다.
새로운 문서 ID·고정 높이·행수별 예외를 넣지 않는다. 현재는 분석 단계이며
아직 이 경로를 구현·실행했다고 보고하지 않는다.

## 검증과 보존

11월 HWP/HWPX 변형의 최초 차이 페이지, 앞뒤 단, 마지막 조각의 뒤 문단을
Native/fresh WASM과 Print로 직접 비교한다. 행·유닛 누락/중복, 표 외곽과
다음 내용, 불필요한 빈 페이지도 확인한다. 행 분할에 영향이 있어 최종
쪽수·시각 판정에는 전체 페이지 TSV가 필요하다.

미달이나 측정 불가이면 회귀 기대값·baseline은 변경하지 않는다. 모든 관련
페이지 90% 이상과 수식·간격 확대 판독 뒤 기존 px 검사를 소유·행/유닛
보존·앞뒤 순서·포함·겹침 관계로 전환한다.


## 원본 표의 측정 경계 확인

2024년 11월 구분선 없음 HWP와 HWPX의 원본 본문 문단 162, 미주 19
내부 문단 3을 source `b616c877624d1740b79a75b40b2a9bfcadacf6fc`의
HeightMeasurer로 독립 진단했다. 양쪽 모두 총 높이 47.28, 행 높이
15.76 × 3, 누계 [0, 15.76, 31.52, 47.28]이다. repeat_header는 true지만
has_header_cells는 false다. 첫 행을 반복 머리행으로 복제할 근거가 없다.
표 개체 선언 높이 1182 HWPUNIT은 한 행과 같고 세 행 전체 높이가 아니다.
독립 Print의 첫 두 행/마지막 한 행 분할과 원본 셀 높이가 대응한다.

이 진단은 원본 표의 HeightMeasurer 결과이며 일반 표 fit·분할·renderer
normalization의 실행을 대신하지 않는다. 라이브러리 source, 진단 소스 및
실행 파일 해시와 두 원본 해시는 ignored
`stage52-owned-table-measure-probe.json`에 기록했다.

실제 전달 경계는 StateView의 미주 측정 원장 → PaginationResult →
DocumentCore 렌더 캐시의 section별 measured_tables → 최종 Table/PartialTable
배치다. 현재 캐시는 본문 MeasuredSection.tables만 받으므로 가상 미주
표를 조판 시 측정하기만 해서는 paint가 같은 결과를 받지 못한다.
전체·부분·pending 섹션 페이지네이션 각각의 캐시 저장 위치도 함께 수정해야 한다.

분할 함수 호출에는 본문과 현재 미주 문단을 같은 전역 인덱스로 연결한
읽기 문맥이 필요하다. 다음 원본 문단·표의 속성 조회도 이 문맥을 사용한다.
기존 본문 분할은 미주 소유를 자동으로 게시하지 않으므로 단 전환 뒤에도
미주 조각 소속과 후속 간격 계약이 보존되는 소비 위치를 추가로 확인한다.
현재 원장 연결이나 분할 방출 구현은 아직 시작하지 않았다.


## 공통 조판과 소유 원장 연결

source `c0d6903a8`의 수식 꼬리 줄 보정과 함께 확인할 구현을 추가했다.
가시 호스트가 없는 문단에 문단 기준 TopAndBottom·flow_with_text 표만
있으면 `flow_table.rs`에서 원본 표를 같은 profile·normalization으로 측정한다.
각 표의 전역 문단/컨트롤 인덱스를 원장에 보존하며, 본문과 같은
`format_table → typeset_block_table → prepare/scan/continuation`을 호출한다.
가시 글줄·배경/전경·Square 개체와 명시적 단 경계는 기존 경로를 유지한다.
문서 번호·행 수·진단 높이를 분기에 사용하지 않는다.

진입 원점은 기존 항목의 공통 단 배치가 반환한 실제 흐름 끝이다.
본문의 줄 높이 추정이나 개체 선언 높이로 예산을 대신하지 않는다.
다음 문단 조회에는 본문·등록한 미주·아직 방출하지 않은 원본 미주 문단을
같은 전역 인덱스로 제공한다. 명시적인 단 조각·내용 컷의 생산과 예약,
최종 종료는 기존 본문 분할 경로를 사용한다.

StateView는 가상 미주 문단의 시작과 표 측정을 소유한다. `append_item`은
등록 범위 안의 Table/PartialTable 등 항목을 미주로 표시해 단 전환 뒤에도
소속을 보존한다. 측정 원장은 `into_result → PaginationResult → pending/full/
incremental 섹션 렌더 캐시 → LayoutEngine`으로 전달한다. 미주 fit 조회도
caller에 없는 소유 표 측정과 같은 normalization을 보충한다. 따라서 표를
조판할 때만 측정하고 최종 paint에서 다시 선언 높이를 선택하지 않는다.

새 코드의 빌드와 실제 PartialTable 방출은 아직 검증 전이다. 앞 두 행과
다음 마지막 행, 반복 머리행 부재, 후속 본문 보존 및 모든 대조군을 새
Native/fresh WASM에서 확인해야 한다. 이전 source99의 진행 중인 전쪽
Native는 보존용 전후 대조군이며 새 source의 결과로 보고하지 않는다.

## source 334246be3의 실행 결과

Native 빌드는 411.78초에 exit 0, fmt check와 Native Clippy도 각각
19.81초와 70.35초에 exit 0이었다. frozen binary SHA-256은
`831b9ad02348d53c1f8c5ca5bceea038332f4406a01d99b259d213b0d6625e9d`다.
빌드 전후 source 해시와 실제 검증 source를 ignored provenance에 보존했다.

11월 구분선 없음 HWPX 13쪽의 실제 pagination은 같은 원본 본문 162,
미주 내부 3, 컨트롤 0을 소유하는 PartialTable 두 개를 방출했다.
왼쪽 단은 행 [0, 2), 오른쪽 단은 이어받기 행 [2, 3)이다. 행 범위가
빠지거나 겹치지 않고 반복 머리행을 추가하지 않는다. 전체 쪽수는 Print와
같은 22쪽이다. 13쪽은 63.65353%에서 91.47413%로 개선됐으며 12쪽은
98.36368%다. 표 뒤 본문·그래프의 미세 간격은 추가 판독 대상이다.

11월 구분선 위 9·미주 사이 8·구분선 아래 7 HWPX는 13쪽 69.57823%다.
표 분할과 다른 원인으로 그래프 앞의 (iii) 제목이 다음 단으로 밀린다.
원본 keepWithNext=0과 Print의 단 하단 제목을 확인했고 Stage55에서
강제 묶음 판정을 분석한다. 전체 Native/fresh WASM 통과는 아직 미검증이다.
