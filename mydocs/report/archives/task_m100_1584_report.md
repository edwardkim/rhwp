# Task #1584 — 최종 결과보고서

**제목**: HWPX 저장 시 본문 문단의 인라인 ColumnDef(cold) 드롭 수정
**마일스톤**: M100 (v1.0.0) · **이슈**: edwardkim/rhwp#1584 · **브랜치**: `local/task1584`

---

## 1. 문제

실문서 무손실 검증(hwpdocs 9350)에서 잔여 IR_DIFF 최대 단일 클래스(49건). 게이트는
"표 셀 char_shape ID 오매핑"으로 보고했으나, raw XML/verbose ir-diff 로 **진짜 근본원인을
본문 첫 문단의 인라인 ColumnDef 드롭으로 정정**함. 셀 char_shape 변위는 컨트롤 인덱스
시프트의 하위 증상.

```
orig 문단0: [secPr, ctrl,colPr, ctrl,colPr, ctrl,fieldBegin, tbl]   colPr 2
rt   문단0: [secPr, ctrl,colPr, ctrl,        fieldBegin, tbl]        colPr 1  ← 드롭
```

## 2. 근본원인 (`src/serializer/hwpx/section.rs`)

- 섹션 템플릿 앵커가 본문 첫 문단의 **첫 ColumnDef 1개만** colPr 로 흡수.
- 본문(depth 0) 인라인 슬롯 필터가 ColumnDef 를 전부 제외 → **2번째+ ColumnDef 가 어느
  경로로도 방출되지 않아 드롭**(controls 6→5, cc −8, 후속 컨트롤 인덱스 시프트).

## 3. 해결 (Option A surgical)

| 파일 | 변경 |
|------|------|
| `context.rs` | `body_coldef_template_pending` consume-once 플래그 추가 |
| `section.rs` write_section | 첫 문단 렌더 전후로 플래그 set/reset |
| `section.rs` render_runs | **all-controls 분기**: 첫 ColumnDef 슬롯 유지(회계 보존). **filter 분기**: 첫 ColumnDef 슬롯 제외(위치 미점유분), 2번째+ 포함 |
| `section.rs` render_control_slot | 본문 첫 ColumnDef 의 XML 만 consume-once 억제(템플릿 중복 방지) |

**설계 핵심**: 두 슬롯 분기가 ColumnDef 의 char-offset 슬롯 점유 여부에서 다르므로 분기별로
처리. 단순 일괄 제거는 회귀(−8 시프트 / position→mismatch 전환)를 유발 — 통제 비교로 검출·교정.

## 4. 검증

| 검사 | 결과 |
|------|------|
| 단위 RED→GREEN | PASS (ColumnDef 1→2 보존) |
| `cargo test --lib` | 1960 passed, 0 failed |
| `hwpx_roundtrip_baseline` | 4/4 (#1407/#1388 보존) |
| opengov snapshot (36382399 가드 추가) | PASS |
| **fidelity 통제 비교** | **개선 49 / 회귀 0 / 순효과 +49** |
| Hangul 오라클 (8 표본) | OK 8 / COLLAPSE 0 |

실문서 HWPX IR_DIFF: **59 → 10** (공통 9350 기준 −49, 악화 0).

## 5. 산출물

- 소스: `src/serializer/hwpx/section.rs`, `src/serializer/hwpx/context.rs`
- 테스트: `task1584_..._roundtrip` 단위 + `samples/hwpx/opengov/36382399…` 통합 가드
- 문서: 수행/구현 계획서, `_stage1~3`, 본 보고서

## 6. 후속

잔존 IR_DIFF 10건(F3 잔여 다중필드 복합슬롯 + shapeComment + ruby) 및 PARSE_FAIL 12건
(손상 다운로드, rhwp 무관)은 별건. 채택 후 `local/devel` → `devel` 반영.
