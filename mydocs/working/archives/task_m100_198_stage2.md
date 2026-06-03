---
타스크: #198 rhwp-chrome 다운로드 인터셉터 부작용 수정
단계: Stage 2 — dist 빌드
브랜치: local/task198
작성일: 2026-04-19
선행: Stage 1 완료
---

# Stage 2 단계별 완료 보고서

## 1. 목표

`rhwp-chrome/dist/` 갱신. 작업지시자가 Chrome 에 로드 가능한 압축 해제된 확장 산출물 준비.

## 2. 산출물

### 2.1 빌드 절차

```bash
cd rhwp-chrome
npm run build       # node build.mjs
```

### 2.2 빌드 결과

```
[1/4] Vite 빌드 (rhwp-studio → dist/) ✅
[2/4] 확장 파일 복사 ✅ (manifest, background, sw/, content-script, options 등)
[3/4] WASM 복사 ✅ (pkg/rhwp.* → dist/wasm/)
[4/4] 폰트 복사 ✅ (필수 폰트 14개)

=== 빌드 완료 ===
출력: /home/edward/mygithub/rhwp/rhwp-chrome/dist
```

### 2.3 발견 + 추가 수정 — 테스트 파일 dist 포함 방지

초회 빌드 시 `dist/sw/download-interceptor.test.js` 가 함께 복사되는 것을 발견.
배포본에 테스트 파일이 들어가면 안 되므로 [build.mjs](rhwp-chrome/build.mjs) 갱신:

```javascript
const EXCLUDE_FROM_DIST = /\.(test|spec)\.[mc]?[jt]sx?$/i;

copy(resolve(__dirname, 'sw'), resolve(DIST, 'sw'), {
  filter: (src) => !EXCLUDE_FROM_DIST.test(src),
});
```

`cpSync` 의 `filter` 옵션 활용. 향후 sw 모듈에 추가되는 `*.test.*` / `*.spec.*` 도 자동 제외.

### 2.4 dist 검증

```
$ ls rhwp-chrome/dist/sw/
context-menus.js
download-interceptor.js     ← 수정 반영 확인
message-router.js
thumbnail-extractor.js
viewer-launcher.js
                            ← download-interceptor.test.js 미포함 ✅

$ grep -c "shouldInterceptDownload" rhwp-chrome/dist/sw/download-interceptor.js
2                           ← export + 호출 확인 ✅
```

## 3. 정체성 셀프 체크

- [x] 빌드 성공 (에러 0건)
- [x] 수정된 인터셉터 dist 반영
- [x] 테스트 파일 dist 미포함 (배포본 정합성)
- [x] manifest / 다른 sw 모듈 변경 없음 (회귀 0)

## 4. 다음 단계

Stage 3: 작업지시자 수동 검증.

### 검증 절차 안내

1. Chrome 에서 `chrome://extensions/` 접속 → 개발자 모드 ON
2. (기존 rhwp-chrome 확장이 설치돼 있으면 비활성 또는 제거)
3. **"압축 해제된 확장 프로그램 로드"** → `/home/edward/mygithub/rhwp/rhwp-chrome/dist` 선택
4. 검증 케이스:
   - **케이스 A — 일반 이미지/PDF 저장 위치 기억**: 임의 사이트에서 이미지 우클릭 → "다른 이름으로 저장" → 위치 X 선택 → 저장 → 다시 다른 이미지 저장 시 위치 X 가 기본값으로 표시되는지 확인 (chrome-fd-001 핵심)
   - **케이스 B — HWP 자체 뷰어 트리거**: hwp 파일 다운로드 가능한 사이트에서 .hwp 다운로드 → rhwp 자체 뷰어 탭 자동 열림 확인
   - **케이스 C (선택, DEXT5)**: 관공서 사이트 (예: 행정안전부, 국세청) 에서 .hwp 다운로드 → URL 이 토큰 기반이어도 rhwp 뷰어 트리거되는지 확인

## 5. 승인 요청

본 단계 완료 보고서 승인 후 Stage 3 작업지시자 수동 검증 단계 진입.
