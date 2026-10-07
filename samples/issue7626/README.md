# #7626 재현 문서와 한컴 대조군

| 파일 | 역할 | 크기(byte) | SHA-256 |
| --- | --- | ---: | --- |
| [sample-document.hwpx](sample-document.hwpx) | 공개 gist Base64를 그대로 디코드한 원본 | 33836 | `8fa018bafb94ae023ed1a9be50cd710bec2a09ee19d76dbc755bb1ba0310ed92` |
| [hancom-resaved.hwp](hancom-resaved.hwp) | 같은 원본을 한컴 2020에서 HWP로 다시 저장한 대조군 | 34304 | `5856ad9002cce54a53b831dc7ca557e4da6e3a56f652c738ca74072dea640cfe` |
| [기준 PDF](../../pdf/issue7626-original-2020.pdf) | 원본 HWPX의 한컴 2020 1-up Print 출력, 2쪽 | 104997 | `2fd348b692e2c555bc33dd26c4feb2f5ed5f33e2f3a7ce2214b515836e5d1546` |

출처는 [이슈 #7626](https://github.com/edwardkim/rhwp/issues/7626)과
[원본 gist](https://gist.github.com/flamingo8006/cf171c692b6d4c484027832a060da894)다.
원본 저장 제품 메타데이터는 한컴 2020이며 실제 작성·치환 과정은 확인되지 않았다.
2026-10-07 한컴 2020 `11.0.0.9136`에서 전처리 없이 PDF 인쇄와 HWP 재저장을 각각 실행했다.
재저장 HWP는 원본 HWPX의 시각 검증을 대체하지 않는다.

원본의 52개 저장 LineSeg는 모두 폭 0·세로 위치 0이다. 재저장본에는 실제 배치의 폭과
세로 위치가 있다. 특히 원본 p30의 가시 글자 run은 1000HU, 끝의 빈 run은 1200HU이며,
한컴 재저장 p30은 `lh=1200`, `spacing=720`, `vpos=4000`이다. 다음 p31은 `vpos=5920`이다.
p31의 표 host 폭은 0이어도 구역의 유효한 높이 사다리를 버리면 안 된다.

[조사·시각 검증 기록](../../mydocs/tech/investigations/issue-7626/README.md)에서
원본 전체 Native/fresh WASM 비교, 명령·페이지별 TSV·최종 PNG와 회귀 결과를 연결한다.
