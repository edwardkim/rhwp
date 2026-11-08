# 원 PR별 개별 검토 색인

> **2026-10-27 증적 대응 정정:** 원 PR에 검증용 HWP/HWPX가 없는 사례는 유사 문서로 대체했음을 표시한다. 아래 수치는 실제 측정 입력에 한정하며 배경 실물 문서의 직접 검증으로 표현하지 않는다. [현재 대응 원장](../pr8319_evidence_alignment_20261027/alignment.json)과 개별 review를 우선한다.

원 contributor head·메인터너 보정·실제 검증 코드와 원격 CI를 구분한다. 대표 쪽은 [주장·입력·편집 상태 색인](claim-evidence.json)의 명시적 선택을 따른다. 다른 문서의 높은 점수나 첫 페이지로 대신하지 않는다.

| 원 PR | 관련 전체 TSV 조건 | 주장에 맞는 대표 쪽 | 개별 리뷰 / 구현 기록 |
| --- | --- | --- | --- |
| [#7559](https://github.com/edwardkim/rhwp/pull/7559) | 059, 060, 061, 062, 063, 064, 065, 066, 067, 068, 069, 070, 071, 072, 076, 077, 078, 079 | 077 p1: 새 표를 HWPX로 저장·재열기한 표와 뒤 본문의 원점 | [review](../../archives/pr_7559_review.md) / [구현·검증](../../archives/pr_7559_review_impl.md) |
| [#7572](https://github.com/edwardkim/rhwp/pull/7572) | 176, 177, 178 | 178 p1: 본문 조합 뒤 첫 문단과 명시적 배치의 최종 출력; 배치 중 재조판 지연은 실제 API 검사로 구분 | [review](../../archives/pr_7572_review.md) / [구현·검증](../../archives/pr_7572_review_impl.md) |
| [#7573](https://github.com/edwardkim/rhwp/pull/7573) | 179, 180, 181, 182 | 181 p1: 시작 경계에서 오려 붙인 뒤 표 하나와 앞뒤 텍스트의 소유·순서 | [review](../../archives/pr_7573_review.md) / [구현·검증](../../archives/pr_7573_review_impl.md) |
| [#7588](https://github.com/edwardkim/rhwp/pull/7588) | 018, 019, 020, 021, 032, 033, 034, 035, 048, 049, 050, 051, 052, 053, 054, 055 | 021 p1: 30도 회전 묶음을 푼 뒤 자식과 누름틀의 유지 | [review](../../archives/pr_7588_review.md) / [구현·검증](../../archives/pr_7588_review_impl.md) |
| [#7589](https://github.com/edwardkim/rhwp/pull/7589) | 074, 075 | 074 p1: 여러 줄 입력 상자의 한컴 Print 표시와 false 대조군 | [review](../../archives/pr_7589_review.md) / [구현·검증](../../archives/pr_7589_review_impl.md) |
| [#7662](https://github.com/edwardkim/rhwp/pull/7662) | 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194 | 192 p1: 개체 바로 앞 문단 나눔 뒤 인라인 사각형과 뒤 텍스트의 다음 문단 소속 | [review](../../archives/pr_7662_review.md) / [구현·검증](../../archives/pr_7662_review_impl.md) |
| [#7694](https://github.com/edwardkim/rhwp/pull/7694) | 124, 125, 128, 129 | 125 p1: 차트를 논리 슬롯으로 옮긴 뒤 본문·차트·후속 문단의 순서 | [review](../../archives/pr_7694_review.md) / [구현·검증](../../archives/pr_7694_review_impl.md) |
| [#7819](https://github.com/edwardkim/rhwp/pull/7819) | 080, 081, 082, 083, 084, 085, 086, 087, 088, 089, 090, 091, 092, 093, 094, 095, 096, 097, 098, 099, 100, 101, 102, 103, 104, 105, 106 | 082 p1: 비 BMP 글자 뒤 미주가 있는 실제 출력과 논리 슬롯의 API 검사 | [review](../../archives/pr_7819_review.md) / [구현·검증](../../archives/pr_7819_review_impl.md) |
| [#7833](https://github.com/edwardkim/rhwp/pull/7833) | 133, 134, 135, 136, 137, 138, 139, 140, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169 | 134 p1: 셀 경계 오리기 실행 후 결과; 오리기 전 133은 대조군 | [review](../../archives/pr_7833_review.md) / [구현·검증](../../archives/pr_7833_review_impl.md) |
| [#7904](https://github.com/edwardkim/rhwp/pull/7904) | 210, 211 | 211 p1: 실제 WASM의 인라인 그림 뒤 IME 확정 결과; 확정·취소·Undo/Redo 상태 기록과 연결 | [review](../../archives/pr_7904_review.md) / [구현·검증](../../archives/pr_7904_review_impl.md) |
| [#8014](https://github.com/edwardkim/rhwp/pull/8014) | 195, 196 | 195 p1: NONE 선이 보이지 않는 Print 출력과 실선 196 대조군 | [review](../../archives/pr_8014_review.md) / [구현·검증](../../archives/pr_8014_review_impl.md) |
| [#8025](https://github.com/edwardkim/rhwp/pull/8025) | 113 | 113 p4: bold=false인 굵은 표시용 실제 HY헤드라인M의 Canvas 잉크 | [review](../../archives/pr_8025_review.md) / [구현·검증](../../archives/pr_8025_review_impl.md) |
| [#8027](https://github.com/edwardkim/rhwp/pull/8027) | 실제 파싱·조회 계약 함수 | 실제 계약 함수; 관계없는 PNG 없음 | [review](../../archives/pr_8027_review.md) / [구현·검증](../../archives/pr_8027_review_impl.md) |
| [#8029](https://github.com/edwardkim/rhwp/pull/8029) | 실제 파싱·조회 계약 함수 | 실제 계약 함수; 관계없는 PNG 없음 | [review](../../archives/pr_8029_review.md) / [구현·검증](../../archives/pr_8029_review_impl.md) |
| [#8030](https://github.com/edwardkim/rhwp/pull/8030) | 113 | 실제 계약 함수; 관계없는 PNG 없음 | [review](../../archives/pr_8030_review.md) / [구현·검증](../../archives/pr_8030_review_impl.md) |
| [#8031](https://github.com/edwardkim/rhwp/pull/8031) | 132, 145 | 145 p1: 공개 API로 표 셀 안에 정의한 쪽 번호의 실제 출력 | [review](../../archives/pr_8031_review.md) / [구현·검증](../../archives/pr_8031_review_impl.md) |
| [#8032](https://github.com/edwardkim/rhwp/pull/8032) | 141, 142, 146, 147, 148, 149, 150, 151, 152, 153 | 141 p1: 큰 인라인 그림이 남은 줄 폭에 들어가지 않아 그림 앞에서 줄바꿈; 작은 그림 142는 같은 줄 정상 대조군 | [review](../../archives/pr_8032_review.md) / [구현·검증](../../archives/pr_8032_review_impl.md) |
| [#8033](https://github.com/edwardkim/rhwp/pull/8033) | 015, 059, 060, 061, 062, 063, 064, 065, 066, 067, 068, 069, 070, 071, 072, 170, 171 | 170 p4: 중첩 표의 시작 조각·바깥여백과 뒤 본문; 같은 측정·배치 프레임 소비 | [review](../../archives/pr_8033_review.md) / [구현·검증](../../archives/pr_8033_review_impl.md) |
| [#8035](https://github.com/edwardkim/rhwp/pull/8035) | 212, 213, 214 | 212 p1: 그림을 지운 뒤 앞글뒤글의 굵게 경계; 다른 두 삭제 시나리오 TSV와 API 경계 대조 | [review](../../archives/pr_8035_review.md) / [구현·검증](../../archives/pr_8035_review_impl.md) |
| [#8036](https://github.com/edwardkim/rhwp/pull/8036) | 143, 144 | 144 p1: 큰 저장 vpos 뒤 실제 한 쪽 출력과 뒤 문단 보존; 143은 영 원점 대조군 | [review](../../archives/pr_8036_review.md) / [구현·검증](../../archives/pr_8036_review_impl.md) |
| [#8037](https://github.com/edwardkim/rhwp/pull/8037) | 059, 060, 061, 062, 063, 064, 065, 066, 067, 068, 069, 070, 071, 072 | 065 p1: 쪽 기준 그림을 가진 표 문단의 양수 offset과 바깥여백 | [review](../../archives/pr_8037_review.md) / [구현·검증](../../archives/pr_8037_review_impl.md) |
| [#8038](https://github.com/edwardkim/rhwp/pull/8038) | 056, 057, 058, 073 | 073 p1: 첫 줄 윗강조점 잉크와 본문 clip 상단 | [review](../../archives/pr_8038_review.md) / [구현·검증](../../archives/pr_8038_review_impl.md) |
| [#8039](https://github.com/edwardkim/rhwp/pull/8039) | 130, 131 | 130 p1: TAC 표 뒤 컨트롤 전용 저장 줄의 점유와 NEXT 본문; false 대조군 131 | [review](../../archives/pr_8039_review.md) / [구현·검증](../../archives/pr_8039_review_impl.md) |
| [#8041](https://github.com/edwardkim/rhwp/pull/8041) | 022, 023, 024, 025, 026, 027, 028, 029, 030, 031, 036, 037, 038, 039, 040, 041, 042, 043, 044, 045 | 026 p1: 실제 자식 틀을 사용하는 도형 겹침 순서의 편집 후 출력 | [review](../../archives/pr_8041_review.md) / [구현·검증](../../archives/pr_8041_review_impl.md) |
| [#8042](https://github.com/edwardkim/rhwp/pull/8042) | 018, 019, 020, 021, 032, 033, 034, 035, 048, 049, 050, 051, 052, 053, 054, 055 | 052 p1: 묶기·풀기 편집 뒤 누름틀과 자식 회전 상태 | [review](../../archives/pr_8042_review.md) / [구현·검증](../../archives/pr_8042_review_impl.md) |
| [#8043](https://github.com/edwardkim/rhwp/pull/8043) | 197 | 197 p1: 책갈피 링크 문단의 표시와 순서; 링크 대상 읽기·갱신은 별도 API 검사 | [review](../../archives/pr_8043_review.md) / [구현·검증](../../archives/pr_8043_review_impl.md) |
| [#8045](https://github.com/edwardkim/rhwp/pull/8045) | 112 | 112 p1: 한컴 저장본의 셀 안 어울림 중첩 표 옆 글 시작 위치 | [review](../../archives/pr_8045_review.md) / [구현·검증](../../archives/pr_8045_review_impl.md) |
| [#8049](https://github.com/edwardkim/rhwp/pull/8049) | 046, 047 | 046 p1: 캡션 자동 번호·접두부·뒤 문자의 표시 | [review](../../archives/pr_8049_review.md) / [구현·검증](../../archives/pr_8049_review_impl.md) |
| [#8050](https://github.com/edwardkim/rhwp/pull/8050) | 198 | 198 p1: NONE 셀 변을 표 외곽선으로 채우지 않는 실제 출력 | [review](../../archives/pr_8050_review.md) / [구현·검증](../../archives/pr_8050_review_impl.md) |
| [#8051](https://github.com/edwardkim/rhwp/pull/8051) | 199 | 199 p1: 첫 표 조각의 cellzone 색과 외곽선; 뒤 빈 쪽은 쪽수 대조이며 가시 내용 증거가 아님 | [review](../../archives/pr_8051_review.md) / [구현·검증](../../archives/pr_8051_review_impl.md) |
| [#8052](https://github.com/edwardkim/rhwp/pull/8052) | 200, 201 | 실제 계약 함수; 관계없는 PNG 없음 | [review](../../archives/pr_8052_review.md) / [구현·검증](../../archives/pr_8052_review_impl.md) |
| [#8053](https://github.com/edwardkim/rhwp/pull/8053) | 202 | 202 p1: 용지 기준 머리말 도형과 본문 기준 도형의 상대 원점 | [review](../../archives/pr_8053_review.md) / [구현·검증](../../archives/pr_8053_review_impl.md) |
| [#8054](https://github.com/edwardkim/rhwp/pull/8054) | 203 | 203 p1: offset 표 아래에서 같은 문단 본문이 재개하는 순서와 위치 | [review](../../archives/pr_8054_review.md) / [구현·검증](../../archives/pr_8054_review_impl.md) |
| [#8055](https://github.com/edwardkim/rhwp/pull/8055) | 015, 123, 170, 171 | 170 p8: 표 이어받기·종료 조각 뒤 첫 줄의 간격과 후속 본문 | [review](../../archives/pr_8055_review.md) / [구현·검증](../../archives/pr_8055_review_impl.md) |
| [#8056](https://github.com/edwardkim/rhwp/pull/8056) | 204, 205 | 205 p1: 줄 끝 공백이 있는 숫자의 실제 오른쪽 정렬; 204는 공백 없는 대조군 | [review](../../archives/pr_8056_review.md) / [구현·검증](../../archives/pr_8056_review_impl.md) |
| [#8057](https://github.com/edwardkim/rhwp/pull/8057) | 107, 108, 109 | 107 p1: U+2024를 가운뎃점으로 사용한 실제 본문과 A·B 간격 | [review](../../archives/pr_8057_review.md) / [구현·검증](../../archives/pr_8057_review_impl.md) |
| [#8058](https://github.com/edwardkim/rhwp/pull/8058) | 113 | 실제 계약 함수; 관계없는 PNG 없음 | [review](../../archives/pr_8058_review.md) / [구현·검증](../../archives/pr_8058_review_impl.md) |
| [#8061](https://github.com/edwardkim/rhwp/pull/8061) | 115, 116, 117, 118, 119, 120, 121, 122 | 120 p1: 쪽 기준 떠 있는 그림의 높은 z-order와 실제 셀 내용 쌓기 순서; 115는 정상 대조군 | [review](../../archives/pr_8061_review.md) / [구현·검증](../../archives/pr_8061_review_impl.md) |
| [#8062](https://github.com/edwardkim/rhwp/pull/8062) | 206 | 실제 계약 함수; 관계없는 PNG 없음 | [review](../../archives/pr_8062_review.md) / [구현·검증](../../archives/pr_8062_review_impl.md) |
| [#8065](https://github.com/edwardkim/rhwp/pull/8065) | 207 | 207 p1: 안 여백 아래 가 글줄이 셀 안에 포함되는 실제 행 높이 | [review](../../archives/pr_8065_review.md) / [구현·검증](../../archives/pr_8065_review_impl.md) |
| [#8066](https://github.com/edwardkim/rhwp/pull/8066) | 208 | 208 p1: 마지막 줄 간격만 넘치는 TAC RowBreak 행과 가 글줄 | [review](../../archives/pr_8066_review.md) / [구현·검증](../../archives/pr_8066_review_impl.md) |
| [#8090](https://github.com/edwardkim/rhwp/pull/8090) | 209 | 209 p2: 두 구역 중 앞 구역에 삽입·편집한 미주가 마지막 쪽에 표시되고 캐럿 소속이 일치 | [review](../../archives/pr_8090_review.md) / [구현·검증](../../archives/pr_8090_review_impl.md) |
| [#8093](https://github.com/edwardkim/rhwp/pull/8093) | 110, 111 | 110 p1: 가와 나 사이 TAC 양식 개체의 표시; 논리 한 칸·캐럿은 API 검사 | [review](../../archives/pr_8093_review.md) / [구현·검증](../../archives/pr_8093_review_impl.md) |
| [#8094](https://github.com/edwardkim/rhwp/pull/8094) | 046, 047 | 046 p1: 캡션 자동 번호와 뒤 글자의 표시; 캐럿·클릭 역변환은 API 검사 | [review](../../archives/pr_8094_review.md) / [구현·검증](../../archives/pr_8094_review_impl.md) |
| [#8096](https://github.com/edwardkim/rhwp/pull/8096) | 126, 127 | 126 p1: 저장 줄 없는 양식 전용 문단의 다섯 개체 표시 | [review](../../archives/pr_8096_review.md) / [구현·검증](../../archives/pr_8096_review_impl.md) |
