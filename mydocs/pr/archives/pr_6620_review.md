# PR #6620 검토 - 뒤집힌 WMF 창 축 정규화

- 원 PR head: `a3b2bee6a60c4ba6690ded74afc6adf990f82a0a`
- 통합 기준: `upstream/devel` `51043f5f8d0453b9bc929233de443fa60cb3df4b`
- 통합 후보: `cb1df750d`를 포함한 `9088bd705cafd004d703fcf4fa1a40002e9e3bee`
- reviewer: `jangster77` 요청 완료

## 판정: 승인

`#6617`의 y-up 또는 반전된 WMF 창에서 블릿이 viewBox 밖으로 나가던 문제를 장치 좌표 정규화로 해결한다.

## 검증 및 증적

- `bitmap.hwp` 1쪽과 `pdf/bitmap-2022.pdf` 직접 비교: `0.12%`; 기존에 비어 있던 OLE WMF 잉크와 위치가 기준 PDF에 맞게 나타난다.
- [stable review PNG](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6620_issue6617_p001.png)

원 PR은 직접 merge하지 않고 승인된 통합 PR에서만 수용한다.
