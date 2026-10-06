# 작업 2472 단계 1 - HML 표 TextWrap 왕복 통합

## 범위

- 현재 메인터너 브랜치에 기여자 PR #2472를 통합한다.
- HML을 읽을 때 표 `SHAPEOBJECT`의 `TextWrap`을 보존한다.
- 사각형 경로는 바꾸지 않으며, focused HML serializer 테스트로 표와 비표 질의 동작을 모두
  다룬다.

## 검토 근거

- PR 본문: `write_shape_object`는 이미 표의 `common.text_wrap`을 직렬화하지만, 읽기 경로는
  사각형에 대해서만 이를 복원했다.
- PR 코멘트: 없음.
- 원격 CI 실패: `cargo fmt --check`뿐이므로 cherry-pick 뒤 로컬에서 서식을 보정한다.

## 검증 계획

1. focused HML serializer 테스트를 실행한다.
2. `cargo fmt --all --check`와 `cargo clippy --all-targets -- -D warnings`를 실행한다.
3. 최종 통합 PR 전 통합 전체 회귀에 이 그룹을 포함한다.
