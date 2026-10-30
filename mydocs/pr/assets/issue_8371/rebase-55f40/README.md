# 2026-10-31 리베이스 후 Native 기능 보존

기준 `upstream/devel`은 `55f40ecd99485a1c9b1f7cdc709cb6efcb9e9b29`, 후보 코드는
`11def71519`다. #8394가 반영한 실물 입력 6개와 독립 Print PDF의 SHA-256은
[`inputs.json`](../../seo_rii_20261030/inputs.json)의 기록과 일치한다. 편람은 이번 PR의 보정
대상이 아니라 새 기준의 기능 보존 확인에만 사용했다.

동일한 원본·PDF·명시 글꼴·96 DPI 조건에서 새 기준과 후보의 Native 출력을 대조했다.
[전쪽 정밀 대조](seo-native-exact.tsv)는 388쪽이며 raw 하락이 0쪽이고, 388개 출력 PNG는
기준과 후보가 바이트 단위로 같다. 후보 최저 raw는 98.928024502297%다. 본문 목록·글꼴 대조군은
compat 2022, 편람은 compat 2024를 사용했다. 선택 글꼴의 파일별 해시는
[로컬 글꼴 목록](seo-rii-local-fonts.json)에 기록했으며 글꼴 프로그램 자체는 제출하지 않는다.
원 통합 PR과 일부 글꼴 파일의 해시가 다르므로 그 PR의 숫자와 직접 비교하지 않고,
현재 환경에서 새 기준과 후보를 각각 출력했다.

기준 Native 실행 파일 SHA-256은 `3d35006a37ab9e819c3eba55bd37e36976c0efd9a1bb807e8fdc8f6dde6e654c`,
후보는 `cb069b5827cd2a4507417952f4a8b4ad84462e29dba45f780fa040feb9efe9f7`이다.
WASM 전쪽 비교와 Native 캐시·타일·Direct PDF 집중 검사는 별도로 완료해 기록한다.

## Direct PDF의 연결선 보존

#8394에서 직접 PDF 출력이 가능해진 원본 63쪽과 289쪽을 각각 새 기준과 후보에서 출력했다.
`export-pdf --backend direct --profile print --compat 2024`에서 0부터 시작하는 쪽 번호
62와 288을 사용했고, 출력 PDF는 각각 한 쪽이다. 같은 원본 쪽의 독립 한컴 Print와 96 DPI로
대조한 결과 두 쪽의 raw 미대응 픽셀은 모두 0이다. 새 기준과 후보의 Direct PDF 래스터도
각각 바이트 단위로 같다. 출력된 선·화살촉과 상자 배치를 직접 확인했으며 색상·외곽선에는
미세한 잔차가 있다. 이 확인은 편람 전체의 Direct PDF 지원을 뜻하지 않는다.

| 원본 쪽 | raw 미대응 / 합집합 | review | overlay |
| --- | ---: | --- | --- |
| 63 | 0 / 99,670 | [비교](direct-pdf-p063-review.png) | [겹침](direct-pdf-p063-overlay.png) |
| 289 | 0 / 203,464 | [비교](direct-pdf-p289-review.png) | [겹침](direct-pdf-p289-overlay.png) |
