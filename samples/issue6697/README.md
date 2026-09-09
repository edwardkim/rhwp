# samples/issue6697 - 칸 안 문단 기준 중첩 표 vertOffset 회귀 입력

이 폴더는 [#6697](https://github.com/edwardkim/rhwp/issues/6697)의 정식 시각 검토
fixture다. `80550` HWPX 원본의 30쪽에는 칸 안 문단 기준 `TopAndBottom` 중첩 표가 있고,
저장 `vertOffset=3062HU`를 셀 경로도 적용해야 한다.

- 원본 문서 식별자: `80550`, 농업기계화 촉진법 시행규칙 일부개정령(안)
- 시각 계약: 중첩 표 상단은 저장 offset 3062HU(96dpi에서 약 40.8px)만큼 호스트 문단 아래에 놓인다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 한컴 기준 PDF와 통합 head 시각 증적은 PR 검토 자산에 기록한다.
