# #6266 양식 개체(PushButton)가 배치를 잃고 제목 줄에 인라인으로 그려진다

문서: `samples/issue6266/seizure_list_form_button.hwp` (대법원 2955289, HWP3, 1쪽)
렌더: `export-png --compat 2024 --scale 1.5`

| 영역 | 전 | 후 |
| --- | --- | --- |
| 1쪽 제목 (원본 y 140~200px) | `before_title.png` | `after_title.png` |
| 쪽 하단 (원본 y 1010~1090px) | `before_bottom.png` | `after_bottom.png` |

전: 양식 개체가 제목 `압 류 목 록` 오른쪽에 인라인으로 붙어 제목이 왼쪽으로 밀렸고,
쪽 하단은 비어 있다. 후: 개체가 쪽 하단 가운데로 가고 제목이 본문 가운데로 복귀한다.

한글 2024 오라클(COM SaveAs PDF, producer=Hancom PDF 1.3.0.547) 대조:

| | rhwp 종전 | rhwp 수정 | 한글 |
| --- | --- | --- | --- |
| `- 581-13 -` y | 124.18pt | 792.3pt | 787.06pt |
| `- 581-13 -` x 중심 | 330.6pt | 297.64pt | 297.47pt |
| `압 류 목 록` x 중심 | 261.64pt | 297.64pt | 297.56pt |

참고 — PNG 에서 단추가 검은 상자로, 캡션 글씨가 안 보이는 것은 이 PR 과 무관한
Skia 경로의 종전 동작이다(같은 문서의 PDF 출력에는 `- 581-13 -` 이 정상 기록된다).
