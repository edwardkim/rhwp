# 구현계획서 — #3233 review-only merge candidate 검증

## 판정 규칙

PR commit을 최신순으로 읽어 trailing single-parent review-only 커밋을 센다. 그 뒤에 다음 merge commit이
나오면 코드 candidate로 승격한다.

| 조건 | 결과 |
| --- | --- |
| trailing review-only 커밋이 1개 이상 | 계속 확인 |
| merge parent가 정확히 2개이고 현재 `pr.base.sha`를 포함 | candidate SHA로 사용 |
| merge diff가 `mydocs/**` 또는 허용된 신규 검증 자료만 포함 | 계속 확인 |
| candidate의 기존 required 검증이 green | fast-pass |
| 위 조건 하나라도 불만족 | full CI fallback |

`Build & Test`, CodeQL 3개 분석, Canvas visual diff의 기존 identity 검증은 유지한다. 따라서 문서-only
merge라는 형태만 예외로 인정하며, 다른 PR·base·repo에서 생성된 성공 run을 재사용하지 않는다.

## 파일

- `.github/workflows/ci.yml`
- `.github/workflows/codeql.yml`
- `.github/workflows/render-diff.yml`
- `mydocs/manual/pr_review_workflow.md`

세 workflow의 JavaScript는 같은 분기와 실패 이유를 사용해 drift를 방지한다.
