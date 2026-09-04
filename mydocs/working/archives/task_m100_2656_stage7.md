---
kind: working-note
status: complete
issue: 2656
stage: 7
---

# Task #2656 Stage 7: PR #2658 partial-sync review 보강

## Review 분석

[requested-changes review](https://github.com/edwardkim/rhwp/pull/2658#pullrequestreview-4745744846)와
[메인테이너 재실증](https://github.com/edwardkim/rhwp/pull/2658#issuecomment-5040233424)은 같은 공백을
가리켰다.

1. 기존 사용자의 sync에 `autoOpen=false`가 있고 local snapshot은 없다.
2. update lifecycle은 진단만 기록해 `false` 복구 근거를 만들지 않는다.
3. 이후 sync에서 `autoOpen`만 빠지고 다른 legacy key가 남으면 읽기 자체는 성공한다.
4. 기존 구현은 이를 clean install처럼 보고 `true`를 반환하며 local snapshot에도 굳힌다.

보강 전에는 lifecycle 통합 테스트가 local snapshot 부재로 실패했고, legacy key만 남은 partial sync
테스트가 기대 `false` 대비 실제 `true`로 실패했다.

## 구현

- `update`와 `chrome_update`는 sync에 쓰지 않고, 수명주기 진단을 기록하기 전에 현재 유효한 설정을
  local snapshot으로 선보존한다.
- settings-store는 전체 sync payload를 읽어 현행 key뿐 아니라 legacy key와 schema metadata도 기존
  설치 정황으로 판정한다.
- 자동 동작 경로는 다음 순서로 `autoOpen` 신뢰 여부를 결정한다.
  - 유효한 sync boolean: 권위 값 사용
  - 유효한 local snapshot: 누락 key 복구
  - sync/local 근거가 모두 없는 clean install: 기본값 `true`
  - 기존 설치 정황만 있고 `autoOpen` 근거가 없는 partial sync: `false`로 fail-closed
- 마지막 경우에는 default `true`를 last-known-good snapshot으로 기록하지 않는다.
- clean install 기본값, legacy key, schema metadata, local snapshot 복구, update 뒤 key 누락 전체 경로를
  회귀 테스트로 추가했다.

## 검증

- 보강 전 red: 2 failed / 15 passed
- settings-store + lifecycle: 19 passed, 0 failed
- Chrome options + service worker: 41 passed, 0 failed
- shared + Firefox 다운로드 회귀: 76 passed, 0 failed
- Chrome/Firefox dist 계약: 3 passed, 0 failed
- 최신 `upstream/devel` 동기화 후 Chrome/Firefox 빌드: 각각 169 modules transformed, 성공
- source/dist 핵심 모듈 byte 비교: 통과
- Rust/WASM/renderer 변경 없음, 시각 검증 대상 아님

빌드 worktree에는 의존성과 `pkg/`가 없어서 lockfile 기준 npm 의존성을 설치하고 기존 로컬 WASM
산출물을 복사해 패키징 검증을 수행했다. 추적 파일에는 생성물이 남지 않았다.

## 판정

requested-changes의 재현 경로와 메인테이너가 제안한 두 방어 시점을 모두 보강했다. 원격 반영 전
`git diff --check`와 최종 변경 범위를 다시 확인하고, 작업지시자 승인 뒤 commit·push·재검토 요청을
진행한다.
