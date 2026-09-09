# 구현 계획서 — Task #1584

**제목**: HWPX 본문 인라인 ColumnDef(cold) 드롭 수정 (Option A surgical)
**브랜치**: `local/task1584` · **이슈**: edwardkim/rhwp#1584
**전제**: 수행계획서(`task_m100_1584.md`) 승인됨

---

## 1. 변경 대상 요약

`src/serializer/hwpx/section.rs` 의 본문 인라인 슬롯 경로 + 슬롯 렌더 디스패치 +
`src/serializer/hwpx/context.rs` 의 컨텍스트 플래그 1개.

| # | 파일:지점 | 현재 | 변경 |
|---|-----------|------|------|
| C1 | context.rs:97 (`SerializeContext`) | `sub_list_depth` 만 | `is_section_first_para: bool` 필드 추가 |
| C2 | section.rs:78–82 (assemble) | 첫 문단 무표식 렌더 | 첫 문단 렌더 전후로 `ctx.is_section_first_para` true/false 설정 |
| C3 | section.rs:424–438 (slots 구성) | ColumnDef 를 본문 인라인에서 제외 | ColumnDef 를 인라인 후보로 포함 + **본문 첫 문단의 첫 ColumnDef 1개 제거**(템플릿 흡수분) |
| C4 | section.rs:832 (`render_control_slot`) | `ColumnDef if depth>0` 만 방출 | 가드 완화 — slots 에 들어온 ColumnDef 는 본문도 방출 |

> C1·C2 는 "섹션 템플릿(line 119–127)이 첫 ColumnDef 1개를 흡수한다"는 사실을
> render_runs 가 알도록 신호를 전달하기 위함. 그 1개만 인라인에서 빼고 2번째+는 방출.

## 2. 설계 근거 (드롭/중복 양쪽 방지)

핵심 불변식: **본문 첫 문단의 첫 ColumnDef = 섹션 템플릿이 흡수, 나머지 = 인라인.**

- C3 에서 ColumnDef 를 인라인 후보로 올리면 `slot_count == controls.len()`(line 425, 전 컨트롤
  슬롯) 분기와 필터 분기(line 431) **양쪽** 모두 ColumnDef 를 포함하게 된다. 따라서
  **두 분기 공통으로** "본문 첫 문단이면 slots 에서 첫 ColumnDef 1개 제거"를 적용한다
  (`if ctx.sub_list_depth == 0 && ctx.is_section_first_para { slots.remove(첫 ColumnDef pos) }`).
- 이로써: 템플릿(1번째) + 인라인(2번째+) = **드롭 없음, 중복 없음**.
- 본문 **비첫** 문단(템플릿 미흡수): `is_section_first_para=false` → 제거 없음 → ColumnDef 전부
  인라인 방출(현재는 전부 드롭되던 잠재 버그도 동시 해소).
- subList(depth>0): 종전과 동일하게 전부 인라인(제거 없음).
- C4: slots 에 도달한 ColumnDef 는 위 불변식상 "흡수분이 아닌" 것이므로 본문에서도 무조건 방출 안전.

## 3. 슬롯 카운트 정합 메모

`inferred_control_slot_count`(line ~717)는 ColumnDef 의 8유닛 슬롯도 카운트하므로, 수정 후
`slot_count` 와 `slots.len()` 관계는 케이스별로 달라질 수 있다. 어느 경우든:
- 일치 → 위치추정 경로로 ColumnDef 가 제 위치에 방출.
- 불일치 → mismatch 경로(line 454)가 `for slot in &slots` 로 **일괄 방출** → ColumnDef 보존.

즉 **드롭은 어느 경로에서도 발생하지 않는다**(현재는 slots 에서 빠져 양쪽 다 드롭).

## 4. 구현 단계 (3단계)

### Stage 1 — 드롭 재현 회귀 테스트 박제 (RED)
- 49건 중 대표 1–2건(예 `36382399`)을 `tests/fixtures/` 또는 기존 roundtrip 테스트에 편입.
- 단위 테스트: 본문 첫 문단 ColumnDef 2개 → serialize → reparse 후 `controls` 수/ColumnDef 보존
  검증. 현재 코드에서 **실패(RED)** 함을 확인.
- `samples/hwpx` 에 동형 미니 케이스 부재 시, hwpdocs 대표 파일 기반 통합 테스트로 대체.
- 커밋: `Task #1584: ColumnDef 드롭 재현 테스트 (RED)` + `_stage1.md`.

### Stage 2 — Option A 구현 (GREEN)
- C1–C4 적용.
- Stage1 테스트 GREEN.
- `cargo test --test hwpx_roundtrip_baseline` 회귀 0 (#1407 2단·#1388 여백 케이스 보존 확인).
- `cargo clippy` 클린, 신규/수정 파일만 정리(무관 fmt diff 금지).
- 커밋: `Task #1584: 본문 인라인 ColumnDef 방출 (Option A)` + `_stage2.md`.

### Stage 3 — 통제 비교 검증 (채택 게이트)
- fidelity 전수(hwpdocs 9350 hwpx + samples 319 hwp) 재측정.
- 수정 전(devel HEAD) 대비 **개선−회귀** 집계: 49건 IR_DIFF 해소 확인, **악화 0 필수**, 순효과>0.
- `tools/verify_hangul_pages.py` 대표 샘플 페이지수 불변 확인(시각 붕괴 0).
- `tests/opengov_corpus_snapshot.rs` 스냅샷 갱신(개선 반영).
- 커밋: `_stage3.md` + `_report.md`.

## 5. 롤백 기준

Stage 3 통제 비교에서 **악화 1건 이상** 또는 baseline 회귀 발생 시: F3(#1556) 선례대로
**전량 되돌리고**(net-negative 불채택) 원인 재분석. 부분 채택하지 않는다.

## 6. 산출물
- 소스: `section.rs`, `context.rs`
- 테스트: 신규 픽스처 + roundtrip 단위/통합 테스트, opengov 스냅샷 갱신
- 문서: `_stage1.md`, `_stage2.md`, `_stage3.md`, `_report.md`
