# Stage 1 보고서 — Task M100-1323

- 이슈: #1323
- 작성일: 2026-06-10
- 브랜치: `local/task1323`

## 1. 작업 요약

`Paragraph::merge_from`(`src/model/paragraph.rs`)을 보강하여 컨트롤 보유 문단 병합 시
`controls`/`ctrl_data_records`/`control_mask`가 보존되도록 했다. TDD 방식으로 실패
테스트(컨트롤 드롭 재현) 4건을 먼저 작성·확인한 뒤 구현했다.

## 2. 변경 파일

| 파일 | 내용 |
|------|------|
| `src/model/paragraph.rs` | `merge_from` 컨트롤 병합 보강 (5개 변경점) |
| `src/model/paragraph/tests.rs` | merge_from 컨트롤 단위 테스트 5건 + 헬퍼 추가 |

## 3. 구현 세부 (구현계획서 Stage 1 항목별)

1. **early-return 조건 수정**: `other.text.is_empty() && other.controls.is_empty()`일
   때만 반환. 텍스트 없이 컨트롤만 있는 문단(그림 복사 클립보드)도 병합 진행.
2. **utf16_end 계산 보강**: `control_text_positions()`로 trailing 컨트롤
   (`position >= self_text_len`) 수를 구해 `8 × n`을 가산. 이로써 2차 병합
   (right_half) 시 컨트롤 갭이 char_offsets에 자동 인코딩되어, 렌더링
   (`control_text_positions`)과 HWP5 직렬화(`serialize_para_text` 갭 분석)가
   병합 지점에 컨트롤을 복원한다.
3. **controls/ctrl_data_records/control_mask 병합 (신규 5-2절)**:
   `ctrl_data_records[i] ↔ controls[i]` 인덱스 정렬을 위해 self 쪽을 None 패딩 후
   other를 이어붙임. `control_mask`는 OR 병합. other에 컨트롤이 없으면 기존 동작
   유지(불필요 패딩 없음).
4. **field_ranges ctrl_offset**: 병합 전 `self.controls.len()` 캡처 위치를 5-2절
   앞으로 유지 — 기존 보정 의미 불변 (테스트로 고정).
5. **char_count**: `텍스트 + 8 × 병합 후 controls.len() + 1`로 갱신. `split_at`의
   `ctrl_code_units` 계산 및 HWPX 직렬화의 컨트롤 수 역산
   (`serializer/hwpx/section.rs:379`)과 정합.
6. **has_para_text**: 병합 후 텍스트/컨트롤이 있으면 `true` (split_at L682-687
   역방향 정합).

## 4. 추가된 단위 테스트

| 테스트 | 검증 내용 |
|--------|----------|
| `test_merge_from_empty_text_with_control` | `text=""+controls` 병합 → 컨트롤·mask·char_count(11)·has_para_text 보존 |
| `test_merge_from_control_then_right_half` | paste 3단 흐름(split→merge컨트롤→merge right_half) → `control_text_positions()==[2]`(커서 위치), char_offsets `[0,1,10,11]` 갭 인코딩 |
| `test_merge_from_ctrl_data_alignment` | self None 패딩 + other CTRL_DATA 인덱스 정렬 `[None, Some([1,2,3])]` |
| `test_merge_from_text_with_mid_control` | 양쪽 중간 갭 컨트롤 → 위치 `[1,3]` 보존, char_count 21 |
| `test_merge_from_field_ranges_ctrl_offset` | other field_ranges의 control_idx +1 보정 (병합 전 캡처 고정) |

구현 전 실측: 신규 5건 중 4건 실패(컨트롤 드롭 재현), `field_ranges_ctrl_offset`은
기존 동작 고정용으로 통과. 구현 후 5건 전체 통과.

## 5. 검증

통과:

- `cargo test --lib model::paragraph::tests` — 50 passed (기존 merge_from/split
  round-trip 테스트 무회귀 포함)
- `cargo test` 전체 — **1627 passed, 0 failed**, 6 ignored + 통합 테스트 전부 통과
  (merge_from 호출처: 백스페이스 병합, deleteRange, 각주/머리말, HTML import,
  본문/셀 paste 경로 회귀 없음)
- `cargo clippy --all-targets` — 무경고
- `cargo fmt --check` (변경 파일 한정) — 통과

## 6. 비고

- char_count 의미 변화(컨트롤당 8 code unit 포함)는 파서가 생성하는 char_count와
  동일한 의미론으로의 정합이며, 전체 테스트에서 부작용 미검출.
- Stage 2에서 셀/글상자/중첩 셀 paste round-trip 및 백스페이스 병합 컨트롤 보존
  통합 테스트로 end-to-end 검증 예정.
