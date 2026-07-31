---
kind: pr-review
status: active
---

# PR #3651 검토 — ClickHere `fieldBegin@dirty` 왕복 보존

| 항목 | 값 |
| --- | --- |
| 작성자 / reviewer | `@JamesPsh` / `@jangster77` |
| 원 PR / 관련 이슈 | [#3651](https://github.com/edwardkim/rhwp/pull/3651) / [#3545](https://github.com/edwardkim/rhwp/issues/3545) |
| 원 head 참고값 | `9f1c2addf5ae8e3968394f2ffff0967e634359f1` |
| 통합 후보 | [#3659](https://github.com/edwardkim/rhwp/pull/3659) `00450ccb1cb075c688b037462c760e1f4dd23700` |
| 원 변경 규모 | 4 files, +170 / -2 |
| 권고 | #3659로 수용. #3545의 잔여 설계 축은 open 유지 |

## 변경과 통합 판정

HWPX parser가 `hp:fieldBegin@dirty`를 버리고 serializer도 방출하지 않아, ClickHere 필드의
`properties` bit 15가 HWPX 축에서는 항상 0이 됐다. 그 결과 안내문과 같은 실제 값을 채운 필드는
`clear_initial_field_texts` 정규화에서 다시 사라질 수 있었다.

PR은 `dirty`를 bit 15로 읽고, `Field::is_dirty()`를 통해 자기닫힘과 자식 보유 `fieldBegin` 두
직렬화 경로에서 모두 다시 쓴다. 새 `issue_3545_clickhere_dirty_roundtrip` 회귀는 실제
`samples/hwpx/form-01.hwpx`를 사용해 다음 경계를 고정한다.

- 안내문과 같은 값을 채운 뒤 HWPX 저장·재적재해도 값이 남는다.
- 입력 상태의 `dirty="1"` 파일은 적재만으로 값이 지워지지 않는다.
- 초기 상태와 채운 상태가 각각 `dirty="0"`·`"1"`로 방출된다.
- 초기 상태 `dirty="0"`의 기존 정규화는 바뀌지 않는다.

원 head의 `devel` 병합 commit `9f1c2addf`은 통합에서 제외했다. 기능 commit `1ac0270b1`은
통합 branch에서 `00450ccb1`으로 patch 동등하게 적용됐고 충돌은 없었다.

## 검증

| 검증 | 결과 |
| --- | --- |
| 원 #3651 head CI | lint, Native Skia, default-feature 8 shards, `Build & Test`, CodeQL, Canvas visual diff success |
| 통합 #3659 code head CI | 동일 full CI, CodeQL 및 `Build & Test` success |
| HWPX 회귀 | 실제 form fixture의 parser·serializer·재적재와 초기 상태 무회귀를 한 integration test에 고정 |
| 체리픽 동등성 / diff | `git range-diff` 동등, `git diff --check` 통과 |
| 추가 로컬 Cargo | 원 PR 및 정확한 통합 head CI와 중복되므로 작업지시에 따라 실행하지 않음. 성공 근거로 사용하지 않음 |

원 PR CI는 [CI run](https://github.com/edwardkim/rhwp/actions/runs/30627559514), 정확한 통합
code head CI는 [#3659 CI run](https://github.com/edwardkim/rhwp/actions/runs/30627974311)에서 확인했다.

## 남은 범위

이 PR은 입력된 필드의 보존 표식 왕복만 해결한다. 초기 상태 `dirty="0"` 필드의 안내문 텍스트를
IR 적재에서 물리 삭제하는 기존 정규화를 무손실 해석으로 바꿀지는 renderer·API·roundtrip 계약에
영향을 주는 별도 설계 결정이다. 이를 해결된 것으로 표시하지 않고 #3545를 open으로 유지한다.

## 권고와 merge 전 조건

**권고: 수용.** #3659의 code head full CI가 성공했고 상태는 작성 시점 `CLEAN`·`MERGEABLE`이다.
archive review·통합 계획·오늘할일만 추가한 review-only head의 fast-pass를 확인한 뒤 #3659을
merge한다. merge 뒤 #3543 close, #3545 open 상태, 원 PR #3651의 supersede 처리와 contributor
결과 comment, `devel` 동기화와 검토 ref 정리를 확인한다.
