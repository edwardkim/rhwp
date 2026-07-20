# PR #2632: recent-store HML 문서 테스트 케이스 추가

## 이슈
- **Issue**: #2631 — recent-store 테스트에 HML 문서 케이스 누락

## 분석

`rhwp-studio/tests/recent-store.test.ts`는 HWP와 HWPX 문서에 대한
recent store 동작만 검증하고 HML 문서는 전혀 테스트하지 않았다.

HML 포맷 지원이 추가됨에 따라 recent store가 HML 문서를 올바르게
처리하는지 검증할 필요가 있다.

## 변경

1. **메타-only 기록 테스트에 HML 추가**: `{fileName: '문서.hml', sourceFormat: 'hml'}` 추가
2. **HML 문서 전용 테스트 추가**: `addRecentDoc` → `listRecentDocs` → 핸들 보존 확인
3. **최대 8개 상한 테스트 혼합**: HML/HWPX/HWP를 순환하도록 fixture 다양화

## 결과
- `node --test tests/recent-store.test.ts` → 8/8 pass
- Closes #2631
