# PR #6618 검토 - PDF 하위 SVG 비트맵 보존

- 원 PR head: `7351fe3e21b61fcb581dfd8db9dc27f1c9cf2e5b`
- 통합 기준: `upstream/devel` `51043f5f8d0453b9bc929233de443fa60cb3df4b`
- 통합 후보: `7946a457d`를 포함한 `9088bd705cafd004d703fcf4fa1a40002e9e3bee`
- reviewer: `jangster77` 요청 완료

## 판정: 승인

`#6612`는 SVG 안의 비트맵이 `export-pdf` XObject까지 전달되도록 보장한다. PDF 헤더와 이미지 XObject 수를 확인하는 focused 회귀 및 전체 nextest가 통과했다.

## 검증 및 증적

- `hwp3-sample14-hwp5` 1쪽의 SVG/PDF 직접 비교: `9.62%`.
- [stable review PNG](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6618_issue6612_p001.png)

원 PR은 직접 merge하지 않고 승인된 통합 PR에서만 수용한다.
