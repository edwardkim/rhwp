# Task #67 — 최종 완료보고서

## 폰트 폴백 전략 수립 및 적용 (1~4단계) ✅

### 수정 파일

| 파일 | 내용 |
|------|------|
| `web/fonts/` | 오픈소스 woff2 17개 추가 |
| `web/fonts/FONTS.md` | 카테고리별 분류, 라이선스/출처 명시 |
| `src/renderer/mod.rs` | `generic_fallback()` 전 플랫폼 CSS 체인 |
| `rhwp-studio/src/core/font-loader.ts` | FONT_LIST 개편 + CDN 참조 + OS 폰트 감지 |
| `rhwp-studio/src/core/font-substitution.ts` | `fontFamilyWithFallback()` 전 플랫폼 체인 |
| `web/editor.html` | 폰트 목록 동기화 |
| `mydocs/tech/font_fallback_strategy.md` | 전략 보고서 |

### 단계별 완료 내용

#### 1단계: 저작권 폰트 제거 + 오픈소스 폰트 도입
- 오픈소스 woff2 17개 추가 (Noto Sans/Serif KR, 나눔고딕/명조, D2 Coding, 고운바탕/돋움 등)
- 함초롬바탕/돋움: jsdelivr CDN 참조 (비상업적 사용, 한컴 라이선스)
- 저작권 폰트(HY, MS) → 오픈소스 대체 매핑

#### 2단계: Rust 폰트 매핑 테이블 수정
- `generic_fallback()` 3카테고리 분리 (Serif/Sans-serif/Monospace)
- 전 플랫폼 CSS 체인: Windows → macOS/iOS → Android → 오픈소스 → generic

#### 3단계: 프런트엔드 폰트 로더 수정
- font-loader.ts, font-substitution.ts, editor.html 동기화
- `FontEntry` format 필드 추가 (woff/woff2 구분)

#### 4단계: Canvas 폰트 감지 + OS 폰트 우선
- `document.fonts.check()` 기반 OS 폰트 감지 (Windows/macOS/iOS/Android)
- OS에 설치된 폰트는 웹폰트 로딩 건너뛰어 네트워크 요청 절감

### 후속 이슈

| Issue | 내용 | 상태 |
|-------|------|------|
| [#68](https://github.com/edwardkim/rhwp/issues/68) | SVG export 폰트 서브셋 임베딩 | 대기 |
| [#69](https://github.com/edwardkim/rhwp/issues/69) | 오픈소스 대체 폰트 메트릭 보정 | 대기 |

### 검증 결과

- `cargo test`: 783 passed, 0 failed
- TypeScript 타입 체크: font-loader/font-substitution 에러 없음
- SVG export: 전 플랫폼 font-family 체인 출력 확인
