# Task #1584 — Stage 2 완료보고서

**단계**: Option A 구현 (GREEN)
**브랜치**: `local/task1584`

## 변경 내용

| # | 파일 | 변경 |
|---|------|------|
| C1 | `context.rs` | `body_coldef_template_pending: bool` 필드 추가 (consume-once 플래그) |
| C2 | `section.rs` write_section | 첫 문단 렌더 직전 플래그 set, 직후 reset |
| C3 | `section.rs` render_runs | 본문 ColumnDef 슬롯 처리를 분기별로 정밀화 (아래) |
| C4 | `section.rs` render_control_slot | 본문 첫 ColumnDef 의 XML 만 consume-once 로 억제 |

## 설계 핵심 — 두 슬롯 분기의 차이

초기 단순 구현(첫 ColumnDef 를 slots 에서 일괄 제거)은 회귀를 유발했다:
- **all-controls 분기**(`slot_count==len`)에서 ColumnDef 는 char-offset 슬롯을 점유 →
  제거 시 8유닛 회계 손실 → 후속 char_shape −8 시프트 (aift/exam_kor 회귀).
- **filter 분기**(`slot_count!=len`)의 단일 ColumnDef 는 위치 슬롯 미점유(추정 카운트 제외) →
  추가 시 position→mismatch 경로 전환 → equation 테스트 회귀.

→ 분기별로 다르게 처리:
- **all-controls**: 첫 ColumnDef 를 슬롯에 **유지**(회계 보존), XML 만 consume-once 억제.
- **filter**: 첫 ColumnDef 를 슬롯에서 **제외**(위치 슬롯 미점유분), 2번째+ 만 포함 →
  드롭 방지. 제외 시 consume-once 플래그 해제(2번째 오억제 방지).

## 검증 결과 (모두 GREEN)

| 검사 | 결과 |
|------|------|
| RED 테스트 `task1584_..._roundtrip` | **PASS** (1→2 보존) |
| `equation_control_*` (3건, 회귀 가드) | PASS |
| `cargo test --lib` 전체 | **1960 passed, 0 failed** |
| `cargo test --test hwpx_roundtrip_baseline` | **4/4 PASS** (#1407/#1388 컬럼 보존) |
| `cargo clippy --lib` (변경 파일) | 무경고 |
| 실문서 `36382399` roundtrip | **PASS diff=0**, colPr 2==2 (인라인 ColumnDef 보존) |

## 다음 단계

Stage 3 — fidelity 전수(hwpdocs 9350 + samples 319) 통제 비교: 49건 IR_DIFF 해소 확인,
악화 0 확인, 순효과>0. 스냅샷 갱신.
