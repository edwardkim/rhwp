# Task M100 #5986 완료 보고서

- **Issue**: [#5986](https://github.com/edwardkim/rhwp/issues/5986)
- **기준**: `upstream/devel` `ad2867708`
- **브랜치**: `codex/issue-5986-save-protection`
- **완료일**: 2026-08-24 KST

## 결과

`WasmBridge.loadDocumentAtomically()`가 다음 문서의 `requiresPasswordForSave` boolean을 명시적으로
받도록 바꿨다. 평문 load는 `false`, `loadDocumentWithPassword()` 성공은 `true`를 전달하며, 문서 준비가
끝난 atomic commit 구간에서만 현재 상태를 교체한다. 오답·손상 등 load 실패는 commit 전에 예외가 나므로
기존 문서와 기존 보호 의도를 함께 유지한다.

새 문서와 `releaseDocument()`의 `false` 초기화는 유지했다. 또한 Save As fallback download가 실패하기
전에 보호 상태를 바꾸던 순서를 고쳐, download 시작이 성공한 뒤에만 파일명과 보호 의도를 commit한다.

암호 문자열을 보관하는 필드는 추가하지 않았다. 장기 상태는 기존 boolean 하나뿐이며 암호 입력은 각
열기·저장 시도의 지역 변수 범위를 벗어나지 않는다.

## 회귀 테스트

- HWP3 실제 암호 fixture: 성공 시 보호 의도 `true`, 취소·오답 시 이전 상태 유지
- HWP5 EncryptVersion 4 실제 fixture: 같은 계약
- HWPX ODF AES-256-CBC 실제 fixture: 같은 계약
- 평문 HWPX load, 새 문서, document release: 보호 의도 `false`
- 보호 문서에서 평문 Save As를 시도해 fallback download가 실패: 기존 보호 의도 `true`, dirty 유지
- 암호 문자열의 local/session storage 비보존 유지

## 검증 결과

- focused Node 계약 테스트: 11/11 통과
- `npm test`: 1,071 통과, 1 skip, 실패 0
- fresh WASM binding 생성: locked wrapper `--no-opt` 통과
- `npm run build`: TypeScript + Vite production build 통과
- `npm run e2e:hwp-password-open`: HWP3/HWP5/HWPX와 상태 수명주기 전 항목 통과
- `npm run e2e:issue-4430-content-loss`: 저장 성공·실패·암호·fallback 전 항목 통과
- `node --check`와 `git diff --check`: 통과

`python3 scripts/check_e2e_manifest.py`는 변경 전부터 존재하던 미등재 파일
`loading-busy-cursor.test.mjs`, `status-page-number.test.mjs`, `toolbox-visibility.test.mjs` 3건 때문에
실패했다. 이번에 변경한 두 E2E 파일의 manifest 행은 함께 갱신했으며, 기존 기준선 3건은 범위 밖으로
남겼다.

## 계획 대비 실제

| 계획 | 실제 | 판정 |
|---|---|---|
| atomic load에 boolean 보호 의도 전달 | 평문 `false`, 암호 성공 `true`를 commit 구간에서만 반영 | 계획대로 |
| 새 문서·release는 `false` | 기존 초기화 계약 유지 및 회귀 검증 | 계획대로 |
| load·저장 실패 시 기존 상태 유지 | load commit 전 예외와 download 성공 뒤 fallback commit으로 보장 | 계획대로 |
| HWP3/HWP5/HWPX 실제 fixture 검증 | 세 형식 성공·취소·오답과 수명주기 E2E 통과 | 계획대로 |
| 기존 검증 하네스 사용 | dialog 범위, onboarding, 두 번째 파일명 확인을 추가 안정화 | 계획 외 추가 |
| E2E manifest 검사 통과 | 기존 미등재 3건 때문에 실패, 변경한 행만 현행화 | 미완료/예외 미승인 |

## 하이퍼 워터폴 절차 감사와 보정

원 작업은 구현 요청 뒤 계획을 작성했으나 계획서 작성 뒤의 별도 사용자 승인을 받지 않았다. 또한 계획,
구현, 테스트, 완료 보고를 `bdc90ded9` 한 커밋에 묶었고 contemporaneous working stage report를 남기지
않았다. 따라서 기능 결과가 검증됐더라도 원 작업을 하이퍼 워터폴 완전 준수로 소급 판정하지 않는다.

2026-08-24 사용자에게 이 이탈을 보고한 뒤 현재 시점의 보정 승인을 받았다. 그 승인에 따라 다음을 별도
후속 문서 커밋으로 보완한다.

- 오늘할일에 #5986 수행·검증·절차 상태 추가
- 사후 설계임을 명시한 `task_m100_5986_impl.md` 추가
- 증거 기반 Stage 1~3 사후 재구성 보고 추가
- 이 보고서에 계획 대비 실제, 하네스 추가 작업, 미승인 manifest 예외 기록

이 보정은 검토 연속성과 감사 가능성을 회복하지만 누락된 사전 승인이나 단계별 Git 이력을 만들어내지는
않는다. 원격 push·PR 생성은 별도 사용자 승인과 manifest 기준선 처리 또는 명시적 예외 승인 전까지
진행하지 않는다.
