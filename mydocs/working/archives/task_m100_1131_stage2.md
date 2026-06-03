# 단계 2 완료보고서 — task_m100_1131_stage2

- **이슈**: edwardkim/rhwp#1131
- **브랜치**: `feature/1131-file-url-access-guidance`
- **단계**: 2/4 — `loadFromUrlParam` 실패 분기 연결
- **작성일**: 2026-05-26

## 변경 내용

**`rhwp-studio/src/main.ts`** `loadFromUrlParam` 최종 `catch`:

```ts
} catch (error) {
  // 로컬 file:// 로드 실패 + "파일 URL 액세스 허용" 미허용 → 전용 안내 (#1131)
  if (fileUrl.startsWith('file:') && typeof chrome !== 'undefined') {
    const allowed = await isFileSchemeAccessAllowed();
    if (allowed === false) {
      showFileUrlAccessGuidance();
      return;
    }
  }
  showLoadError(error);
}
```

- `file://` + 확장 환경 + 권한 **미허용(false)** 일 때만 안내 토스트.
- 권한 허용 / 판정 불가(null) / 원격 URL → 기존 `showLoadError` 유지(회귀 차단).

## 검증

- `npx tsc --noEmit`: 본 변경 관련 타입 에러 없음. 남은 2건은 WASM `pkg/` 미빌드(기존, 무관).

## 다음 단계

단계 3 — `rhwp-chrome/sw/download-interceptor.js`에서 `file://` HWP 중복 다운로드 억제.
