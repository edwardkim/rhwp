# PR #2658 검토 기록

| 항목 | 내용 |
|---|---|
| PR | [#2658](https://github.com/edwardkim/rhwp/pull/2658) |
| 작성자 / base | [@postmelee](https://github.com/postmelee) / `devel` |
| reviewer | [@jangster77](https://github.com/jangster77), [@edwardkim](https://github.com/edwardkim) |
| 관련 이슈 | [#2656](https://github.com/edwardkim/rhwp/issues/2656) |
| 범위 | Chrome/Edge 설정 저장·복구, options 초기화·오류 처리, 다운로드 자동 열기 중복·fail-closed 방어 |
| 처리 경로 | 원 PR head 보강 후 재검토·merge 완료, 옵션 2 후속 기록 PR로 최종 상태 보존 |
| merge 결과 | 2026-07-22, merge commit `bb99b8903bb3cec077ff808a6573a87b96739ee9` |
| 이슈 결과 | [#2656](https://github.com/edwardkim/rhwp/issues/2656) auto-close 확인 |

## 검토 결론

저장 실패를 성공으로 표시하던 options UI, 초기 로딩 경쟁, 설치 시 기본값 덮어쓰기, sync read 실패,
동일 download id 중복 탭 위험에 대한 기존 PR 보강은 유지할 수 있다.

[requested-changes review](https://github.com/edwardkim/rhwp/pull/2658#pullrequestreview-4745744846)에서
지적된 blocker는 local snapshot이 없는 기존 설치의 partial sync였다. update 직후 sync에서
`autoOpen`만 누락되면 기존 구현은 read 성공으로 판정해 기본값 `true`를 반환하고 local snapshot에도
굳혔다.

보강안은 다음 두 시점을 모두 방어한다.

- `update`와 `chrome_update`에서 현재 유효한 설정을 local snapshot으로 선보존한다.
- legacy key 또는 schema metadata만 남은 partial sync는 clean install과 구분하고, 유효한
  `autoOpen` 근거가 없으면 자동 동작을 fail-closed 처리한다. 이 상태의 default `true`는
  last-known-good snapshot으로 기록하지 않는다.

clean install의 `autoOpen=true` 기본 동작과 유효한 local snapshot 복구는 별도 회귀 테스트로 유지했다.
따라서 requested-changes의 재현 경로는 로컬 구현 기준으로 해소됐다.

## 최종 처리

보강된 최신 head `31af56fb0df360e98462896446cc29235fc4b97b`에서 reviewer 재검토와 최신 GitHub
Actions 성공을 확인한 뒤 [#2658](https://github.com/edwardkim/rhwp/pull/2658)을 merge했다. merge commit은
`bb99b8903bb3cec077ff808a6573a87b96739ee9`이며, `Closes #2656`에 따라
[#2656](https://github.com/edwardkim/rhwp/issues/2656)이 2026-07-22에 자동 종료된 것도 확인했다.

이 문서는 원 PR에 먼저 포함된 보류 단계 기록을 merge 이후 확정 사실로 보완하는 옵션 2 후속 기록이다.

## 렌더 영향과 시각 검증

변경 범위는 Chrome 확장의 storage와 Service Worker 제어 흐름, 테스트, 관련 문서다. Rust, WASM,
renderer, layout, golden, 샘플은 변경하지 않으므로 visual sweep 대상이 아니다.

## 로컬 검증

- 보강 전 red: 2 failed / 15 passed
- settings-store + lifecycle: 19 passed, 0 failed
- Chrome options + service worker: 41 passed, 0 failed
- shared + Firefox 다운로드 회귀: 76 passed, 0 failed
- Chrome/Firefox dist 계약: 3 passed, 0 failed
- 최신 `upstream/devel` 동기화 후 Chrome/Firefox 확장 빌드: 각각 169 modules transformed, 성공
- source/dist `background.js`, `settings-store.js`, `extension-lifecycle.js`, `options.js` byte 비교: 통과
- 변경 JavaScript `node --check`: 통과
- `git diff --check`: 통과
- reviewer 재검증: Chrome/Firefox/shared service worker 113 passed, Chrome options UI 4 passed
- reviewer 재검증: `wasm-pack build --target web --out-dir pkg` 후 Chrome/Firefox production build 성공,
  확장 배포 산출물 계약 3 passed
- 최신 GitHub Actions: CI, CodeQL, Render Diff의 실행된 check 모두 성공

## 보류 단계 기록과 최종 확인

- 최초 기록의 `CHANGES_REQUESTED`와 `BEHIND`는 보강 전 head를 기준으로 한 과거 참고값이다.
- 최종 확인 시점에는 보강 코드가 포함된 최신 head의 GitHub Actions가 통과했고, requested-changes
  reviewer가 `APPROVED`로 재검토를 마쳤다.
- 이슈 auto-close 봇 코멘트만으로 끝내지 않고, merge commit·로컬 검증·최신 CI 결과를 이 archive 기록과
  옵션 2 오늘할일 갱신에 함께 남긴다.

requested-changes가 단일 동작 영역으로 수렴하고 실행 순서도 commit, devel sync, 재검증, push,
재검토로 고정되어 별도 `pr_2658_review_impl.md`는 작성하지 않는다.
