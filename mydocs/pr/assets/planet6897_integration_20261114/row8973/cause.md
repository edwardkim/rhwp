# #8973 보정이 필요한 근거

- 원 #8976의 2·4쪽 주장은 현재 Native 99.99%·100.00%이며 집중 검사도 통과했다. 추가 개선 대상이 아니다.
- 같은 원본의 10쪽은 고정 devel과 통합 후보 모두 raw 80.98088%, fresh WASM 후보 80.99875%이다. 기존 #8973과 같은 범위이며 전쪽 90% 제출 조건의 잔여 항목이다.
- 원본 표 pi=54, 행23 열2: 선언 39792HU = 530.56px, 실제 행 544.0px. 내부 p4는 저장 줄 없는 빈 문단의 비-TAC TopAndBottom 표이며 PARA/TOP, flowWithText=true, 세로 오프셋 -5622HU이다.
- 원본의 해당 오프셋 하나만 0으로 만든 진단 사본에서 행은 530.6px이다. 글줄·중첩 표 82개는 크기와 상대 배치가 같으며 가운데 정렬 이동만 생겼다. 원본·기준 PDF는 변경하지 않았다.
- 생산/소비: `reflow_block_table_host_occupied_height` → HeightMeasurer의 text_height와 상세 CellUnit 원장 → layout/table_layout의 같은 host 점유 → 행 높이·가운데 정렬. 기존 측정은 음수 오프셋을 공통 합집합에서 제외해 host 줄과 중첩 표를 가산하지만 실제 셀 배치의 fallback은 `nested_h.max(line_based_h)`로 합집합을 소비한다.
- 보정 후보: 공통 helper가 같은 흐름으로 소비되는 비양수 오프셋을 받도록 한다. 양수 오프셋·보이는 글자·필드/TAC·실제 저장 줄의 기존 제외 조건은 유지한다. 문서 ID·선언 높이·PDF 수치에 맞춘 clamp는 추가하지 않는다.
- 보정 후 Native/fresh WASM은 전11쪽 최저 각각95.64%, 10쪽 각각100.00%다. 나머지10쪽은 각 backend의 보정 전 PNG와 SHA-256이 같다. [전쪽 변경 감사](page-change-audit.json)를 따른다.
- 최소 글꼴 환경에서 새 관계 검사는 실제 보정 전1FAIL(행544.0/530.6), 보정 후1PASS다. ZIP API 반환형 컴파일 오류는 이 결함 재현 결과에서 제외했다. [실행 기록](relation-test.json)을 따른다.
- 원본의 비양수 offset host와 zero-offset 대조군의 행·뒤 행 상자가 같고, 본문 순서·내용·포함 관계를 보존한다. 절대 PDF 좌표를 기대값으로 복사하지 않았다.
- Native [review](native/review-010.png)·[overlay](native/overlay-010.png), fresh WASM [review](wasm/review-010.png)·[overlay](wasm/overlay-010.png), [독립 Print PNG](pdf-010.png)를 같은 쪽으로 보존했다. 전체 회귀·정상 대조군·보호 문서 검증의 최종 판정은 통합 검토 기록에 별도로 남긴다.
