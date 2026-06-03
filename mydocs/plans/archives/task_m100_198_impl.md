---
타스크: #198 rhwp-chrome 다운로드 인터셉터 부작용 수정
브랜치: local/task198
작성일: 2026-04-19
선행: mydocs/plans/task_m100_198.md (수행계획서, 승인됨)
---

# 구현계획서: rhwp-chrome 다운로드 인터셉터 부작용 수정

## 0. 작업지시자 결정 사항 (수행계획서 §11)

| 질문 | 결정 |
|---|---|
| Q1. 단계 분할 | **3단계 유지** (수정+테스트 / 빌드 / 작업지시자 검증) |
| Q2. 단위 테스트 범위 | **분기 로직을 순수 함수 `shouldInterceptDownload(item)` 로 추출 후 그것만 테스트**. Chrome API mock 미작성 |
| Q3. DEXT5 fetch + Blob 케이스 | **본 타스크 범위 외** (별도 이슈 후속) |

## 1. 핵심 설계

### 1.1 분기 로직 추출 — `shouldInterceptDownload(item)`

리스너 안의 hwp 감지 로직을 **순수 함수** 로 추출하여 단위 테스트 가능하게 만든다.

```javascript
// rhwp-chrome/sw/download-interceptor.js
const HWP_EXTENSION_RE = /\.(hwp|hwpx)(\?|$)/i;
const HWP_MIME_HINTS = ['haansoft', 'x-hwp', 'hwp+zip'];

/**
 * 다운로드 항목이 HWP/HWPX 인지 판별.
 * filename / url / finalUrl / mime 어느 하나라도 매치되면 true.
 *
 * @param {chrome.downloads.DownloadItem} item
 * @returns {boolean}
 */
export function shouldInterceptDownload(item) {
  const filename = item.filename || '';
  if (HWP_EXTENSION_RE.test(filename)) return true;

  const url = item.url || '';
  if (HWP_EXTENSION_RE.test(url)) return true;

  const finalUrl = item.finalUrl || '';
  if (finalUrl !== url && HWP_EXTENSION_RE.test(finalUrl)) return true;

  const mime = (item.mime || '').toLowerCase();
  if (HWP_MIME_HINTS.some(hint => mime.includes(hint))) return true;

  return false;
}
```

### 1.2 리스너의 분기 명확화

```javascript
export function setupDownloadInterceptor() {
  chrome.downloads.onDeterminingFilename.addListener((item, suggest) => {
    if (shouldInterceptDownload(item)) {
      handleHwpDownload(item);
      // suggest 호출은 hwp 인 경우에만 — 일반 파일은 Chrome 기본 동작 유지
      suggest({ filename: item.filename });
    }
    // hwp 가 아니면 suggest 호출 자체를 안 함 → 마지막 위치 기억 정상 동작
  });
}
```

### 1.3 변경 영향 영역

| 파일 | 변경 |
|---|---|
| `rhwp-chrome/sw/download-interceptor.js` | `shouldInterceptDownload` 추출 + 리스너 분기 명확화 |
| `rhwp-chrome/sw/download-interceptor.test.js` (신규) | 순수 함수 단위 테스트 |
| `rhwp-chrome/dist/` | 빌드 산출물 갱신 |

`viewer-launcher.js`, `manifest.json`, 다른 sw 모듈 — **변경 없음**.

## 2. 단계 분할 (3 Stage)

### Stage 1 — 인터셉터 수정 + 단위 테스트

**변경**:
- `download-interceptor.js`:
  - `HWP_EXTENSION_RE` 정규식 + `HWP_MIME_HINTS` 상수 추가
  - `shouldInterceptDownload(item)` 순수 함수 export
  - 리스너에서 `if (shouldInterceptDownload) { handleHwpDownload + suggest }` 분기

