# 최종 보고서 — Task M100-1166: HWPX 가로 편집 용지 무시 결함

## 1. 이슈

#1166 (bug, 외부 제보). 가로 편집 용지 HWPX 가 rhwp 에서 세로로 렌더링됨.
`getPageDef()` 가 `landscape:false` + 세로 치수 반환.

작업지시자 추가 요구: HWPX→HWP 저장 시 용지 가로/세로 보존 + 섹션별 용지 설정.

## 2. 근본 원인 (진단 완료)

### 권위 매핑 (hwplib ForSecPr.java:151-158)
- `WIDELY` = 세로(Portrait) → `landscape=false`
- `NARROWLY` = 가로(Landscape) → `landscape=true`
- width/height 는 짧은변/긴변 (HWP5 동일 규약). landscape=true 면 렌더가 swap.

샘플 검증: para-001(세로)=WIDELY, landscape-001(가로)=NARROWLY.

### 결함 위치 3곳 (HWPX 경로 — HWP5 파싱/렌더는 정상)
1. **HWPX 파싱** `parser/hwpx/section.rs:234`: `landscape` 무시(false 고정) — 가로 손실
2. **HWPX 직렬화** `serializer/hwpx/section.rs`: 템플릿 pagePr 하드코딩(landscape="WIDELY" width/height) — IR 미반영
3. **HWP5 직렬화** `serializer/control.rs serialize_page_def`: `pd.attr` 그대로 출력. HWPX 출처는 pd.landscape 만 set, attr bit0=0 → HWPX→HWP 저장 시 가로 손실 (작업지시자 지적)

## 3. 정정

| 파일 | 변경 |
|------|------|
| `parser/hwpx/section.rs` parse_page_pr | `landscape` 속성 매핑: `NARROWLY`=true / `WIDELY`(기타)=false |
| `serializer/hwpx/section.rs` replace_page_pr | 템플릿 pagePr 의 landscape/width/height 를 IR page_def 로 치환 |
| `serializer/control.rs` serialize_page_def | landscape 를 attr bit0 에 동기화 (`attr|0x01` / `attr & !0x01`) |
| `tests/issue_1166_landscape.rs` (신규) | 파싱 3 + 저장 round-trip 3 |
| fixture | `samples/hwpx/landscape-001.hwpx`, `samples/landscape-001.hwp` (작업지시자 제공) |

### 섹션별 처리 (작업지시자 요구)
rhwp 는 이미 `sections[].section_def.page_def` 섹션별 PageDef 보유. 파싱/직렬화 모두
섹션별 page_def 를 처리하므로 섹션별 다른 용지 방향 자연 지원.

### 핵심: 파싱 정정만으로 렌더 자동 정합
`PageDef.landscape` 필드 + 렌더 swap(page.rs:177)이 이미 존재 → 파싱 한 줄 정정으로
가로 HWPX 가 landscape=true + 가로 렌더까지 자동 정합.

## 4. 검증

| 항목 | 결과 |
|------|------|
| issue_1166 (6) | ✅ 파싱 3 + HWPX/HWP5 저장 round-trip 3 |
| `cargo test --tests` 전수 | ✅ 실패 없음 |
| native-skia / fmt / clippy --lib | ✅ clean |
| wasm 빌드 | ✅ Stage 1+2 반영 |
| **작업지시자 동작 판정** | ✅ 통과 (가로 렌더 + HWPX/HWP 저장본 가로 유지) |

## 5. 메모리 룰 정합

- `feedback_no_inference_authoritative_spec` — NARROWLY/WIDELY 매핑을 추측 않고 hwplib 권위 + 샘플 2개 교차검증으로 확정 (초기 "NARROWLY=세로" 추측을 검증으로 정정)
- `feedback_self_verification_not_hancom` — HWPX/HWP5 저장은 자기 round-trip + 작업지시자 동작 판정 게이트
- `feedback_image_renderer_paths_separate` 정신 — 파싱/HWPX직렬화/HWP5직렬화 3 경로 각각 점검 (작업지시자 지적으로 HWP5 직렬화 누락 발견·보강)

## 6. 처리 과정 비고

- 초기 Stage 2 는 HWPX→HWPX 저장만 다뤘고, 작업지시자가 HWPX→HWP 저장 미동작을 지적 → HWP5 직렬화(attr bit0) 결함을 추가 발견·정정. 단일 경로 추정의 한계를 작업지시자 테스트가 보완.
