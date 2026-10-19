# #7642 부분 표의 자식 종료 판정과 호스트 뒤 간격

Issue: #7642. 원본 법령과 독립 한컴 Print 74쪽을 같은 PDFium 정책으로
대조한다. 수정 전 source는 57d4adf2a0이다.

## 실행 근거와 소비 경로

18쪽의 원본 193문단 안 1×1 래퍼에는 NO_LS 순수 TAC 호스트가 있다.
측정과 CellUnit은 13pt·130%의 줄 뒤 간격 5.2px를 계산한다. 원본 문단
뒤 간격은 4px다. 이 값은 원인 확인용 관측이며 구현 조건으로 고정하지 않는다.
GDB로 같은 바이너리의 실제 render-tree 출력을 추적했을 때 측정·CellUnit은
Some(5.2)를 반환했지만 부분 배치는 해당 호스트의 helper를 호출하지 않았다.
57d4adf2a0의 일반 셀 보정만으로 이 부분 표 경로가 바뀌었다고 판단하지 않는다.

생산은 cell_units의 자식 행·컷, 소비는 nested_table_split_from_cut_units와
nested_cut_rows, 최종 배치는 table_partial의 nested_table_continues와
호스트 종료 간격이다. scalar 행 컷의 visible_height는 행 본체인데,
calc_nested_table_height의 바깥여백 포함 높이보다 작다는 이유로 마지막 행도
이어받기로 오인한다. recursive cursor의 존재도 종료 여부와 같지 않다.

## 보정과 경계

명시적 행·재귀 컷은 끝 행과 terminal로 종료를 판정한다. scalar 행 컷은
마지막 자식 행을 포함하면 terminal이다. 가용 높이로 임시 분할한 경로의
높이 비교는 유지한다. 종료한 NO_LS TAC 호스트는 다음 내용이 같은 조각에
있을 때 줄 뒤 간격·문단 뒤 간격을 계상한다. recursive/mixed 컷의 flow_height가
이미 소유한 간격은 더하지 않는다. 중간 조각과 셀 마지막 문단에는 추가하지 않는다.

Native/fresh WASM에서 18·19쪽, 25~27·46~48·63~64쪽과 기존 분할 TAC 대조군을
확인한다. 전쪽 90%·쪽수·직접 판독과 기준 devel 대비 하락 검사를 유지한다.
신규 rendering 회귀와 baseline 갱신은 선행 시각 조건 충족 전 보류한다.

## 성장한 자식 행의 종료 간격 공유

298211faf7의 Native18쪽은95.14465%로 개선됐지만19쪽은77.69402%다.
19쪽 첫 표는 PDF와 맞고 둘째 표부터5.2px 위로, 뒤 본문은 두 표의
간격 누락이 누적돼 위로 이동한다. 193문단 래퍼의15·16문단은 같은
NO_LS TAC 호스트지만 자식의 내용 성장 때문에 재귀 유닛 컷을 사용한다.

일반 자식 행 유닛은 기존 helper의5.2px를 마지막 행에 더하지만,
성장한 자식 행의 driver 유닛 분기는 terminal_band만 더하고 helper를
소비하지 않았다. 일반/성장 경로가 끝난 공통 위치에서 마지막 호스트
유닛에 한 번만 더하도록 옮긴다. 이 간격은 부모 흐름 높이이며 자식의
가시 viewport가 아니므로 기존 nested_host_closing_line_height에도
기록한다. nested_table_split_from_cut_units가 flow와 visible을 구분해
소비한다. 일반 scalar 컷의 배치 보정은 유지한다.

43쪽 물리 최소 수정과 이번 종료 간격 수정을 다음 한 빌드에서 검증한다.
아직 시작하지 않은 물리 최소 전용 대기 작업만 확인 후 교체하고,
298211faf7의 진행 중인 WASM 빌드는 유지한다.
