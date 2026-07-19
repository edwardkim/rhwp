---
kind: memory
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-17
---

# PR 본문 한국어 작성 필수

내부 타스크 PR의 제목과 본문은 한국어로 작성한다.

이 프로젝트의 문서 생성 규칙은 "모든 문서는 한국어로 작성"이며, 내부 타스크 PR 본문도 외부 공개 문서이자 작업 보고 문서의 일부로 취급한다. `Summary`, `Verification`, `Related to` 같은 템플릿 문구를 그대로 영어로 사용하지 않는다.

허용 예외:

- 명령어, 파일명, 코드 식별자, 로그에 나온 고유 문자열
- GitHub가 자동으로 표시하는 UI/체크 이름
- 외부 기여자 PR에서 원문 인용이 필요한 경우

권장 형식:

```markdown
## 요약
- ...

## 검증
- cargo test ...

관련 이슈: #1234
```
