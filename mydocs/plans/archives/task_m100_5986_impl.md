# 구현 계획 — Task M100 #5986

- **이슈**: [#5986](https://github.com/edwardkim/rhwp/issues/5986)
- **기준 commit**: `upstream/devel` `ad2867708`
- **구현 commit**: `bdc90ded9`
- **문서 성격**: 2026-08-24 감사 뒤 작성한 사후 설계 정리

이 문서는 구현 전에 승인받은 계획서가 아니다. 당시 실제 변경과 검증 증거를 파일 단위로 재구성해 후속
검토와 재현에 필요한 설계를 보존한다. 따라서 누락된 사전 승인 게이트를 소급 충족시키지는 않는다.

## 상태 계약

`requiresPasswordForSave`는 암호 자체가 아니라 현재 문서를 다음에 저장할 때 암호 입력을 요구해야 한다는
boolean 보호 의도다.

| 사건 | 성공 뒤 상태 | 실패 시 상태 |
|---|---:|---|
| 평문 문서 load | `false` | 기존 문서·보호 의도 유지 |
| 암호 문서 load | `true` | 기존 문서·보호 의도 유지 |
| 새 문서 | `false` | 해당 없음 |
| document release | `false` | 해당 없음 |
| 평문 Save As fallback | download 성공 뒤 `false` | 기존 보호 의도와 dirty 유지 |

## 파일별 구현

### `rhwp-studio/src/core/wasm-bridge.ts`

- `loadDocumentAtomically()`에 성공한 문서의 보호 의도를 명시적 인자로 전달한다.
- 평문 load는 `false`, `loadDocumentWithPassword()`는 `true`를 전달한다.
- 파싱·초기화·문서 준비가 모두 끝난 atomic commit 구간에서만 현재 상태를 교체한다.
- 오답·손상·초기화 실패 경로에는 보호 의도를 쓰지 않는다.

### `rhwp-studio/src/command/commands/file.ts`

- Save As fallback에서 파일명과 보호 의도 갱신을 download 성공 뒤로 옮긴다.
- download 실패는 기존 보호 의도와 dirty 상태를 바꾸지 않는다.

### 계약 테스트와 E2E

- `rhwp-studio/tests/hwp-password-open.test.ts`: 평문/암호 load의 전달값과 실패 시 commit 부재를 고정한다.
- `rhwp-studio/tests/hwp-password-save.test.ts`: fallback download 실패의 보호 상태 보존을 고정한다.
- `rhwp-studio/e2e/hwp-password-open.test.mjs`: 실제 HWP3/HWP5/HWPX 암호 fixture와 새 문서·release 수명주기를
  검증한다.
- `rhwp-studio/e2e/content-loss-save-issue4430.test.mjs`: 보호 문서의 평문 Save As 실패 회귀를 검증한다.
- `rhwp-studio/e2e/MANIFEST.md`: 변경한 E2E의 계약 설명을 현행화한다.

## 보안 경계

- 암호 문자열을 필드·응답·로그·DOM·URL·storage에 새로 보존하지 않는다.
- 상태로 유지하는 값은 boolean 보호 의도 하나뿐이다.
- serializer·암호화 알고리즘·embed transport는 변경하지 않는다.

## 검증 게이트

1. focused Node 계약 테스트
2. Studio 전체 unit test
3. fresh locked WASM wrapper 생성과 production build
4. 암호 열기 및 content-loss 저장 E2E
5. E2E manifest 검사와 변경 범위 판정
6. JavaScript syntax 및 Git whitespace 검사

실제 결과와 계획 대비 차이는 `mydocs/report/task_m100_5986_report.md`에 기록한다.
