---
kind: pr-review
status: code-ci-pending
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #6004 검토 - 다른 이름 저장 문서명과 최근 문서 갱신

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 | [#6004](https://github.com/edwardkim/rhwp/pull/6004) / `@jangster77` |
| 관련 issue | #6003 |
| base / source head | `devel` / `3cdd875d8038d879edba912938a26292781ea152` |
| 작성 시점 참고 상태 | non-draft, `OPEN`, merge 상태 `BLOCKED` (CI 대기) |

## 변경 검토

- 다른 이름 저장의 File System Access 성공 경로는 새 handle, 파일명, 저장 형식을 최근 문서에 기록하고
  상태 표시줄을 새 파일명과 페이지 수로 갱신한다.
- 다운로드 fallback도 handle 없는 최근 문서 메타데이터를 기록하고 같은 상태 갱신을 수행한다. 취소와
  실패 경로는 저장 완료 처리에 도달하지 않으므로 기존 문서 상태를 보존한다.
- 최근 문서 영속 상한은 20개로 늘린다. 메뉴는 기본 8개를 표시하며, 항목이 더 있으면 `최근 문서 더보기`로
  확장해 최대 20개를 보여 준다.
- 암호 저장 경로에서는 암호 문자열의 영속화·로그·파일명 전달을 금지하는 기존 계약을 지킨다. 최근 문서 기록의
  비핵심 실패는 이 경로에서 로그를 남기지 않는다.

## 로컬 검증

- `npm --prefix rhwp-studio run build`: 통과.
- `npm --prefix rhwp-studio test`: 1,075 passed, 1 skipped, 0 failed.
- `cargo fmt --all -- --check`, `git diff --check`: 통과.
- 실제 브라우저의 Save As 파일 선택 대화상자에서 새 파일명·최근 문서 항목을 확인하는 수동 검증은 아직
  수행하지 않았다. CI와 이 검증 결과를 확인한 뒤에만 최종 병합 판정을 한다.

## 판정

**CI 및 실제 브라우저 검증 대기.** 코드·프런트 계약 검증은 통과했지만, 새 PR head의 CI와 실제 Save As
대화상자 흐름은 아직 완료되지 않았다.