**신규 파일**:
- `rhwp-chrome/sw/download-interceptor.test.js`:
  - Node `--test` 러너 (PR #189 가 동일 패턴 사용 — `node --experimental-strip-types --test`)
  - 테스트 케이스:
    - `hwp 파일명 감지` (`item.filename = 'sample.hwp'` → true)
    - `hwpx 파일명 감지` (`item.filename = 'sample.hwpx'` → true)
    - `대소문자 무관 감지` (`'SAMPLE.HWP'` → true)
    - `URL 에서 hwp 감지` (`item.url = 'https://x/y.hwp'` → true)
    - `URL 쿼리 무시` (`item.url = 'https://x/y.hwp?token=abc'` → true)
    - `finalUrl 감지` (redirect 후 hwp 확장자)
    - `mime 감지 (haansoft)` (`item.mime = 'application/haansoft-hwp'` → true)
    - `mime 감지 (x-hwp)` (`item.mime = 'application/x-hwp'` → true)
    - `일반 이미지 미감지` (`item.filename = 'image.png'`, `mime = 'image/png'` → false)
    - `일반 PDF 미감지` (`item.filename = 'doc.pdf'` → false)
    - `빈 item 미감지` (`{}` → false)

**완료 기준**:
- `node --test rhwp-chrome/sw/download-interceptor.test.js` 그린
- `download-interceptor.js` ESM export 정상 (기존 `setupDownloadInterceptor` import 보존)

### Stage 2 — 빌드 + dist 갱신

**작업**:
- `cd rhwp-chrome && npm run build` 실행
- `rhwp-chrome/dist/` 갱신 확인:
  - `background.js` (빌드 시 sw 모듈 번들링) 또는 sw/ 복사 결과 확인
  - manifest.json 변경 없음 확인

**완료 기준**:
- `dist/` 빌드 산출물 정상 생성
- 빌드 로그에 에러 0건

### Stage 3 — 작업지시자 수동 검증 + 보고서 + 커밋

**검증 절차** (작업지시자):
1. `chrome://extensions/` → 개발자 모드 → "압축 해제된 확장" → `rhwp-chrome/dist/` 로드
2. 일반 이미지/PDF 다운로드 → "다른 이름으로 저장" 시 마지막 위치 기억 확인
3. 일반 .hwp 파일 다운로드 → 자체 뷰어 정상 트리거
4. (가능하면) DEXT5 류 관공서 사이트 → hwp 감지

**문서**:
- `mydocs/working/task_m100_198_stage[1..3].md`
- `mydocs/report/task_m100_198_report.md`
- 오늘할일 (`mydocs/orders/20260419.md`) 갱신

**완료 기준**: 작업지시자 검증 통과 + 보고서 + merge 준비.

## 3. 파일 변경 요약

| Stage | 신규 파일 | 수정 파일 |
|---|---|---|
| 1 | `rhwp-chrome/sw/download-interceptor.test.js` | `rhwp-chrome/sw/download-interceptor.js` |
| 2 | — | `rhwp-chrome/dist/*` (빌드 산출물) |
| 3 | `mydocs/working/task_m100_198_stage[1..3].md`, `mydocs/report/task_m100_198_report.md` | `mydocs/orders/20260419.md` |

## 4. 위험 요소 (수행계획서 §6 보강)

| 위험 | 단계 | 완화 |
|---|---|---|
| ESM `export` 추가가 기존 background.js 동적 import 와 충돌 | Stage 1 | 빌드 후 dist 의 background.js 동작 확인 |
| `node --test` 러너가 환경에 없음 | Stage 1 | Node 20+ 표준 기능, PR #189 가 동일 패턴 사용 — 환경 가정 가능 |
| `item.mime` 이 빈 문자열인 경우 | Stage 1 | `(item.mime \|\| '').toLowerCase()` 안전 처리 |
| HWP_MIME_HINTS 가 false positive 유발 | Stage 1 | 보수적 list (`haansoft`, `x-hwp`, `hwp+zip` 만) |
| dist 빌드 실패 (Vite 또는 build.mjs 오류) | Stage 2 | 빌드 사전 점검 (`npm run build`) |
| 작업지시자 환경 (Windows Chrome 147) 과 다른 OS 동작 차이 | Stage 3 | chrome-fd-001 보고자 환경이 우선 검증 게이트 |

## 5. 검증 방법

- **단위 테스트**: `node --test rhwp-chrome/sw/download-interceptor.test.js`
- **빌드**: `cd rhwp-chrome && npm run build`
- **수동 (작업지시자, Stage 3)**: §2 Stage 3 검증 절차

## 6. 일정 합계

- Stage 1: 0.3일
- Stage 2: 0.1일
- Stage 3: 작업지시자 검증 사이클 의존 (0.5일 ~ 1일)
- **총: 1~2일**

## 7. 정체성 셀프 체크

- [x] 최소 침습 — 단일 sw 모듈 + 신규 테스트 파일 1개. 다른 sw / manifest 변경 없음
- [x] 분기 명확화 — `if HWP { suggest } else { skip }` 명시적
- [x] HWP 감지 다중화 — filename/URL/finalUrl/mime
- [x] 회귀 우선 — HWP 처리 동작 (`handleHwpDownload`) 변경 없음
- [x] 트러블슈팅 사전 검색 (메모리 적용) — 본 영역 0건 확인
- [x] 단위 테스트로 분기 검증 + 작업지시자 수동 검증으로 통합 게이트

## 8. 승인 요청

본 구현계획서 승인 후 Stage 1 착수.
