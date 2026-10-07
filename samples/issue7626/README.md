# #7626 재현 문서와 한컴 대조군

| 파일 | 역할 | 크기(byte) | SHA-256 |
| --- | --- | ---: | --- |
| [sample-document.hwpx](sample-document.hwpx) | 공개 gist Base64를 그대로 디코드한 원본 | 33836 | `8fa018bafb94ae023ed1a9be50cd710bec2a09ee19d76dbc755bb1ba0310ed92` |
| [hancom-resaved.hwp](hancom-resaved.hwp) | 같은 원본을 한컴 2020에서 HWP로 다시 저장한 대조군 | 34304 | `5856ad9002cce54a53b831dc7ca557e4da6e3a56f652c738ca74072dea640cfe` |
| [기준 PDF](../../pdf/issue7626/sample-document-hwpx-2020.pdf) | 원본 HWPX의 한컴 2020 1-up Print 출력, 2쪽 | 104997 | `2fd348b692e2c555bc33dd26c4feb2f5ed5f33e2f3a7ce2214b515836e5d1546` |
| [HWP 대조군 PDF](../../pdf/issue7626/hancom-resaved-hwp-2020.pdf) | 재저장 HWP 자체의 한컴 2020 1-up Print 출력, 2쪽 | 104997 | `fd4fac16f44ff0ade3623d48d8d6527f013298df5dd872a50e9556139639124f` |

출처는 [이슈 #7626](https://github.com/edwardkim/rhwp/issues/7626)과
[원본 gist](https://gist.github.com/flamingo8006/cf171c692b6d4c484027832a060da894)다.
원본 저장 제품 메타데이터는 한컴 2020이며 실제 작성·치환 과정은 확인되지 않았다.
2026-10-07 한컴 2020 `11.0.0.9136`에서 전처리 없이 PDF 인쇄와 HWP 재저장을 각각 실행했다.
재저장 HWP는 원본 HWPX의 시각 검증을 대체하지 않는다.
PR 준비에서는 재저장 HWP 자체도 같은 제품/빌드에서 전처리 없이 별도로 Print했다.
PDF 경로는 원본 형식·엔진을 구별하는 canonical 규칙에 맞추었다.

원본의 52개 저장 LineSeg는 모두 폭 0·세로 위치 0이다. 재저장본에는 실제 배치의 폭과
세로 위치가 있다. 특히 원본 p30의 가시 글자 run은 1000HU, 끝의 빈 run은 1200HU이며,
한컴 재저장 p30은 `lh=1200`, `spacing=720`, `vpos=4000`이다. 다음 p31은 `vpos=5920`이다.
p31의 표 host 폭은 0이어도 구역의 유효한 높이 사다리를 버리면 안 된다.

[조사·시각 검증 기록](../../mydocs/tech/investigations/issue-7626/README.md)에서
원본 전체 Native/fresh WASM 비교, 명령·페이지별 TSV·최종 PNG와 회귀 결과를 연결한다.

## 마지막 물리 줄 반례

`end-run-two-lines.hwpx`는 원본 ZIP의 `Contents/section0.xml`에 있는
`({{table_unit}})` 하나를 같은 문자열 두 개와 `<hp:lineBreak/>`로 바꾼 합성 변형이다.
다른 ZIP entry와 원본의 미배치 저장 줄은 그대로 보존했다. 이 변형을 한컴 2020에서
HWP로 재저장한 파일이 `end-run-two-lines.hwp`다. 두 형식 각각의 독립 1-up Print는
`pdf/issue7626/end-run-two-lines-{hwpx,hwp}-2020.pdf`에 보존한다.

한컴 재저장 p30은 첫 줄 `vpos=4000, th=1000, spacing=600`, 마지막 줄
`vpos=5600, th=1200, spacing=720`이다. 두 줄 모두 선언 `lh=1200`이므로 첫 줄의
진행을 선언 높이만으로 정하지 않는다. 첫 줄 뒤 진행 1600HU와 마지막 줄 뒤
표까지 진행 1920HU를 정식 회귀에서 검사한다. 합성 변형을 실제 원본 저장본으로
분류하거나 그 수동 LineSeg를 캐시 수용 조건의 근거로 삼지 않는다.

| 파일 | SHA-256 |
| --- | --- |
| `end-run-two-lines.hwpx` | `444ca7fe666cf64e9d044718f17a20959d1e42a7a318a739e34c3f6e85e7d16f` |
| `end-run-two-lines.hwp` | `0dd5cf4573901716e58700bfab60f6c308ba58ab5af6d47d7bd6c9e43de11e22` |
| `end-run-two-lines-hwpx-2020.pdf` | `f79a1795430cf2af645ef8724e8f3a1d2e5bcd0cc1ee0d5a1d4e00c4f214bff8` |
| `end-run-two-lines-hwp-2020.pdf` | `9f11266aa551d8122fd80ab656c70a3ee9ca00507d3a5f466a8d9ecaaca12f5a` |

회귀 보강 전 Native/fresh WASM 전쪽 실루엣 gate를 확인했다. HWPX는 양쪽 최저
98.85353%, HWP는 Native 98.85353%·fresh WASM 99.10313%다. 각 입력은 2쪽이며
누락 쪽과 90% 미만 쪽은 없다. 외부 다운로드 경로의 검증 파일과 위 저장소 파일은
SHA-256이 같고, 검증 로그와 중간 원장은 ignored `output/pr-review/issue7626/`에 보존한다.
