# PR #2635: recovery-ui HML 문서 테스트 케이스 추가

## 이슈
- **Issue**: #2634 — recovery-ui 테스트에 HML 문서 케이스 누락

## 분석

recovery-ui.test.ts는 recoveryFileName(), describeDraft()를 HWP/HWPX에
대해서만 검증하고 HML 문서는 테스트하지 않았다.

recovery-format.ts의 baseNameWithoutKnownExtension()과 describeDraft()에도
HML 지원이 누락되어 있어, 테스트 추가와 함께 수정이 필요했다.

## 변경

### recovery-format.ts
1. `baseNameWithoutKnownExtension()`에 `.hml` 확장자 인식 추가
2. `describeDraft()`에서 HML 출처도 HWP 복구본임을 표시하도록 조건 확장

### recovery-ui.test.ts
1. `recoveryFileName('sample.hml')` → `'sample 복구본.hwp'` 검증
2. `describeDraft` HML sourceFormat 전달 시 `HML → HWP 복구본` 표시 검증

## 결과
- `node --test tests/recovery-ui.test.ts` → 5/5 pass
- Closes #2634
