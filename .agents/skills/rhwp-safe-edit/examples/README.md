# rhwp-safe-edit 워크스루

이 폴더는 스킬이 안내하는 경로를 **한 편씩** 끝까지 적어 둔 표본이다.
새 편집 명령을 만들지 않는다. 각 편은 이미 있는 CLI 와 이미 있는 봉투 키만 사용한다.

레퍼런스:

- 1층: [../references/single_edit.md](../references/single_edit.md)
- 3층: [../references/run_plans.md](../references/run_plans.md)
- 루프: [../references/verify_loops.md](../references/verify_loops.md)
- 봉투: [../references/failure_envelopes.md](../references/failure_envelopes.md)

픽스처 JSON 은 [../fixtures/catalog.json](../fixtures/catalog.json) 이 목록이다.
워크스루의 "기대 봉투" 블록은 그 픽스처와 같은 키를 쓴다.

## 목록

| # | 파일 | 층 | 보여주는 것 |
|---|------|----|-------------|
| 01 | [01_fill_fields_single.md](01_fill_fields_single.md) | 1 | 발견 → dry-run → `-o --verify` → fields 재독 |
| 02 | [02_replace_text_single.md](02_replace_text_single.md) | 1 | search → 치환 → matchCount 0 |
| 03 | [03_set_cell_single.md](03_set_cell_single.md) | 1 | export-tables 좌표 → set-cell → 재독 |
| 04 | [04_insert_image_single.md](04_insert_image_single.md) | 1 | HWPUNIT · overflow · run 에 넣지 않음 |
| 05 | [05_redact_single.md](05_redact_single.md) | 1 | `-o` 필수 · `--no-raw` · dry-run 먼저 |
| 06 | [06_sanitize_single.md](06_sanitize_single.md) | 1 | removedCount 재실행 0 · 본문 불변 |
| 07 | [07_run_atomic_fill_replace.md](07_run_atomic_fill_replace.md) | 3 | 스키마 → dry-run → 원자 저장 |
| 08 | [08_run_conditional.md](08_run_conditional.md) | 3 | if 한 종류 · 입력 기준 1회 판정 |
| 09 | [09_run_invalid_collected.md](09_run_invalid_collected.md) | 3 | invalid[] 전부 수집 · 디스크 무변경 |
| 10 | [10_verify_exit3.md](10_verify_exit3.md) | 1/3 | exit 3 두 갈래 (산출 남김 vs 안 남김) |
| 11 | [11_overflow_data.md](11_overflow_data.md) | 1 | overflow 는 성공 코드 안의 경고 |
| 12 | [12_ambiguous_not_complete.md](12_ambiguous_not_complete.md) | 1 | filledCount ≠ 완료 |
| 13 | [13_csv_to_table_gate.md](13_csv_to_table_gate.md) | 1 | invalid[] + 한 칸도 안 씀 |
| 14 | [14_batch_fill_row_judgment.md](14_batch_fill_row_judgment.md) | 배치 | 행별 notFound · 최종 exit 0 |
| 15 | [15_original_untouched.md](15_original_untouched.md) | 공통 | 원본 해시 대조 |
| 16 | [16_precondition_cas.md](16_precondition_cas.md) | 3 | CAS exit 3 · nextCall |

## 공통 규칙

1. 입력은 저장소 공개 샘플 (`samples/field-01.hwp`, `samples/table-001.hwp` 등) 또는
   워크스루가 밝히는 가명 경로다. 새 바이너리 픽스처를 이 폴더에 두지 않는다.
2. 명령은 `rhwp ` 로 시작하는 실재 토큰만 쓴다 (`tests/skills_contract.rs` 가 SKILL.md 를,
   `scripts/tests/test_agent_safe_edit.py` 가 레퍼런스·예제를 검사).
3. "하지 않는 호출" 절이 각 편에 있다. 무계획 `--in-place` 와 `edit` 체인은 여기 표본에 없다.
4. 기대 봉투는 키 집합이 계약이다. 수치(`filledCount: 2`)는 샘플이 바뀌면 변할 수 있어
   테스트는 키·exit·분기 이름을 고정하고 숫자는 표본으로만 둔다.
