# PR #6609 검토 - 머리말·꼬리말 부동 그림의 기준 틀

- 원 PR head: `a84c90ab64f1ae285086b145f191f717a59c6447`
- 통합 기준: `upstream/devel` `51043f5f8d0453b9bc929233de443fa60cb3df4b`
- 통합 후보: `9c7caced2`, `9ab0edb0d`를 포함한 `9088bd705cafd004d703fcf4fa1a40002e9e3bee`
- reviewer: `jangster77` 요청 완료

## 판정: 승인

`#6608`은 머리말 그림의 기준 영역을 paper margin이 아니라 header frame으로 바로잡는다.

## 검증 및 증적

- `pic-in-head-02.hwp` 1쪽과 6쪽을 `pdf/pic-in-head-02-2022.pdf`와 비교했다. page diff는 `17.51%`, `14.09%`이며 대상은 header frame origin이다.
- [p1 PNG](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6609_issue6608_p001.png), [p6 PNG](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6609_issue6608_p006.png)

원 PR은 직접 merge하지 않고 승인된 통합 PR에서만 수용한다.
