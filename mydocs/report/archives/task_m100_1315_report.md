# Task M100 #1315 — 최종 결과 보고서

## 타스크 개요

**HWPX 기본 시리얼라이제이션 baseline 구축: IR roundtrip 기반 samples/hwpx 검증** (M100/v1.0.0)

- 목표: HWPX→IR→HWPX roundtrip 검증 하네스 + 샘플 등급화 + 최소 호환성 기준 + 회귀 도구화 + 문서화
- 범위 원칙: full fidelity serializer 완성이 목표가 아님. 실패 발견 시 serializer 수정으로
  확전하지 않고 xfail/known limitation 분류 (이슈 주의사항 준수 — 전 단계 소스 본체 무수정 유지)

## 구현물 요약

| 산출물 | 내용 | 단계 |
|--------|------|------|
| `src/serializer/hwpx/roundtrip.rs` | `diff_documents()` pub 전환 (IR 뼈대 비교) | 1 |
| `src/diagnostics/hwpx_roundtrip_batch.rs` | CLI `rhwp hwpx-roundtrip` — 단일/배치, 13컬럼 TSV, 종료 코드 | 1·2 |
| `src/serializer/hwpx/package_check.rs` | `check_package()` — 패키지(ZIP) 구조 6항목 검사 + 단위테스트 8건 | 2 |
| `tests/hwpx_roundtrip_baseline.rs` | 전수 회귀 게이트 4건 — 신규 샘플 자동 포함, xfail 승격 강제 | 3 |
| `rhwp-studio/e2e/task1315-load.check.mjs` | rt 파일 bytes 주입 로드 검증 (호스트 Chrome CDP) | 4 |
| `mydocs/manual/hwpx_roundtrip_baseline.md` | 등급 체계·CLI·유지보수 매뉴얼 | 5 |
| CLAUDE.md | `hwpx-roundtrip` 명령 안내 추가 | 5 |

serializer 본체(`section.rs`, `context.rs`, `table.rs`) 무수정 — PR #1366 재제출 충돌 회피.
기존 `tests/hwpx_roundtrip_integration.rs` 무수정 (누적-만-가능 원칙).

## 전수 측정 결과 (`samples/hwpx/` .hwpx 54건)

baseline 기준: parse→serialize→재parse + IrDiff 0 + 패키지 검사 + 2-round IrDiff 0.

| 등급 | 건수 | 내용 |
|------|------|------|
| **A (baseline)** | **52** | 전 기준 통과 — 회귀 게이트로 고정 |
| B (xfail) | 1 | `exam_social.hwpx` — serializer 미등록 borderFillIDRef 31 |
| 제외 | 1 | `hwpx-01.hwpx` — HWP5(OLE CFB)가 .hwpx 확장자로 저장된 샘플 |
| C (oracle 부적합) | 13 | A의 부분집합 — 복합 실문서, full visual fidelity oracle 금지 |

## 시각 검증 (4단계, 대표 8건)

- SVG byte 비교: 3건 일치 (blank_hwpx, para-001, basic-table-01) / 5건 차이
- **페이지 수 변화**: form-002 10→17쪽, 보도자료(2025-1Q) 9→13쪽
- rhwp-studio 로드: **8/8 성공** (오류·무응답 없음)
- 한컴에디터 판정: 작업지시자에게 rt 파일 8건 열기 판정 요청됨 (4단계 보고서)

### 원인 진단 (확정, 코드 무수정)

1. **셀 subList 텍스트 전용 직렬화** (`table.rs` `write_sub_list`) — 셀 내 그림/컨트롤 소실
   (ta-pic-001-r `hp:pic` 4→0) + 합성 lineseg 고정
2. **본문 lineseg 보존 불완전** — lh/th/bl/sw 변형 → 페이지 수 변화·px 시프트
3. baseline이 이를 통과시키는 이유: `diff_documents`는 뼈대(카운트)만 비교 —
   "구조 보존 ≠ 시각 충실" 한계를 실증. 매뉴얼·테스트 doc 주석에 명시함

## 검증 결과 (5단계 CI급)

- `cargo test --lib` — 통과
- `cargo test --tests` — 통과 (baseline 게이트 포함)
- `cargo fmt --check` — 통과
- `cargo clippy --lib --tests -- -D warnings` — 통과
- `cargo check --lib --target wasm32-unknown-unknown` — 통과

## 별도 이슈 후보 (승인 시 등록)

| 후보 | 내용 |
|------|------|
| ① 셀 subList 텍스트 전용 직렬화 | 셀 내 그림/컨트롤 소실 + 합성 lineseg |
| ② 본문 lineseg 보존 불완전 | 페이지 수 변화 (form-002 10→17, 보도자료 9→13) |
| ③ exam_social borderFillIDRef 31 | xfail 등록됨 — parser→serializer ID 매핑 경계 |

## 잔여 판단 요청

1. 별도 이슈 후보 ①·②·③ 등록 여부
2. rt 파일 8건 한컴에디터 열기 판정 (최소 호환성 기준 확정)
3. `hwpx-01.hwpx` 샘플 정비 여부 — 현상 유지 제안
4. 이슈 #1315 클로즈 여부

## 단계별 보고서

- 1단계: `mydocs/working/task_m100_1315_stage1.md` — 배치 CLI + 전수 인벤토리
- 2단계: `mydocs/working/task_m100_1315_stage2.md` — 패키지 검사 + 2-round
- 3단계: `mydocs/working/task_m100_1315_stage3.md` — 등급화 + 전수 통합 테스트
- 4단계: `mydocs/working/task_m100_1315_stage4.md` — SVG 비교 + rhwp-studio 확인
