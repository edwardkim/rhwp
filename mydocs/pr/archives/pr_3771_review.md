---
kind: review
status: active
canonical: mydocs/pr/archives/pr_3771_review.md
last_verified: 2026-08-03
---

# PR #3771 검토 - F5 셀 크기 조절과 선택 상태 한컴 정합

## 라우팅

```text
base route: collaborator_external_pr
modifiers: intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, post_merge.md
review 보정 시작 원격 head: efd25b455686847ac27140d2183988859f6ca63c
보정 code candidate: e7fb4f5bebaf9f40aec2395c683b18c0f3842c34
```

## PR metadata

| 항목 | 값 |
| --- | --- |
| PR | [#3771](https://github.com/edwardkim/rhwp/pull/3771) |
| 관련 issue | [#3770](https://github.com/edwardkim/rhwp/issues/3770), PR 본문의 `closes #3770` |
| 작성자 | `@enigma-jerry72` |
| base / head | `devel` / `feat/table-cell-resize-hancom-modes` |
| 보정 시작 원격 head | `efd25b455686847ac27140d2183988859f6ca63c` |
| 원격 변경 규모 | 8 files, +579 / -32 (보정 전 작성 시점 참고) |
| 원격 상태 | open, ready, mergeable, CLEAN (보정 전 작성 시점 참고) |

## 변경 범위와 보정

기여자 원 변경은 F5 셀 선택 뒤 `Ctrl`/`Alt`/`Shift`+방향키의 셀 크기 조절을 한컴 3모드로
맞춘다. 메인터너 보정은 같은 PR head 위에 별도 commit으로만 추가했고 원 contributor commit은
rewrite하지 않았다.

| commit | 보정 |
| --- | --- |
| `43f7c54df` | `Alt` 조절을 선택 열/행과 바로 오른쪽/아래 이웃의 반대 delta로 제한했다. |
| `22e3595c2` | 병합 셀이 차지하는 실제 열/행 범위를 Alt 조절 대상에 반영했다. |
| `7a5785856` | F5 phase 1 이동 때 내부 문서 위치를 새 셀 시작점으로 동기화했다. |
| `85e87a044` | F5 셀 선택 중에는 내부 위치와 관계없이 텍스트 캐럿을 숨겼다. |
| `e7fb4f5be` | F5 셀 선택에서 `Escape`를 누르면 표 개체 선택으로 전환하지 않고 마지막 선택 셀의 편집 캐럿을 복원했다. |

`b22822f67`은 collaborator 외부 PR 보정 시 가시성 branch를 하나로 연속 사용하도록 review
워크플로 문서를 정정한 운영 기록이다.

## 로컬·사용자 검증

다음 명령을 `rhwp-studio`에서 실행해 통과했다.

| 검증 | 결과 |
| --- | --- |
| `node --test tests/cell-selection-caret-sync.test.ts tests/table-keyboard-navigation.test.ts` | 6 / 6 통과 |
| `npx tsc --noEmit` | 통과 |
| `npm test` | 749 / 749 통과 |
| `npm run build` | 통과 |
| `git diff --check` | 통과 |

작업지시자는 Vite 개발 서버에서 다음 상호작용을 직접 검증했다.

1. 셀 안에서 `F5`를 누르면 셀 선택 하이라이트만 보이고 텍스트 캐럿은 숨겨진다.
2. phase 1에서 방향키로 이동하면 하이라이트가 목표 셀로 이동한다.
3. `Escape`를 누르면 표 전체 개체 선택 핸들이 생기지 않고 마지막 선택 셀에 편집 캐럿이 복원된다.

이번 보정은 키보드 선택 상태와 DOM overlay만 바꾸며 문서 layout, PDF/SVG 출력, fixture를 변경하지
않는다. 따라서 정적 PDF/SVG visual sweep과 신규 asset은 적용 대상이 아니다. 위 사용자 브라우저
상호작용 검증을 사용자-visible 근거로 사용했다.

## 위험과 범위 밖

- F5 phase 2의 다중 셀 범위 선택과 phase 3 전체 선택에서 `Escape` 뒤 caret 위치 규칙은 이번 단일 셀
  phase 1 보정 범위 밖이다.
- 표 객체 선택은 일반 편집 상태에서의 기존 `Escape` 경로를 유지한다. F5 셀 선택 전용 `Escape`만
  마지막 셀 편집 상태로 복귀한다.

## 최종 권고

작업지시자가 사용자 검증과 원격 push, 최신 head CI 완료 뒤 merge를 승인했다. 최신 보정 head의 full
GitHub Actions 성공 및 mergeable 상태를 재확인한 뒤에만 merge한다. merge 뒤에는 contributor PR에
보정 범위와 검증 결과를 남기고, `devel` 동기화와 branch 정리를 수행한다.
