---
타스크: #196 rhwp-studio HWPX 저장 비활성화 + 사용자 안내
단계: Stage 2 — rhwp-studio + rhwp-chrome 빌드 검증
브랜치: local/task196
작성일: 2026-04-19
선행: Stage 1 완료
---

# Stage 2 단계별 완료 보고서

## 1. 목표

rhwp-studio + rhwp-chrome 두 빌드 산출물 생성 + 검증.

## 2. 빌드 결과

### 2.1 rhwp-studio

```bash
cd rhwp-studio && npm run build
```

```
vite v8.0.8 building client environment for production...
✓ 84 modules transformed.
dist/index.html                       54.93 kB │ gzip:     6.57 kB
dist/assets/rhwp_bg-Jx70G__L.wasm  3,746.13 kB │ gzip: 1,469.37 kB
dist/assets/index-Di8-R0fz.css        59.68 kB │ gzip:    10.68 kB
dist/assets/index-BF90--ci.js        674.85 kB │ gzip:   143.22 kB
✓ built in 481ms
```

### 2.2 rhwp-chrome dist

```bash
cd rhwp-chrome && rm -rf dist && npm run build
```

```
[1/4] Vite 빌드 (rhwp-studio → dist/) ✅
[2/4] 확장 파일 복사 ✅
[3/4] WASM 복사 ✅
[4/4] 폰트 복사 ✅
=== 빌드 완료 ===
```

### 2.3 dist 검증

```
$ ls rhwp-chrome/dist/sw/
context-menus.js
download-interceptor.js     ← #198 블랙리스트 패턴 반영
message-router.js
thumbnail-extractor.js
viewer-launcher.js
                            ← *.test.* 미포함

$ grep -lc "rhwp-toast-container" rhwp-chrome/dist/assets/*.js
viewer-DDt4yWum.js          ← #196 토스트 컴포넌트 반영
```

## 3. 정체성 셀프 체크

- [x] 두 빌드 모두 에러 0
- [x] dist 의 sw 모듈 정상 (test 파일 제외)
- [x] dist 의 viewer 번들에 토스트 컴포넌트 반영
- [x] WASM 변경 없음 (기존 pkg 재사용)

## 4. 다음 단계

Stage 3: 작업지시자 수동 검증.

### 검증 절차 안내

vite dev server 또는 chrome 확장 dist 로드 후:

1. **HWPX 파일 열기** (`samples/hwpx/hwpx-h-01.hwpx` 등)
   - **저장 메뉴 비활성** 확인 (회색)
   - **저장 메뉴 hover 툴팁** 표시 확인
   - **상태 표시줄 메시지** 표시 확인 ("HWPX 베타 모드 — 저장은 다음 업데이트에서 지원됩니다")
   - **우상단 토스트** 1회 표시 + 자동 페이드 + × 버튼 동작 + [자세히] 클릭 시 #197 페이지 열림
   - **Ctrl+S 무반응** 확인

2. **HWP 파일 열기**
   - **저장 메뉴 활성** 확인
   - 토스트/상태바 메시지 없음
   - 정상 저장 동작 (PR #189 의 current-handle 우선 흐름)

## 5. 승인 요청

본 단계 완료 보고서 승인 후 Stage 3 작업지시자 수동 검증 단계 진입.
