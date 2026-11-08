# Print 종이 표본 소비 중간 검증

Issue: #7445. 부모 HEAD `3a09d0df4b01297d101abc6729939f6fcde29220`, base `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`이다. `producer.json`의 5개 source hash를 확인했다. PR 전체 수용 증거가 아니다.

- TypeScript exit 0, Studio 2,409 PASS / 0 FAIL / 2 skip이다.
- 실제 제품 화면 62건과 동일 인스턴스 확대·DPR 변경/복원 20건을 실행했다. backend·DOM 준비·원상 복원 검사를 유지했다.
- 독립 Print 31건은 90% 미만 0건, 앞선 동일 소비 조건 대비 2%p 이상 하락 0건이다.
- 확대 20건 중 저장 줄 결측 대조군의 zoom 1.25 / DPR 1.5는 86.78369%로 미완료다. 구조 검사 성공을 정량 통과로 바꾸지 않는다.
- `results.json` 및 두 비교 결과의 원본 경로·hash를 연결한다. 원시 로그는 ignored output에만 있다.

| 대조 | review | overlay |
| --- | --- | --- |
| 로드맵 153쪽 DPR 1.5: 96.10150% | [review](roadmap-fractional-dpr-review.png) | [overlay](roadmap-fractional-dpr-overlay.png) |
| 저장 줄 결측 1쪽 확대: 86.78369%, 미완료 | [review](missing-lineseg-zoomed-review.png) | [overlay](missing-lineseg-zoomed-overlay.png) |

직접 판독에서 로드맵 격자·점·제목·출처를 보존했고 회색 선의 농도 차이가 남는다. 저장 줄 결측 대조군은 본문·표 내용이 보존됐으나 확대 가로선의 수 픽셀 위치 차이가 남아 있다. 허용치나 마스크를 바꾸지 않았다.
