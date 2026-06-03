# Task #67: 폰트 폴백 전략 적용 — 구현 계획서

## 현재 아키텍처

### 폰트 파일 (`web/fonts/`)
- Pretendard (9 weight) — OFL ✅
- Happiness Sans, Cafe24, SpoqaHanSans — OFL ✅
- **MS 폰트** (Arial, Calibri, MalgunGothic 등) — ❌ 저작권
- **한컴 폰트** (함초롬바탕/돋움, HY계열) — ❌ 로컬 배포 전용

### 폰트 로딩 (`rhwp-studio/src/core/font-loader.ts`)
- FONT_LIST: 90+ 폰트명 → woff2 매핑
- 2단계 로딩: @font-face 등록 → FontFace API 비동기 로드
- CRITICAL_FONTS: 함초롬바탕, 함초롬돋움

### 폰트 치환 (`rhwp-studio/src/core/font-substitution.ts`)
- 3단계 해소: 등록폰트 확인 → 치환 체인 → generic 폴백
- 7개 언어별 치환 테이블

### Rust 렌더러 (`src/renderer/`)
- `style_resolver.rs`: HFT/TTF 폰트 치환 (한컴바탕→함초롬바탕 등)
- `mod.rs`: `generic_fallback()` — Serif/Sans-serif 판별
- `svg.rs`, `web_canvas.rs`: font-family + generic 폴백 적용

---

## 구현 단계 (6단계)

### 1단계: 저작권 폰트 제거 + 오픈소스 폰트 추가

**대상 파일:**
- `web/fonts/` — 저작권 woff2 삭제, 오픈소스 woff2 추가
- `web/fonts/FONTS.md` — 폰트 목록 갱신

**작업 내용:**
1. `web/fonts/`에서 저작권 폰트 woff2 제거
   - MS 계열: Arial, Calibri, Courier New, MalgunGothic, Tahoma, Times New Roman, Verdana
   - 한컴 계열: hamchob-r, hamchod-r, h2hdrm, hygtre, hygprm, hymjre
   - 기타 저작권: Webdings, Wingdings3
2. 오픈소스 폰트 woff2 추가
   - Noto Serif KR (Regular, Bold) — Serif 대체
   - D2Coding (Regular) — Monospace 대체
   - Pretendard는 이미 있으므로 유지
3. `FONTS.md` 갱신: 폰트별 라이선스, 출처 명시

**검증:** 추가된 폰트 파일이 OFL/Apache 라이선스인지 확인

---

### 2단계: Rust 폰트 매핑 테이블 수정

**대상 파일:**
- `src/renderer/style_resolver.rs` — 폰트 치환 매핑
- `src/renderer/mod.rs` — generic_fallback() 개선

**작업 내용:**
1. `resolve_font_substitution()` 매핑 변경
   - Serif 계열 (한컴바탕, 함초롬바탕, 바탕, HY명조 등) → "Noto Serif KR"
   - Sans-serif 계열 (맑은 고딕, 함초롬돋움, 돋움, 한컴돋움 등) → "Pretendard"
   - Monospace (굴림체, 바탕체) → "D2Coding"
2. `generic_fallback()` CSS 체인에 OS 폰트 우선 삽입
   - Serif: `"바탕", "Batang", "Noto Serif KR", serif`
   - Sans-serif: `"맑은 고딕", "Malgun Gothic", "Apple SD Gothic Neo", "Pretendard", sans-serif`
   - Monospace: `"굴림체", "GulimChe", "D2Coding", monospace`

**검증:** SVG export에서 font-family 체인이 올바르게 출력되는지 확인

---

### 3단계: 프런트엔드 폰트 로더 수정

**대상 파일:**
- `rhwp-studio/src/core/font-loader.ts` — FONT_LIST, CRITICAL_FONTS
- `rhwp-studio/src/core/font-substitution.ts` — 치환 테이블
- `web/editor.html` — 인라인 폰트 로딩

**작업 내용:**
1. FONT_LIST에서 저작권 폰트 매핑 제거, 오픈소스 폰트 매핑 추가
   - `{name: "Noto Serif KR", file: "NotoSerifKR-Regular.woff2", aliases: [...]}`
   - `{name: "D2Coding", file: "D2Coding-Regular.woff2", aliases: [...]}`
2. CRITICAL_FONTS 변경: `함초롬바탕/돋움` → `Noto Serif KR, Pretendard`
3. font-substitution.ts 치환 체인 갱신
   - 함초롬바탕 → Noto Serif KR
   - 함초롬돋움 → Pretendard
4. `editor.html` 인라인 폰트 목록 동기화

**검증:** 웹 브라우저에서 HWP 문서 로드 시 오픈소스 폰트로 렌더링

---

### 4단계: Canvas 폰트 감지 + OS 폰트 우선

**대상 파일:**
- `rhwp-studio/src/core/font-loader.ts` — OS 폰트 감지 로직
- `rhwp-studio/src/core/local-fonts.ts` — 기존 Local Font Access API 활용

**작업 내용:**
1. `document.fonts.check()` 기반 OS 폰트 감지 유틸리티
   - Windows: 맑은 고딕, 바탕, 굴림체 등
   - macOS: Apple SD Gothic Neo, AppleMyungjo 등
   - Linux: Noto Sans CJK KR 등
2. OS 폰트가 있으면 해당 폰트 사용, 없으면 오픈소스 웹폰트 로드
3. font-loader에 감지 결과 캐싱

**검증:** Windows/macOS에서 OS 폰트 우선 사용 확인

---

### 5단계: SVG export 서브셋 임베딩

**대상 파일:**
- `src/renderer/svg.rs` — SVG 서브셋 임베딩 로직
- 의존성: 폰트 서브셋 라이브러리 (Rust crate 또는 외부 도구)

**작업 내용:**
1. SVG 내보내기 시 사용된 글자(codepoint) 수집
2. 시스템에 원본 폰트가 있으면 서브셋 추출 + base64 woff2로 SVG `<style>` 임베딩
3. 원본 폰트가 없으면 오픈소스 폰트로 폴백 (3계층 체인만 적용)
4. 옵션: `--embed-fonts` 플래그로 임베딩 on/off

**검증:** 오프라인에서 SVG 파일의 한글 텍스트가 올바르게 표시

---

### 6단계: 메트릭 보정 + 회귀 테스트

**대상 파일:**
- `src/renderer/font_metrics_data.rs` — 오픈소스 폰트 메트릭 추가
- 테스트 스크립트

**작업 내용:**
1. Noto Serif KR, D2Coding의 폰트 메트릭 데이터 생성 (font-metric-gen)
2. 기존 함초롬바탕/돋움 메트릭과 비교, 차이 보정
3. 기존 샘플 문서로 렌더링 비교 (줄바꿈 위치, 문자 간격)
4. `cargo test` 전체 통과 확인
5. 67페이지 전체 내보내기 회귀 없음 확인

**검증:** 기존 렌더링 품질이 크게 저하되지 않음

---

## 리스크

| 리스크 | 영향 | 대응 |
|--------|------|------|
| 오픈소스 폰트 메트릭 차이 | 줄바꿈 위치 변경 | 메트릭 보정 로직으로 완화 |
| Noto Serif KR woff2 용량 (~4MB) | 초기 로딩 지연 | unicode-range 분할, 지연 로딩 |
| SVG 서브셋 임베딩 시 원본 폰트 필요 | 서버 환경에서 폰트 부재 | 폴백 체인으로 대체 |
| OS 폰트 감지 API 브라우저 호환성 | Safari 미지원 | document.fonts.check() 폴백 |
