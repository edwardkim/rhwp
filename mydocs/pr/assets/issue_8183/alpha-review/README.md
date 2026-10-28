# 반투명 이미지 시각 확인 자료

[메인테이너 확인 안내](../../../../working/task_m100_8183_alpha_visual_review.md)에서 시작한다.

- [오프라인 HTML 뷰어](viewer.html): 100~400% 확대, 한컴 재저장 Print 추가 표시, 원본 PNG 다운로드.
- [전체 비교판](overview.png).
- [일반 alpha 비교판](jpx-alpha-comparison.png).
- [preblend 복원 비교판](jpx-premult-comparison.png).

PNG 원본은 기존 작업 캡처를 바이트 그대로 복사했고, 내장 PNG는 검증한 HWPX에서 그대로 추출했다.
`viewer.html`은 같은 이미지 바이트를 포함해 단독으로 열 수 있다. 비교판 PNG는 이 HTML을
Chrome 154·viewport 2020px·deviceScaleFactor 1·확대 200%에서 캡처했다.
캡처의 실제 214×107 / 211×106 크기 차이를 유지하며 자르거나 색상·alpha를 보정하지 않았다.
제품 source는 `cfc5be8fe`다. 새 Native/WASM 렌더링이나 점수 재측정이 아니다.
파일별 출처·원본 해시는 [provenance.json](provenance.json)에 있다.

독립 기준과 RHWP가 연속 alpha를 표시하는 데 비해 한컴 Print는 1비트 mask를 만든다.
Print 미달 수치는 유지한다. 메인테이너의 직접 수용 결정은 아래에 기록했다.

## 2026-10-24 메인테이너 결정

2026-10-24 메인테이너가 두 4×2 반투명 사례의 RHWP 처리를 정답으로 판정하고 PR 진행을 결정했다.
Print의 미달 수치·TSV·원본은 보존하고 해당 두 Print를 alpha 정답지로 사용하지 않는다.
수치 임계값·제품 alpha·회귀 기대값은 바꾸지 않는다.
