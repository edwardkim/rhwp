# 2026-10-31 리베이스 후 Native 대조

기준 `upstream/devel`은 `55f40ecd99485a1c9b1f7cdc709cb6efcb9e9b29`, 후보 제품 코드는
`11def71519`다. 공유 target에 남은 base 라이브러리가 후보에 잘못 연결된 첫 실행은 폐기했다.
후보 라이브러리와 실행 파일을 다시 빌드한 뒤 sample10 1쪽이 이전 정상 후보와 바이트 단위로
같은지 확인하고 아래 비교를 다시 실행했다.

sample10은 원본과 사용자 제공 한컴 2020 Print PDF의 763쪽을 대조했다. Native 최저 raw는
99.411348592795%이며 90% 미만과 이전 정상 후보 대비 하락은 0쪽이다. PNG도 763쪽 모두
이전 후보와 같다. [전쪽 정밀 대조](sample10-native-exact.tsv)와 상위 `sample10-native.tsv`는
각각 이전 후보 대비 변화와 Print 대비 측정이다.

#8394의 입력 6개와 Print PDF 해시는 [원 통합 PR의 입력 목록](../../seo_rii_20261030/inputs.json)과
같다. 새 기준과 실제 후보를 같은 명시 글꼴·96 DPI 조건으로 출력한 [Native 전쪽 대조](seo-native-exact.tsv)는
388쪽 모두 raw 하락이 없고 PNG도 바이트 단위로 같다. 후보 최저 raw는 98.928024502297%다.
본문·글꼴 대조군은 compat 2022, 편람은 compat 2024를 사용했다. 편람은 이 PR의 보정 대상이 아니라
리베이스로 반영한 기능 보존 확인에만 사용했다. [로컬 글꼴 해시](seo-rii-local-fonts.json)에서
일부는 원 통합 PR의 해시와 다르므로 그 점수와 직접 비교하지 않고 현재 환경에서 양쪽을 새로
출력했다. 글꼴 프로그램 자체는 제출하지 않는다.

기준 Native 실행 파일 SHA-256은 `3d35006a37ab9e819c3eba55bd37e36976c0efd9a1bb807e8fdc8f6dde6e654c`,
후보는 `8b81ce61050b7cc15557c26fa0c8aafce2e9d5b5d79bfffb15a3d170da2590f8`이다.

#8389의 기존 입력 8개도 원 PR과 같은 원본·Print PDF·글꼴 조건으로 실제 후보에서 다시 출력했다.
[Native 전쪽 정밀 대조](pr8389-native-exact.tsv)는 972쪽 중 하락 0쪽, 동일 956쪽, 개선 16쪽이다.
하락 여부는 표시된 소수값이 아니라 미대응·합집합 정수의 분수로 판정했다. issue #7905의 6쪽은
모두 98% 이상이며 최저 raw는 98.860207844452%다. 편람과 sample16은 이 PR의 보정 범위가 아니라
리베이스로 반영한 #8389의 기능 보존 확인 대상으로만 사용했다.

## Direct PDF의 연결선 보존

#8394가 직접 PDF 출력을 지원한 원본 63쪽과 289쪽을 새 기준과 실제 후보에서 각각 출력했다.
`export-pdf --backend direct --profile print --compat 2024`에 0부터 시작하는 쪽 번호 62와 288을
사용했다. 두 결과는 각각 한 쪽이며, 독립 한컴 Print와 96 DPI로 대조할 때 raw 미대응 픽셀이
모두 0이다. 기준과 후보의 Direct PDF 래스터도 바이트 단위로 같다. 선·화살촉과 상자 배치를
직접 확인했으며 색상·외곽선에는 미세한 잔차가 있다. 이 확인은 편람 전체의 Direct PDF 지원을
뜻하지 않는다.

| 원본 쪽 | raw 미대응 / 합집합 | review | overlay |
| --- | ---: | --- | --- |
| 63 | 0 / 99,670 | [비교](direct-pdf-p063-review.png) | [겹침](direct-pdf-p063-overlay.png) |
| 289 | 0 / 203,464 | [비교](direct-pdf-p289-review.png) | [겹침](direct-pdf-p289-overlay.png) |

WASM 전쪽 비교와 Native 캐시·타일·Direct PDF 집중 검사는 별도로 완료해 기록한다.
