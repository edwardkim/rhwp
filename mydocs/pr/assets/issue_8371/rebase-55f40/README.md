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
