# PR #1761 리뷰 구현 메모

## Stage 1. 상태 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1761
- 작성자: @planet6897
- 관련 이슈: #1759
- 변경 범위: Python 조사 도구 + 문서
- 문서 작성 시점 참고값: `mergeable=MERGEABLE`, `mergeStateStatus=BEHIND`
- reviewer assign: `@jangster77`

## Stage 2. 체리픽 누적 검토

완료.

```bash
git checkout -B local/pr1761-1764-review upstream/devel
git cherry-pick 4c984373e9d24ead5b29c13533fceddaab937e05
```

충돌 없음.

## Stage 3. 검증

완료.

- `python3 -m py_compile tools/metric_drift_survey.py`
- `cargo fmt --check`
- `git diff --check upstream/devel..HEAD`
- GitHub Actions 최신 head success 확인

## Stage 3.1. 리뷰 메모

완료.

- PDF 캐시는 `src.stem[:60]` 기반이라 임의 입력에서는 파일명 충돌 가능성이 있다.
- 현재 PR의 hwpdocs 조사 표본은 숫자 ID prefix를 쓰는 고유 파일명 전제라 merge blocker로 보지 않고,
  후속 일반화 후보로 기록한다.

## Stage 4. 다음 작업

- #1761 merge 후 #1762 검토/merge 순서로 진행
- 후속 샘플 복사는 작업지시자가 merge 후 해당 task 디렉터리에 수행 예정
