# PR #3447 검토 기록 — CLI 복학원서 편집 데모

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3447](https://github.com/edwardkim/rhwp/pull/3447) — `docs(report): CLI 편집 작동 사례 — 고려대 복학원서 실물 서식 채우기` |
| 작성자·검토자 | `@kevin9327` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `f9f3073472f6661b14f356c764693a2fecf51497` (`pr/task-bokhak-edit-demo`) |
| 원 변경 규모 | 5 mydocs files, +52 / -0, 4 commits |
| 통합 검토 | `review/kevin9327-20260726-v2`; 최초 기준 `upstream/devel` `732147a30c`, 최신 동기화 `7f8fcfef0`; 원 변경 적용 `0db1f0ee4`·`8acec32ff`·`96eaa2288`·`5e75b5dcb` |
| collaborator 보정 | `a1fe4ce760899f4ad0b12bc5fbddf808611e9dd5` 중 #3447 README 범위 |
| 관련 이슈 | 별도 자동 close 대상 없음 |
| 작성 시점 source 상태 | `MERGEABLE` / `BEHIND`; merge 전 최신 head·required check 재확인 필요 |
| 라우팅 | base: `collaborator_external_pr`; modifiers: `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `review_only_fast_pass` |

Loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_external_pr.md`, `intake_and_review.md`, `local_validation.md`,
`visual_fixture_evidence.md`, `multi_pr_update_branch.md`, `review_only_fast_pass.md`.

## 원 변경 범위와 판정

코드나 fixture를 바꾸지 않고 기존 `samples/복학원서.hwp`를 CLI로 채우는 실사용 여정을 문서와 네 PNG로
기록한다. `edit set-cell`로 안전한 표 값을 채우고, 표 셀이 아닌 본인 성명/서명과 날짜 문단은
`search`에서 얻은 원문을 `edit replace-text`에 전달한다. 접수기관 전용 표와 폭이 매우 좁은 구조용 셀은
편집 대상에서 제외한다.

원 PR 본문과 README에는 표 값 개수를 8개 또는 9개처럼 다르게 설명하고, 일부 재현 예시는 저장소 경로가
아닌 작업 파일을 전제로 했다. 특히 서명 원문의 `인`은 일반 완성형이 아니라 PUA U+F012B이므로 눈으로
재입력한 문자열을 쓰면 `replacedCount: 0`이 된다. 설명과 실제 재현 계약이 어긋난 상태는 작동 사례로
수용할 수 없어 문서를 보정했다.

## Collaborator 보정과 실측 재현

`a1fe4ce76`에서 README를 실제 실행한 순서와 값으로 고쳤다.

- 정확한 원본을 `samples/복학원서.hwp`로 고정하고 복사본에서만 편집하도록 했다.
- 표 0의 안전한 값 셀 **7개**에만 `set-cell`을 적용했다. 일곱 결과의 `oldText`가 모두 빈 문자열임을
  확인했다.
- 서명은 `search ... "Signature" --json`의 실제 PUA 포함 문자열을 그대로 사용했다.
- 날짜는 원본의 `momth` 오타를 고유 키로 검색해 해당 문단 한 곳만 치환했다.
- 서명과 날짜 `replace-text`의 `replacedCount`는 각각 `1`이었다.
- 편집 뒤 `export-tables`를 다시 실행해 표 3개와 입력한 값이 보존됨을 확인했다.

기여자 원 commit과 PNG는 유지하고, 재현 절차 정정만 별도 collaborator commit으로 더했다.

## Renderer·fixture·baseline·시각 자료

- 기존 fixture: `samples/복학원서.hwp`
  (`SHA-256 da81b4010331bcac290f900c7cf224c97ee8355399614725ce46c197ff1a22a4`).
- 이 PR은 `mydocs/report/edit_demo_bokhak/` 아래 문서·PNG만 추가한다. 기존 HWP/HWPX/PDF를
  수정·교체·이동하지 않아 IR field sweep baseline 등록 trigger가 없고, renderer 코드도 바꾸지 않는다.
- PNG는 CLI 작동 여정과 함정을 설명하는 자료다. renderer 변경의 merge 판정을 위한 visual sweep 대상은
  아니므로 별도 sweep을 수행하지 않았다. 픽셀 비율은 기여자 측 측정 기록이며, collaborator 독립 검증은
  위 실제 CLI JSON 계약과 재독 결과를 근거로 한다.

![PR #3447 원본과 set-cell·replace-text 최종본](../../report/edit_demo_bokhak/bokhak-before-after-v2.png)

![PR #3447 표 0 값 셀만 채운 1차 결과](../../report/edit_demo_bokhak/bokhak-before-after.png)

![PR #3447 워터마크·표 테두리·직인란 대조](../../report/edit_demo_bokhak/bokhak-verify-detail.png)

![PR #3447 좁은 구조용 셀 침범 재현](../../report/edit_demo_bokhak/narrow-cell-trap.png)

## 검증

- `export-tables` 구조 확인: 표 3개.
- `edit set-cell` 7회: 모두 `oldText == ""`.
- PUA-safe 서명 `replace-text`: `replacedCount == 1`.
- 날짜 `replace-text`: `replacedCount == 1`.
- 최종 `export-tables` 재독과 `export-svg`: 입력값 보존 및 산출 성공.
- `git diff --check`: 통과.
- 통합 후보 공통 게이트: release build PASS; release lib `2943 passed / 0 failed / 7 ignored`;
  release-test all targets와 IR sweep `2/2` 통과; Native Skia 3종, fmt, clippy, doc test,
  wasm-pack 모두 통과.

원 source PR 자체는 review-only 허용 범위에 해당한다. 다만 현재 통합 후보에는 다른 PR의 source·test
보정이 함께 있으므로 통합 PR 전체를 문서-only fast-pass로 오판하지 않고 full CI를 적용한다.

## Risk와 최종 권고

가장 큰 위험은 구조용 빈 셀이나 접수기관 문구를 신청자 입력란으로 오인해 덮어쓰는 것이다. 보정 문서는
`oldText`와 `replacedCount`를 제출 전 hard gate로 삼고, 표 1·표 2를 명시적으로 제외해 이 위험을 드러낸다.
**메인터너 보정 후 기술적으로 수용 가능**하다.

#3445의 범위 고정은 당시 열린 PR을 v0.8.2 핫픽스 기준선에서 제외한 것이며,
[해당 릴리즈는 완료](../../report/task_m100_3445_report.md)됐다. 현재 보류로 확장하지 않는다. 최신 통합
head CI·mergeable 상태가 성공하면 반영하고, 원 PR에는 통합·검증 결과를 연결한다.
