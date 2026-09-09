# PR #6602 검토 - 부동 그림의 바깥 여백

- 원 PR head: `90bcefebbaaea660c8fcbad6a709ce1accf7cd1f`
- 통합 기준: `upstream/devel` `51043f5f8d0453b9bc929233de443fa60cb3df4b`
- 통합 후보: `0b1062f39`, `266d1a226`을 포함한 `9088bd705cafd004d703fcf4fa1a40002e9e3bee`
- reviewer: `jangster77` 요청 완료

## 판정: 승인

`#6596`의 부동 그림은 바깥 여백 상자를 점유하되 잉크는 그 안쪽에 배치해야 한다. 세 원본 문서의 IR 회귀와 전체 nextest가 통과했다.

## 검증 및 증적

- `hwp3-sample5`, `hwp3-sample`, `온새미로` 1쪽을 각각 canonical 한컴 2020 PDF와 직접 비교했다. page diff는 각각 `16.52%`, `7.73%`, `2.80%`이며 글꼴 래스터 차이를 포함한다.
- [sample5](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6602_issue6596_hwp3_sample5_p001.png), [sample](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6602_issue6596_hwp3_sample_p001.png), [온새미로](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6602_issue6596_onsaemiro_p001.png)

원 PR은 직접 merge하지 않고 승인된 통합 PR에서만 수용한다.
