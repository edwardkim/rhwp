# 공개 합성 시각 증거

이 디렉터리의 HWP 파일은 회사 문서의 사진·본문·원시 스트림을 포함하지 않습니다. 회사 원본에서 배치에 필요한 숫자 속성만 읽어 한컴 빈 문서에 넣고, 그림은 직접 만든 색상 번호 도형으로 대체했습니다.

- `grid-soft-hancom.hwp`: 그림 효과 뒤 크기 필드 파서 재현
- `mixed.hwp`, `top.hwp`, `bottom.hwp`, `short-inline.hwp`, `one-float.hwp`, `positive.hwp`, `following.hwp`, `stale-height.hwp`: 그림 혼합 셀의 겹침·잘림·뒤 문장 흐름 재현
- `*-web-review.png`: 수정 전 Native SVG, 수정 후 Native 또는 fresh WASM, 실제 한컴 WebHwp 캔버스를 한 화면에 둔 비교 자료

기준 엔진은 공개 한컴 WebHwp 체험 서비스 `10.80.0.2862`이며, 이 자료는 화면 배치 확인용입니다. 회사 WebHwp 배포판과 한컴 Print PDF의 최종 수용 증거로 사용하지 않습니다. 회사 원본 파일과 렌더링 결과는 이 저장소에 포함하지 않았습니다.
