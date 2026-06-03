# Task #67 — 2단계 완료보고서

## Rust 폰트 매핑 테이블 수정 ✅

### 수정 파일

- `src/renderer/mod.rs` — `generic_fallback()` 전 플랫폼 CSS 체인 구성

### 변경 내용

`generic_fallback()` 함수를 3개 카테고리로 분리하고, 전 플랫폼(Windows/macOS/iOS/Android) 폴백 체인을 구성.

#### Serif 체인
```
바탕 → AppleMyungjo → Noto Serif KR → serif
```
- Windows: 바탕 (기본 설치)
- macOS/iOS: AppleMyungjo (기본 설치)
- Android: Noto Serif KR (기본 설치)

#### Sans-serif 체인
```
맑은 고딕 → Apple SD Gothic Neo → Noto Sans KR → Pretendard → sans-serif
```
- Windows: 맑은 고딕 (기본 설치)
- macOS/iOS: Apple SD Gothic Neo (기본 설치)
- Android: Noto Sans KR (기본 설치)

#### Monospace 체인 (신규)
```
굴림체 → D2Coding → Noto Sans Mono → monospace
```
- 굴림체/바탕체/Courier 키워드 감지

### 검증 결과

- `cargo test`: 783 passed, 0 failed
- SVG export: font-family 체인에 AppleMyungjo, Noto Serif KR 등 포함 확인
