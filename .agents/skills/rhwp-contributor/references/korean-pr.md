# 8단 — 한국어 PR

fmt 게이트가 통과한 뒤에만 연다. 제목과 본문은 한국어다. base 는 `devel`.

## 명령

Windows PowerShell 은 한글 here-string 을 `gh --body-file -` 로 직접
파이프하지 않는다 (`AGENTS.md`). UTF-8 **without BOM** 파일을 쓴다.

```bash
# 1) 본문을 UTF-8 without BOM 으로 저장
# 2) 게이트
cargo fmt --all -- --check
# 3) 푸시
git push -u origin HEAD
# 4) PR
gh pr create --repo edwardkim/rhwp --base devel --head kevin9327:feat/<브랜치> --title "<한국어 제목>" --body-file <본문.md>
```

본문에 `closes #<이슈>` 가 있어야 한다.

## 본문 최소

1. 변경 요약
2. 관련 이슈 `closes #N`
3. 테스트 체크리스트 — **첫 칸은 fmt 게이트**
4. 실행한 명령과 결과
5. (해당 시) 시각 근거, 영수증 JSON

`--body-file` 로 올린 뒤 API 로 한글·선두 BOM·`??` 치환을 확인한다
(`pr_review_workflow.md` 3.4.1).

## 중복

같은 주제의 열린 PR 이 있으면 새로 만들지 않는다.
[exceptions.md](exceptions.md).

예제: [16_korean_pr_body_file.md](../examples/16_korean_pr_body_file.md),
[21_closes_issue.md](../examples/21_closes_issue.md).
