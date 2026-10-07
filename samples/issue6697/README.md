# samples/issue6697 - 칸 안 문단 기준 중첩 표 vertOffset 회귀 입력

이 폴더는 [#6697](https://github.com/edwardkim/rhwp/issues/6697)의 정식 시각 검토
fixture다. `80550` HWPX 원본의 30쪽에는 칸 안 문단 기준 `TopAndBottom` 중첩 표가 있고,
저장 `vertOffset=3062HU`를 셀 경로도 적용해야 한다.

- 원본 문서 식별자: `80550`, 농업기계화 촉진법 시행규칙 일부개정령(안)
- 시각 계약: 중첩 표 상단은 저장 offset 3062HU(96dpi에서 약 40.8px)만큼 호스트 문단 아래에 놓인다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 한컴 기준 PDF와 통합 head 시각 증적은 PR 검토 자산에 기록한다.


## 전체 피델리티 후속 항목

2026-09-29 PR #7382 검토에서 이 원문의 정상31쪽/현재32쪽과 말미 중첩 표의 내용 소속 차이를 확인했습니다.
[후속 #7445](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5889107858)에 독립 PDF와 시각 증거를 연결하고,
해당 문서의 실제 실패 캡션 함수1개만 정식 회귀에서 이관했습니다. 원문과 manifest는 보존하며 일반 offset·음수 보호·가운데 정렬의 기존3계약은 유지합니다.
전체 피델리티와31쪽·캡션/뒤 내용 소속이 복원되고 시각 기준을 충족하면 실물 회귀를 회복합니다.
