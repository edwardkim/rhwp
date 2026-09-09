# PR #6607 검토 - 글자처럼 그림·묶음 도형의 바깥 여백

- 원 PR head: `92ee899becb19c501de49a36bc14c99c870e814c`
- 통합 기준: `upstream/devel` `51043f5f8d0453b9bc929233de443fa60cb3df4b`
- 통합 후보: `683a74e36`, `197bbea93`, `7a739e003`을 포함한 `9088bd705cafd004d703fcf4fa1a40002e9e3bee`
- reviewer: `jangster77` 요청 완료

## 판정: 승인

stack에 남은 `#6603` 그림과 `#6606` 묶음 도형의 바깥 여백 규칙을 함께 수용한다. 그림의 잉크 원점과 묶음 자식의 줄 안 위치가 각각 상자 여백 안쪽으로 이동한다.

## 검증 및 증적

- `hwp3-sample14-hwp5` 1쪽 PDF 직접 비교: `9.62%`.
- `draw-group.hwp` 1쪽 PDF 직접 비교: `0.91%`; 묶음의 자식 그림·연결선 위치를 직접 확인했다.
- [#6603 PNG](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6607_issue6603_p001.png), [#6606 PNG](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6607_issue6606_p001.png)

원 PR은 직접 merge하지 않고 승인된 통합 PR에서만 수용한다.
