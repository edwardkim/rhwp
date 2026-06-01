# Stage 1 보고서 — Task M100-1166: HWPX 파싱 landscape 매핑

## 목표
HWPX 가로 용지 → page_def.landscape=true 정합 + RED 회귀 테스트.

## 변경
### src/parser/hwpx/section.rs (parse_page_pr)
`b"landscape"` 분기: 종전 무시(false 고정) → OWPML 매핑.
- `NARROWLY` = 가로(Landscape) → landscape=true
- `WIDELY`/기타 = 세로(Portrait) → landscape=false
(hwplib ForSecPr: Portrait→WIDELY, Landscape→NARROWLY 권위.)

### tests/issue_1166_landscape.rs (신규)
- landscape_001_hwpx_is_landscape: 가로 HWPX → landscape=true
- portrait_hwpx_stays_portrait: 세로 HWPX(para-001) → false (회귀 가드)
- landscape_hwp5_matches_hwpx: HWP5 가로 정합

## 검증
- issue_1166 3 passed (파싱 정정으로 가로 HWPX landscape=true)
- 렌더 swap 은 page.rs:177 기존 로직이 landscape=true 시 자동 적용
- cargo test --tests 전수 통과, fmt 정합

## 다음 (Stage 2)
HWPX 직렬화: 템플릿 pagePr 하드코딩(landscape="WIDELY" width/height)을 IR page_def 로 치환 (round-trip).
