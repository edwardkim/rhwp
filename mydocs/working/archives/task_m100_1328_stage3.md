# Task M100-1328 Stage 3 완료 보고서 — 원본 글꼴명 보존과 표시용 font-family chain

- 이슈: https://github.com/edwardkim/rhwp/issues/1328
- 수행 계획서: `mydocs/plans/task_m100_1328.md`
- 구현 계획서: `mydocs/plans/task_m100_1328_impl.md`
- 작성일: 2026-06-21
- 브랜치: `codex/1328-local-font-consent`
- 기준 커밋: `1ce7b79d7466`

## 1. 완료 범위

Stage 3 범위인 표시/측정 단계의 font-family chain 정리를 구현했다.

변경 파일:

- `rhwp-studio/src/core/font-substitution.ts`
- `rhwp-studio/src/core/wasm-bridge.ts`
- `rhwp-studio/tests/font-substitution.test.ts`

## 2. 주요 변경

### 2.1 표시용 font-family chain helper 추가

`fontFamilyChainForDisplay()`를 추가했다.

반환 순서:

1. 문서 원본 글꼴명
2. `resolveFont()` 결과인 rhwp 웹 대체 글꼴명
3. OS/system fallback
4. generic fallback

예:

```text
"휴먼명조", "HY신명조", "Batang", "AppleMyungjo", "Noto Serif KR", serif
```

이 방식은 로컬 글꼴 목록을 새로 읽지 않는다. 브라우저가 이미 알고 있는 CSS font-family 해소 규칙에 따라
원본 글꼴이 설치되어 있으면 먼저 사용하고, 없으면 웹 대체 글꼴과 system fallback으로 내려간다.

### 2.2 WASM 측정 경로 반영

`wasm-bridge.ts`의 `substituteCssFontFamily()`가 더 이상 원본 글꼴명을 웹 대체 글꼴명 하나로 치환하지 않는다.

기존:

```text
"휴먼명조" → "HY신명조", "Batang", ...
```

변경 후:

```text
"휴먼명조" → "휴먼명조", "HY신명조", "Batang", ...
```

Canvas 렌더링 경로는 이미 원본 글꼴명을 포함한 font-family를 사용하고 있었으므로,
이번 Stage에서는 레이아웃/측정 경로도 원본 글꼴 우선 순서와 맞췄다.

### 2.3 기존 resolveFont 동작 유지

`resolveFont()`의 치환 테이블 동작은 유지했다. 새 helper는 기존 치환 결과를 chain의 두 번째 후보로 사용한다.

## 3. 테스트 보강

`rhwp-studio/tests/font-substitution.test.ts`를 추가했다.

검증 항목:

1. `resolveFont()`가 기존 웹 대체 글꼴 해소를 유지한다.
2. `fontFamilyChainForDisplay()`가 원본 글꼴을 웹 대체 글꼴보다 앞에 둔다.
3. 등록 웹폰트에도 system fallback을 붙인다.
4. generic font는 그대로 처리한다.
5. 기존 `fontFamilyWithFallback()` helper의 fallback 계열을 유지한다.

## 4. 검증 결과

통과:

```bash
cd rhwp-studio && node --test tests/font-substitution.test.ts tests/document-font-status.test.ts tests/local-fonts.test.ts
cd rhwp-studio && npm test
cd rhwp-studio && npm run build
git diff --check
```

결과 요약:

- 관련 직접 테스트: 15개 통과
- `npm test`: 99개 통과
- `npm run build`: `tsc && vite build` 통과
- `git diff --check`: 출력 없음

## 5. 남은 작업

다음 Stage 4에서 진행할 항목:

- `local-fonts-changed` 이벤트를 toolbar/options dialog 갱신 경로에 연결
- 감지 완료 직후 로컬 글꼴 optgroup 갱신
- canvas reload와 toolbar refresh를 명확한 단일 경로로 정리

## 6. 승인 요청

Stage 3은 완료되었다. 승인 후 Stage 4 감지 완료 이벤트와 UI 갱신 구현으로 진행한다.
