# PR #2629: recovery-ui HML 문서 복구 시 포맷 표시 및 기본 파일명 개선

## 이슈
- **Issue**: #2628 — HML 문서 복구 다이얼로그에서 포맷 표시 누락

## 분석

### 문제 1: 기본 파일명 고정

`recovery-ui.ts:61`에서 `draft.fileName`이 없으면 항상 `'문서.hwp'`로
표시된다. HML 문서의 복구 draft는 fileName이 없을 때 `문서.hml`로
표시되어야 한다.

### 문제 2: describeDraft HML 포맷 미표시

`recovery-format.ts:37`에서 `sourceFormat === 'hwpx'`만 검사하여
HML 출처의 복구본임을 표시하지 않았다. autosave는 항상 exportHwp()
결과를 저장하므로 HML 출처도 HWP 복구본임을 명시해야 한다.

## 변경

### recovery-ui.ts
```typescript
// before
title.textContent = draft.fileName || '문서.hwp';
// after — sourceFormat에 따라 기본 파일명 동적 생성
const displayName = draft.fileName || (() => {
  switch (draft.sourceFormat?.toLowerCase()) {
    case 'hml': return '문서.hml';
    case 'hwpx': return '문서.hwpx';
    default: return '문서.hwp';
  }
})();
```

### recovery-format.ts
```typescript
// before
const suffix = draft.sourceFormat.toLowerCase() === 'hwpx' ? ' → HWP 복구본' : '';
// after — HML도 exportHwp 복구 대상
const suffix = ['hwpx', 'hml'].includes(draft.sourceFormat.toLowerCase()) ? ' → HWP 복구본' : '';
```

## 검증

- HWP 문서: `'문서.hwp'` + 접미사 없음 (기존과 동일)
- HWPX 문서: `'문서.hwpx'` + `' → HWP 복구본'` (기존과 동일)
- HML 문서: `'문서.hml'` + `' → HWP 복구본'` (신규)
- describeDraft에서 HML 출처도 올바르게 표시됨
