# PDF 곡선·마스크·회전 대조군

`curve-mask-controls.pdf`는 PDF 문법으로 직접 작성한 합성 입력이다. 한컴에서
저장한 문서나 한컴 Print 결과가 아니다. 독립 PDF 렌더러의 출력과 가져온 문서의
Native/fresh WASM 출력을 비교한다. 원본 사용자의 PDF는 이 폴더에 포함하지 않는다.

`generate-controls.py`는 pikepdf와 Pillow로 입력을 재생성한다. 날짜·문서 ID 등의
컨테이너 바이트는 재생성 시 달라질 수 있으며 검증은 내용·배치 계약을 대상으로 한다.

| 쪽 | 독립 입력 계약 |
| --- | --- |
| 1 | 4개의 3차 Bézier로 만든 원형 clip |
| 2–3 | even-odd와 반대 winding의 동일한 사각 구멍 |
| 4 | v/y 곡선, q/Q 및 clip 밖의 별도 이미지 |
| 5–7 | 같은 alpha 표본의 Flate 기본값·PNG predictor 15·Gray JPEG SMask |
| 8 | 서로 다른 X/Y 배율을 갖는 90도 회전 텍스트와 정상 텍스트 |
| 9 | 음수 폭·높이를 갖는 복합 사각형 path |
| 10 | BBox와 아핀 변환을 갖는 Form 내부 곡선 clip |
| 11 | 아핀 CTM의 4색 이미지 |
| 12 | d1 메트릭만 갖는 Type3의 비가시 문자열과 실제 보이는 글자·벡터 |
| 13 | PDF RGB 회색·다색 표본의 저장·출력 색 보존 |

시각 측정과 보정 전후 실행 근거는
[작업 기록](../../mydocs/working/task_m100_8413_implementation.md)에 연결한다.
