# PR #1942 처리 계획

## 적용 커밋

- `d9bb870e47e89ab22a2c036bdac10443f5b3edca` Issue #1932: UTF-16/UTF-8 관용 디코딩 폴백

## 처리 기록

- PR #1940 적용 뒤 #1942를 cherry-pick.
- `src/parser/hwpx/reader.rs` 충돌은 #1940 대형 XML 테스트와 #1942 invalid UTF-8 lossy 테스트를 모두 유지하도록 해결.

## 후속 절차

- merge 전 `gh pr view 1942`로 최신 head, mergeability, checks 재확인.
- PR #1942 merge 후 #1932에 남은 암호화 HWPX 축이 #1958에서 처리되는지 함께 기록한다.
