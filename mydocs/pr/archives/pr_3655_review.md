---
kind: pr-review
status: active
---

# PR #3655 검토 — 셀 문단 병합 메타 회귀 경로 보강

| 항목 | 값 |
| --- | --- |
| 작성자 / reviewer | `@lpaiu-cs` / `@jangster77` |
| 원 PR / 관련 이슈 | [#3655](https://github.com/edwardkim/rhwp/pull/3655) / [#3439](https://github.com/edwardkim/rhwp/issues/3439) |
| 원 head 참고값 | `de0bf178b746bb54f02843ab2109c085a7638db2` |
| 통합 후보 | [#3661](https://github.com/edwardkim/rhwp/pull/3661) `52903c91bf132f7f3a977afc9cc265859b024c85` |
| 원 변경 규모 | 1 file, +232 / -63 |
| 권고 | #3661로 수용. merge 시 #3439 close |

## 변경과 통합 판정

이미 고쳐진 `Paragraph::capture_meta` / `apply_meta` 계약이 table cell 외의 문단 컨테이너와
실제 깊이 2 `by-path`에서도 유지되는지를 실물 `DocumentCore` 조작으로 넓힌 test-only PR이다.

- table cell, textbox, picture caption의 각 문단 컨테이너에서 병합 뒤 undo 분할을 수행한다.
- 사라진 둘째 문단의 para shape·style·column break·numbering restart·raw header extra·extended tab을
  함께 복원하고, 첫 문단의 메타가 바뀌지 않음을 확인한다.
- 중첩 표는 상위 셀 안에 실제 table control을 만들고 depth-2 path로 insert/split/merge/undo 한다.
  그래서 얕은 cell index를 by-path처럼 취급하는 false positive를 피한다.
- 일반 Enter split에서 meta를 넘기지 않을 때 앞 문단 상속이라는 기존 동작도 유지한다.

기능 commit `de0bf178b`은 통합에서 `e2a3cd1ff`와 patch 동등하다. production 동작을 변경하지
않으므로 source branch devel merge는 제외하고 회귀만 누적했다.

## 검증과 권고

| 검증 | 결과 |
| --- | --- |
| source #3655 CI | full CI, CodeQL, Canvas visual diff success |
| 통합 code head CI | lint·WASM check, frontend package gates, Native Skia, archive, default-feature 8 shards, CodeQL, Canvas visual diff, `Build & Test` 모두 success |
| 체리픽 동등성 | `de0bf178b` = `e2a3cd1ff` patch-id, `git diff --check` 통과 |
| 추가 로컬 Cargo | exact integration CI와 중복되므로 작업지시에 따라 실행하지 않음. 성공 근거로 사용하지 않음 |

**권고: 수용.** #3439의 남은 셀 컨테이너 경로를 직접 재현하는 회귀로 범위가 정확하다. #3661의
동일 code head full CI를 확인했으며, 문서-only fast-pass 뒤 통합 merge 때 #3439의 자동 close 상태를 확인한다.
