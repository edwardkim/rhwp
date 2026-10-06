---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_6534_impl.md
issue: 6534
stage: 2
date: 2026-08-31
---

# Task M100 #6534 Stage 2 — 다운로드 결정·adapter 정정 결과

## 1. 구현 결과

공유 다운로드 판정을 기존 양성 신호 OR boolean에서 근거 우선순위를 가진 3상태 결정으로 바꿨다.

```text
non-refetchable
  > explicit HWP filename
  > explicit non-HWP filename
  > concrete non-HWP MIME
  > provisional/final HWP URL·finalUrl·MIME evidence
```

`classifyDownload()`은 `intercept`, `defer`, `ignore`와 개인 정보가 없는 닫힌 `reason`을 반환한다.
기존 `shouldInterceptDownload()`은 현재 item을 최종 메타데이터로 간주하는 boolean wrapper로 남겼다.
Chrome/Firefox observer만 실제 이벤트 단계 context를 새 함수에 전달한다.

## 2. 이벤트별 동작

| 이벤트와 근거 | 결과 |
| --- | --- |
| `onCreated`, filename `.hwp/.hwpx/.hml` | 즉시 `intercept` |
| `onCreated`, URL/MIME만 HWP | 상태 추적 후 `defer` |
| `onChanged`, filename `.xlsx` 등 확정 | `ignore` |
| `onChanged`, filename 확정 또는 complete + HWP 보조 근거 | `intercept` |
| DEXT5 URL/referrer | 다른 근거와 무관하게 `ignore` |

비-HWP filename allowlist는 Office 문서군, PDF, ZIP, ODF에 한정했다. `.bin`과
`application/octet-stream`은 공공기관 extensionless HWP 호환을 위해 중립으로 유지했다.

Chrome과 Firefox 모두 `delta.filename.current` 또는 terminal state가 있을 때만
`metadataFinalized: true`를 넘긴다. `finalUrl`만 먼저 도착하면 계속 보류한다. observer state,
terminal cleanup, settings, Chrome in-flight 선점과 `file://` cancel/erase에는 손대지 않았다.

## 3. 검증 결과

### focused

| 검사 | 결과 |
| --- | ---: |
| 공유 다운로드 결정 | 32/32 PASS |
| 공유 observer state | 14/14 PASS |
| Chrome adapter | 20/20 PASS |
| Firefox adapter | 14/14 PASS |

Stage 1에서 실패했던 공유 6건, Chrome 3건, Firefox 3건이 모두 green으로 전환됐다.

### CI service-worker 묶음

```bash
node --test rhwp-shared/sw/*.test.js rhwp-chrome/sw/*.test.mjs rhwp-firefox/sw/*.test.mjs
```

- 131/131 PASS
- 설정 저장·복구와 자동 동작 fail-closed
- fetch URL 보안
- Chrome extension lifecycle
- GitHub document URL 분류
- bounded thumbnail stream
- 다운로드 신선도·중복·restart·terminal 보호계약

### 다음 단계 red 보존

Stage 2가 ZIP/HWPX 책임까지 섞어 고치지 않았는지 다시 확인했다.

- Studio: 15 PASS, Stage 3 대상 2 red (`hwpx !== zip`)
- Rust: Stage 3 대상 2 red (`Hwpx != Unknown`)

따라서 다운로드 의도 판정만 green이고, HWPX 구조 판정은 계획대로 Stage 3에 남아 있다.

## 4. 변경 파일과 비변경 불변식

### 제품 변경

- `rhwp-shared/sw/download-interceptor-common.js`
- `rhwp-chrome/sw/download-interceptor.js`
- `rhwp-firefox/sw/download-interceptor.js`

### 변경하지 않음

- `rhwp-shared/sw/download-observer-state.js`
- settings store와 browser manifest
- Studio/Rust 제품 구현
- dist·generated artifact

`git diff --check`도 통과했다.

## 5. Stage 2 판정과 다음 게이트

Stage 2는 **qualified-download-decision**이다.

- #6534 XLSX 충돌은 공유 정책과 Chrome/Firefox adapter에서 탭 0 계약으로 전환됐다.
- 확정 HWP 즉시 열기와 extensionless HWP terminal fallback은 유지됐다.
- 기존 #198, #1498/#1515, #2656 보호계약도 유지됐다.

다음 단계는 Stage 3이다. Rust가 ZIP 중앙 디렉터리의 두 HWPX 필수 엔트리를 확인해 최종 포맷을
소유하도록 하고, Studio는 ZIP을 `zip` 후보로만 부른다. Stage 2 checkpoint 뒤 별도 승인을 받기 전에는
해당 제품 코드를 수정하지 않는다.
