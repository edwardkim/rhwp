# Task #76: Chrome/Edge 확장 프로그램 — 4단계 완료 보고서

## 완료 항목

### 오류 처리 강화
- 다운로드 인터셉터: try-catch 추가, 대용량 파일(50MB 초과) 경고 로그
- loadFromUrlParam: fetch 실패 시 Service Worker 프록시 폴백, 에러 메시지 표시
- 인쇄: 팝업 차단 시 안내 메시지

### 개발자 가이드 (DEVELOPER_GUIDE.md)
- `data-hwp-*` 프로토콜 전체 사양서
- 3단계 적용 가이드: 최소(속성 1개) → 표준(메타데이터) → 고급(썸네일/양식)
- CMS별 적용 예제: 그누보드, XpressEngine
- 디버깅 명령어: rhwpDev.inspect(), inspectLink(), help()
- 확장 감지 방법 (data 속성, CustomEvent)
- FAQ

### 개인정보 처리방침 (PRIVACY.md)
- 한국어/영어 이중 작성
- 파일 미전송, 데이터 미수집, 외부 서버 미통신 명시
- 권한별 사용 목적 표
- 오픈소스 안내

### README.md
- 확장 프로그램 소개, 특징
- 사용 방법 (다운로드, 드래그&드롭, 우클릭, 배지, 인쇄, 저장)
- 웹사이트 개발자 안내
- 빌드/설치 가이드

### 스토어 배포

| 항목 | 상태 |
|------|------|
| manifest.json | ✓ 완료 |
| 아이콘 (16/32/48/128/300px) | ✓ 완료 |
| 개인정보 처리방침 | ✓ 완료 |
| README | ✓ 완료 |
| 개발자 가이드 | ✓ 완료 |
| i18n (한국어/영어) | ✓ 완료 |
| 스크린샷 (1280x800) | ✓ 완료 (Chrome/Edge 각 4장) |
| Small promotional tile (440x280) | ✓ 완료 |
| Large promotional tile (1400x560) | ✓ 완료 (Chrome/Edge 각 1장) |
| Chrome Web Store 등록 | ✓ 심사 대기 중 |
| Microsoft Edge Add-ons 등록 | ✓ 심사 대기 중 (Store ID: 0RDCKCKDPBQR) |
