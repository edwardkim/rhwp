# #7642 실패 회귀 원본의 Native 대조

생산 source: `f4bd0a760e930cc0bc9a257fb0a9b2f1c92a0513`.
Native 바이너리 SHA-256: `e02987f4376305b4ba86f5e3f78981cf809cf6e0b05e85963cd6f03de2b4e953`.
인쇄 프로필, 전체 글꼴 포함, 96 DPI, 기본 2px 관용 조건으로 새로 출력했다.

| 입력 / 독립 한컴 Print | 페이지 / 세대 | TSV 관용값 / 이진화 원값 | 판독 |
| --- | --- | --- | --- |
| `samples/issue6172/2599643_port_call_form.hwp` / `pdf/issue7642-failed-inputs-followup-20261011/2599643_port_call_form-2020.pdf` | 1 / 2022 | 95.19569% / 95.19569% | 신청 번호 상자 크기·글리프 차이 미해결 |
| `samples/hwp3-sample16-hwp5.hwpx` / `pdf/hwp3-sample16-hwp5-hwpx-2024.pdf` | 18 / 2024 | 98.83576% / 76.72151% | 그림 색상·세부 표현 차이 미해결. 색상 경계 보정 62,720픽셀 |

원본/Print의 전체 쪽수는 각각 1/1쪽, 64/64쪽이다. sample16은 선택 18쪽만 이번에
대조했으며 전체 쪽 검증·fresh WASM·출력 보정 완료를 주장하지 않는다. 높은 관용값으로
실제 색상·표현 결함을 면제하지 않는다. 파랑=RHWP만, 빨강=PDF만, 주황=양쪽 차이,
회색=일치다. 각 review는 RHWP / Print / overlay 순이다.

- [항만 서식 review](port-call-p001-native-review.png), [overlay](port-call-p001-native-overlay.png)
- [sample16 review](sample16-p018-native-review.png), [overlay](sample16-p018-native-overlay.png)

명령·입력/PDF 해시·run manifest·TSV는 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage144-failed-native-preflight/receipt.json`에 있다.
기록과 실패 분류는 [Stage 144](../../../working/task_m100_7642_stage144.md)에 연결한다.
