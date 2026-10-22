# Stage 50: 빈 앞 프레임 뒤 종료 행의 원본 컷 복구

Issue: #7445. parent HEAD는 `715c9d259126850da7de8d60126cd52d20743707`, 고정 base는 `2e087a5b4bc1515ad877e8020b30114639a59392`다. 개인정보 원본 범위만 계속 보정하며 다른 HWP 문서를 추가하지 않았다.

원 #5846 → PR #5904 → 통합 #5912/b9eb551과 후속 #7642/363ef22d2c7의 내용 소유·완료 억제 계약을 대조했다. 리베이스 전 5e23738에 있던 `stored_empty_prefix_row_closing_frame`과 측정/재귀 컷 호출이 통합 4eeea에서 빠진 것을 확인했다. 후처리는 첫 제목과 표만 5.873333px 내리고 뒤 문단은 기존 147.966667px에 남겨 첫 표 하단 153.84px와 겹쳤다. 뒤 문단을 clamp하는 대신 실제 source 컷을 복구했다.

빈 앞 문단·명시적 원점0 재시작·후속 단조 사다리·미편집 원본을 확인하고, 각 TopAndBottom 자식의 측정 높이와 저장 다음 줄 원점이 같은 상자를 닫는 경우에만 원본 종료 RowCut과 높이를 반환한다. mixed nested query가 같은 컷을 재귀 partial paint에 전달하며 `row_cut_height`도 같은 높이를 소비한다. 새 devel의 accepted row 장부는 유지하고 별도 물리 원장을 되살리지 않는다.

Native 첫 표 상단은 100.44px이며 독립 Print 가로선 중심 100.370687px와 대응한다. 첫 표 51.28px의 하단 151.72px와 다음 제목 시작 151.72px가 같아 겹침이 없다. 문서/페이지별 상수나 최종 bbox 보정은 추가하지 않았다.

기존 95개 집중 검사는94 PASS / 1 FAIL(test 18.418초)이다. #5846의 60쪽 관계 검사는 지나갔지만65·66쪽 설명 칸 소실과57·58쪽 물리 종료 차이가 남아 함수 전체는 FAIL이다. #4138 원본115/분할197 및 #7752 기능 보존은 PASS다. 기존 검사와 기대값을 바꾸지 않았다.

같은 Native 조건 전 66쪽 재출력은66/66쪽,60쪽 63.16676→98.32834%다. 바뀐 PNG는60쪽 하나이고 나머지65쪽은 byte 동일, 수정 전 대비0.1%p 이상 하락 0이다. 두 빈 source 문단의 빈 문자열 노드를 제외하면 소유별 비공백 본문 변화 0이다. review와 overlay를 직접 열어 제목→표→다음 제목 및 뒤 서명을 확인했으며 괘선 굵기와 미세 외곽 차이는 남는다.

전쪽 최저 46.20358%(59쪽), 미달 57·59·66 및 base 대비 41쪽 하락은 계속 보류다. fresh WASM·최신 전체 회귀·최종 게이트/actual36 완료로 보고하지 않는다. 첫 Python 3.14 캡처는 PDFium 부재로 중단됐고 보존 후 Python 3.12 venv에서 새로 전 66쪽 실행을 마쳤다.

[입력·producer SHA, 원인 소비 경로, 전쪽 TSV와 전후 review/overlay](../pr/assets/issue7445/completed-20261009/stage50-pii-source-closing-20261022/verification.md). 로그는 ignored output의 `repair-focused-v36/nextest.log`, `pii-closing-native-v36.log`에 보존한다. 지침·원격 변경은 없고 PR은 필수 검증 완료 후 새 사용자 승인을 받는다.
