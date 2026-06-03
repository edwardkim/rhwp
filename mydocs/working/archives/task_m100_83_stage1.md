# Task #83 — 1단계 완료보고서

## Xcode 빌드 + 기본 동작 확인 ✅

### 작업 내용

1. **Chrome 확장 빌드**: `rhwp-chrome/` → `npm run build` → `dist/` 생성
2. **Safari 전용 dist 생성**: `rhwp-safari/dist/`에 Chrome dist 복사 후 Safari 호환 수정
   - `background.js`: ES module → 단일 파일 번들링 (import/export 제거)
   - `manifest.json`: `service_worker` + `type: "module"` → `scripts` + `persistent: false`
   - `downloads` 권한 제거 (Safari 미지원)
3. **Safari Web Extension 변환**: `safari-web-extension-converter` → Xcode 프로젝트
4. **macOS 빌드**: BUILD SUCCEEDED
5. **iOS Simulator 빌드**: BUILD SUCCEEDED
6. **Safari 등록**: 확장 아이콘 + 팝업 정상 표시, 백그라운드 로드 오류 해결

### 해결한 문제

| 문제 | 원인 | 해결 |
|------|------|------|
| 백그라운드 콘텐츠 로드 실패 | ES module import 미지원 | 단일 파일 번들링 |
| 백그라운드 로드 실패 (2차) | `service_worker` 형식 미지원 | `scripts` 형식으로 변경 |
| Chrome dist 오염 방지 | 동일 dist 공유 | Safari 전용 `rhwp-safari/dist/` 분리 |

### 파일 구조

```
rhwp-safari/
├── dist/                    — Safari 전용 빌드 (Chrome dist 기반 + 호환 수정)
│   ├── background.js        — 번들링된 단일 파일
│   ├── manifest.json        — scripts 형식
│   ├── content-script.js
│   ├── viewer.html
│   ├── wasm/
│   └── fonts/
└── HWP Viewer/              — Xcode 프로젝트
    ├── HWP Viewer.xcodeproj
    ├── Shared (App)/
    ├── Shared (Extension)/
    ├── macOS (App)/
    ├── macOS (Extension)/
    ├── iOS (App)/
    └── iOS (Extension)/
```

### 미해결 (2단계)

- HWP 링크 클릭 시 뷰어 미동작 (content-script + API 호환성)
