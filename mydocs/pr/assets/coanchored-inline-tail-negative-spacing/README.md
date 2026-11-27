# 인라인 중첩 표 뒤 음수 줄 간격 검증

Windows에서 한컴오피스 2020 11.0.0.9136으로 공개 합성 HWP를 인쇄했다. 원본은 한컴 2020으로 저장한 공개 예제의 파일 헤더·문서 정보와 스타일을 유지한다. 인쇄 형식은 A4, 원본 방향, 전체 문서, 1-up이다. 서비스의 한컴 PDF 드라이버, Print method 0과 보호 글꼴 세션 등록을 응답에서 확인했다. [합성 입력](../../../../tests/fixtures/coanchored_inline_tail/negative-spacing.hwp) · [Print PDF](../../../../pdf/coanchored-inline-tail-negative-spacing/negative-spacing-2020.pdf) · [서버 상태](print-status.json) · [인쇄 영수증](print-receipt.json)

서버는 `input_preprocess: "none"`을 반환했다. 영수증은 이 값을 그대로 기록하고, RHWP 검증기에서 같은 뜻인 `preprocess: null`로도 표기한다. 원본과 PDF의 SHA-256을 대조했다.

| 경로 | 수정 전 → 수정 후 쪽수 | 수정 후 Print 실루엣 일치율 |
| --- | --- | --- |
| Native Skia | 3 → 2 | 1쪽 98.98%, 2쪽 100.00% |
| fresh WASM Canvas | 3 → 2 | 1쪽 98.97%, 2쪽 100.00% |

96 DPI, PDFium 기준 래스터, 색상 경계 보정 전 2px 실루엣으로 두 쪽을 각각 검사했다. 수정 후 제목 표와 본문 표는 같은 1쪽에 놓이고 `Next page` 표는 2쪽에 놓인다. 중첩 표 직후 `Tail line`은 Print보다 약 6 CSS px 위에 있다. 수정 전 2쪽과 수정 후 1쪽에서 중첩 표와 글줄의 렌더링 위치는 동일했다.

- Native의 두 쪽과 대표 1쪽. [TSV](windows-2020/native/silhouette.tsv) · [비교](windows-2020/native/review-page-1.png) · [오버레이](windows-2020/native/overlay-page-1.png)
- fresh WASM의 두 쪽과 대표 1쪽. [TSV](windows-2020/wasm/silhouette.tsv) · [비교](windows-2020/wasm/review-page-1.png) · [오버레이](windows-2020/wasm/overlay-page-1.png)
