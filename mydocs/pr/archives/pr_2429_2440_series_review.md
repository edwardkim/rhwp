# kevin9327 4차 후반 연작 검토 — #2429 계약 + 실증 7건 (2026-07-19 야간)

## #2429 — 직렬화 passthrough·무효화 3계층 계약 (canonical)

트러블슈팅(레코드 계층, 당일 오전 등재)을 tech README 승격 규정대로 3계층
공통 계약으로 승격 — 게이트 전부 코드 실증(raw_data/raw_stream/
raw_stream_dirty), front matter·canonical 귀속·지도 연결. **merge — 연작의
규범 참조로 확정.**

## 그룹 1 — 무효화 계열 (#2434 누름틀 raw_stream / #2435 스타일 DocInfo)

계약의 섹션·DocInfo 계층 조항 구현. red→green 전건(#2434 2건, #2435 각
1건), roundtrip 무회귀. **merge.**

## 그룹 2 — HWPX fidelity (#2432 allowOverlap / #2438 textDirection)

#2415 계열 — 이웃 속성 정합 완결·세로쓰기 저장 유실. hwpx_roundtrip 4/4,
수술적 red(타입 정합 재시도 후) → green. **merge.**

## 그룹 3 — HWP5 스펙 정합 (#2436 WCHAR / #2437 비트 30 / #2440 시프트)

스펙 원문 교차 확인(WCHAR 2바이트·비트 30=묶음 빈칸). red→green: #2436
chars 되돌림 FAIL / #2437 분기 제거 FAIL(정확 분기 특정 재시도) / #2440
기여자 red 기록 + insert_text_at 동형성. **merge — 총결 코멘트 게시.**

## 총평

kevin9327 하루 21건(1차 5·2차 3·3차 5·4차 8) — 지식 증류(트러블슈팅→계약
→실증)의 당일 완결 사이클. red 변형 시 치환 적용 assert 필수 교훈(무단
통과 2회 자기 검출).
