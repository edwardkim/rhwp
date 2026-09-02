# PR #6605 검토 - 글자처럼 그림 바깥 여백 stack

- 원 PR head: `c2271acd49f69f50b8e2c71a9fd24f224f6dbe03`
- 통합 기준: `upstream/devel` `51043f5f8d0453b9bc929233de443fa60cb3df4b`
- reviewer: `jangster77` 요청 완료

## 판정: 머지 보류 (중복)

이 head의 patch-id는 #6595와 동일하여 통합 후보에 별도 cherry-pick하지 않았다. 직접 merge하면 같은 변경을 중복 적용한다.
`#6607` stack에서 보존된 #6603 선행 변경은 #6607 review record와 시각 증적으로 따로 검토했다.

## 처리

통합 PR이 승인·병합될 때까지 원 PR의 상태나 contributor branch는 변경하지 않는다. 통합 후 source PR 처리만 post-merge 절차로 수행한다.
