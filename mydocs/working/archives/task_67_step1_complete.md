# Task #67 — 1단계 완료보고서

## 저작권 폰트 제거 + 오픈소스 폰트 도입 ✅

### 수정 파일

- `web/fonts/` — 오픈소스 woff2 17개 추가
- `web/fonts/FONTS.md` — 카테고리별 분류, 라이선스/출처/대체 대상 명시
- `rhwp-studio/src/core/font-loader.ts` — FONT_LIST 전면 개편

### 변경 내용

#### 1. 오픈소스 폰트 추가 (17개 woff2)

| 카테고리 | 폰트 | 파일 수 | 라이선스 |
|----------|------|---------|---------|
| Serif | Noto Serif KR (R/B), 나눔명조 (R/B/EB), 고운바탕 (R/B) | 7 | SIL OFL |
| Sans-serif | Noto Sans KR (R/B), 나눔고딕 (R/B/EB), 고운돋움 (R) | 6 | SIL OFL |
| Monospace | D2 Coding (R/B), 나눔고딕코딩 (R/B) | 4 | SIL OFL |

#### 2. 폰트 매핑 변경 (font-loader.ts)

| 기존 (저작권 폰트) | 변경 후 |
|-------------------|---------|
| hamchob-r.woff2 (로컬) | jsdelivr CDN woff (함초롬바탕) |
| hamchod-r.woff2 (로컬) | jsdelivr CDN woff (함초롬돋움) |
| HY 폰트 → hamcho* | HY 폰트 → Noto Sans/Serif KR |
| MS 폰트 (Arial, Calibri 등) | 제거 (OS 폰트 폴백) |
| 맑은 고딕 → MalgunGothic.woff2 | 맑은 고딕 → Pretendard |
| 바탕/돋움/굴림 → hamcho* | 바탕 → Noto Serif, 돋움/굴림 → Noto Sans |
| 굴림체/바탕체 → hamcho* | 굴림체/바탕체 → D2 Coding |
| 궁서 → hamchob-r | 궁서 → 고운바탕 |

#### 3. FontEntry 구조 개선

- `format` 필드 추가 (woff/woff2 구분)
- @font-face 및 FontFace API에서 format 반영
- CDN URL과 로컬 파일 혼합 사용 지원

### 검증

- TypeScript 타입 체크: font-loader.ts 에러 없음
- 모든 woff2 파일 유효성 확인 (file 명령으로 Web Open Font 확인)
