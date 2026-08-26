---
kind: pr-review
status: active
pr: 5719
---

# PR #5719 검토 - 올드스쿨 스킨과 첫 실행 스킨 선택 안내

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#5719](https://github.com/edwardkim/rhwp/pull/5719) |
| 작성자 | `keepYaoung` |
| base / head | `devel` / `feat/theme-oldschool` |
| 구현 후보 head | `0d0b62ab13badfde26c0e9319da82e77417307bf` |
| 관련 기준 | [#4714](https://github.com/edwardkim/rhwp/pull/4714) (merged: 플랫 스킨·테마 토큰 기반) |
| 변경 범위 | 올드스쿨 스킨, 첫 실행 선택 대화상자, 설정 마이그레이션, 테마 문서·정적 테스트 |

## 변경 판정

- `oldschool`을 옵트인 스킨으로 추가하고, 기존 저장값 `default`·`flat`은 각각
  클래식·모던이라는 사용자 노출 명칭으로 계속 호환한다.
- `skinChosen`을 설정에 추가해 새 사용자는 한 번만 선택 안내를 받고, 기존에 `flat` 또는
  `oldschool`을 고른 사용자는 안내를 다시 보지 않는다.
- 초기 `theme-init.js`, 런타임 `theme.ts`, 보기 메뉴, 첫 실행 카드가 동일한 스킨 값을
  사용한다. 올드스쿨의 라이트 팔레트는 다크 모드 가드 밖으로 새지 않도록 분리했다.
- 임베드·브리지·iframe 환경과 렌더러 초기화 실패 경로에서는 안내를 표시하지 않아 호스트 UX와
  실패 화면을 방해하지 않는다.

## 완료한 로컬 검증

- 최신 `upstream/devel` 위 누적 적용 및 `git merge-tree --write-tree upstream/devel HEAD`: 성공
- `node --test tests/theme-skin.test.ts tests/user-settings.test.ts` (`rhwp-studio`): 17 passed
- `CARGO_TARGET_DIR=target/pr-5719-wasm wasm-pack build --target web --out-dir pkg --dev`: 성공
- `npm --prefix rhwp-studio run build`: TypeScript 검사와 Vite 프로덕션 번들 성공
- 구현 후보 CI: Frontend package gates, Canvas visual diff, CodeQL, adapter inter-diff,
  prop roundtrip 등 현재 required check 성공

## 위험과 범위 밖

- Vite가 보고한 500KB 초과 chunk 경고는 기존 번들 크기 경고이며 이번 스킨 기능의 빌드 실패는 아니다.
- 별도 브라우저 수동 조작 검증은 하지 않았고, 새 trailing head의 CI로 재확인한다.
- 관련 번호 #4714는 이미 병합된 선행 PR이므로 종료할 열린 이슈는 없다.

## 결론

**CI 검증 대기.** 이 검토 기록과 오늘할일을 trailing docs commit으로 PR #5719에 push한다.
최신 head의 required check와 mergeability를 다시 확인한 뒤 병합하고, source branch·review
worktree·검토 전용 WASM target은 `post_merge.md` 순서로 정리한다.
