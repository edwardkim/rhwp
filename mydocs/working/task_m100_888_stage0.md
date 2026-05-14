# Task #888 Stage 0 작업 로그

## 1. 작업 브랜치

- 기준 브랜치: `local/devel`
- 통합 브랜치: `local/task_m100_888_integrate_854`
- worktree: `/tmp/rhwp-task-888`
- GitHub issue: https://github.com/edwardkim/rhwp/issues/888
- 동기화 기준 커밋: `d4ece849` (`origin/devel`)

## 2. Stage 0 목표

#854 POC 에서 Stage 14 정답 판정까지 확인된 변경만 `local/devel` 기반 브랜치로 선별 이식한다. POC 브랜치가 오염된 상태였기 때문에 직접 머지하지 않고 중간 브랜치에서 검증한다.

작업 중 PR 처리로 `devel` 기준선이 변경되었으므로, Stage 0 중간에 `origin/devel` 을 다시 fetch 하고 `local/devel` 을 `d4ece849` 로 동기화했다. #888 변경분은 stash 로 보관한 뒤 최신 기준선 위에 충돌 없이 재적용했다.

## 3. 이식한 코드 범위

| 파일 | 내용 |
|------|------|
| `src/document_core/converters/common_obj_attr_writer.rs` | common object attr packing helper 재사용 가능화 |
| `src/document_core/converters/hwpx_to_hwp.rs` | table materialization, no-fill BorderFill normalization, adapter report 확장 |
| `src/parser/hwpx/section.rs` | HWPX `pageBorderFill` 파싱 및 HWP SectionDef 매핑 |
| `tests/hwpx_to_hwp_adapter.rs` | #854 확정 회귀 테스트 추가 |

## 4. 샘플

| 파일 | 목적 |
|------|------|
| `samples/hwpx/basic-table-01.hwpx` | basic-table 한컴 손상/문단 배경 무늬 회귀 |
| `samples/hwpx/expense_report.hwpx` | TAC table 배치/page background/한컴 손상 회귀 |

`samples/hwpx/tac-table-01.hwpx` 는 현재 worktree 및 POC worktree 에서 `.hwpx` 원본을 찾지 못했다. 이번 Stage 0 통합 대상에는 포함하지 않았다.

## 5. 범위 조정

POC 패치에 포함된 `stage1_export_hwp_with_adapter_preserves_live_hwpx_ir_snapshot` 테스트는 제거했다.

사유:

- 이 테스트는 저장 결과가 아니라 `export_hwp_with_adapter()` 호출 후 live `DocumentCore` 내부 구조가 변하지 않는지를 검사한다.
- `local/devel` 의 현재 저장 진입점은 일부 materialization 을 live IR 에 반영하는 구조다.
- #888 Stage 0 의 목표는 #854 POC 에서 확인한 한컴 호환 저장 결과의 선별 통합이며, save clone 아키텍처 확대 적용은 비범위로 둔다.
- 기존 HWP 출처 no-op 및 반복 저장 결과 idempotent 테스트는 유지한다.

## 6. 검증

```text
cargo test --test hwpx_to_hwp_adapter
```

결과:

```text
30 passed; 0 failed
```

동일 테스트를 `local/devel = d4ece849` 동기화 후에도 재실행했다.

```text
cargo test --test hwpx_to_hwp_adapter
30 passed; 0 failed
```

## 7. 현재 판정

Stage 0 통합 기준은 통과했다.

다음 단계에서는 필요한 경우 HWP 산출물을 생성해 작업지시자 환경의 한컴 에디터/rhwp-studio 시각 판정을 받는다. 그 뒤 커밋 단위와 local/devel 반영 절차를 확정한다.
