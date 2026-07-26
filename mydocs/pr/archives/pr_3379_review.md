# PR #3379 검토 기록 — 10k 한글 오라클 서베이 r24

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3379](https://github.com/edwardkim/rhwp/pull/3379) — `docs: 10k 한글 오라클 서베이 r24` |
| 작성자·검토자 | `@planet6897` · `@jangster77` |
| 원 기능 커밋 / 최신 source head | `9aa24bc1b682dcf808e39a7c9c87cab0644551c4` / `632b7a87f9f523e89070cca52308dfb9b3107329` |
| 통합 후보 적용 | `review/planet6897-20260726`의 `a4ec5330e` (`-x` cherry-pick) |
| 기준 | 로컬 검증 시작점 `upstream/devel` `ace187d52`; 최신 source head `632b7a8`은 이후 `devel` merge를 포함 |
| 변경 범위 | `mydocs/report/survey_10k_r24_20260726.md` 1개 문서, +60 줄 |
| 라우팅 | collaborator 통합 PR: `collaborator_self_merge` + intake/review·local validation·multi-PR 보조 절차 |

## 범위·내용 판정

이 PR은 코드·fixture·CI 설정을 바꾸지 않고, 폰트 관련 PR 두 건을 포함한 10,000건 한글 오라클
서베이의 r24 결과를 날짜가 붙은 보고서로 보존한다. 반복 측정 보고서의 파일명
`survey_10k_r24_20260726.md`는 기존 r23 계열과 같은 수명주기·명명 방식이며, 새 canonical fixture나
baseline TSV를 추가하는 변경은 아니다.

보고서가 가리키는 원시 output, `harness_r24_template`, `BINARY_FINGERPRINT`는 이 통합 후보 또는
저장소에서 추적되지 않는다. 따라서 수치와 결론은 **2026-07-26 시점의 측정 스냅샷**으로 읽어야 하며,
후속 CI가 재현·판정할 수 있는 baseline 또는 독립 검증 가능한 증적으로 과장해서는 안 된다. 이 한계를
명시한 역사적 측정 기록으로는 수용 가능하다.

## 검증

- `git diff --check upstream/devel...HEAD`: 통과.
- 기존 r23 반복 보고서와 파일 역할·날짜 기반 명명·보존 위치를 대조했다.
- 문서 단독 변경이므로 Cargo 검증의 직접 대상은 아니다. 같은 통합 후보에 포함된 #3410의 code-path
  검증은 [#3410 검토 기록](pr_3410_review.md)에 남겼다.

## 최종 권고

**통합 PR에 포함해 수용 권고**. 다만 보고서의 결과를 저장소 내에서 재실행 가능한 품질 gate로 쓰지
않고, raw output과 harness를 별도 보존·추적하기 전까지는 해당 날짜의 관측값으로만 참조한다. 통합 PR
최신 head의 CI·merge 가능 상태와 작업지시자 승인이 최종 merge 조건이다.
