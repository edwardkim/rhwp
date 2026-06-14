# 구현 계획서 — Task M100-1323

- 이슈: https://github.com/edwardkim/rhwp/issues/1323
- 수행계획서: `mydocs/plans/task_m100_1323.md`
- 브랜치: `local/task1323`

## 1. 구현 방향

`Paragraph::merge_from`(`src/model/paragraph.rs:716`)을 model 계층에서 보강하여
컨트롤 보유 문단 병합 시 `controls`/`ctrl_data_records`/`control_mask`를 보존한다.
클립보드/편집 명령 계층(`document_core/commands/`)은 수정하지 않는다(라우팅은 기존
그대로 동작). 프론트엔드도 수정하지 않는다.

### 설계 근거 (직렬화·렌더링 계약)

병합 후 문단이 올바르게 렌더링·직렬화되려면 다음 계약을 지켜야 한다 (코드 확인 완료):

1. **컨트롤 위치 = char_offsets 갭**: 렌더링(`control_text_positions()`,
   `paragraph.rs:856`)과 HWP5 직렬화(`serialize_para_text`, `body_text.rs:485`
   `while prev_end + 8 <= offset`) 모두 인라인 컨트롤 위치를 char_offsets의
   8 code unit 갭으로 복원한다. 텍스트 뒤(trailing) 컨트롤은 갭 없이 문단 끝에
   배치된다(`body_text.rs:547`, `paragraph.rs:932` 폴백).
2. **char_count는 컨트롤당 8 유닛 포함**: `split_at`은 이미
   `split_pos + controls.len()*8 + 1`로 계산(`paragraph.rs:678-679`)하며, HWPX
   직렬화는 `(char_count - 1 - text_units) / 8`로 컨트롤 수를 역산
   (`serializer/hwpx/section.rs:379`). 현 merge_from은 컨트롤 유닛 미반영(잠재
   불일치) — 본 수정에서 정합화한다.
3. **ctrl_data_records[i] ↔ controls[i] 인덱스 정렬**: HWP5 직렬화가
   `ctrl_data_records.get(ctrl_idx)`로 소비(`body_text.rs:282-288`). 길이가
   짧으면 None 취급되므로, 병합 시 self 쪽을 None 패딩 후 other를 이어 붙여
   정렬을 유지한다.
4. **control_mask**: HWP5 직렬화는 재계산(`compute_control_mask`,
   `body_text.rs:302`)하지만, `hwpx_to_hwp.rs:952` 등 model 소비처가 있으므로
   OR 병합으로 일관성을 유지한다.

### 병합 위치 의미론

paste 흐름(`split_at(char_offset)` → left.merge_from(클립 문단) →
last.merge_from(right_half))에서:

- **1차 병합** (left + 컨트롤 문단): 컨트롤은 left 텍스트 끝의 trailing 컨트롤이
  된다 (갭 인코딩 불필요 — 뒤따르는 텍스트가 없음).
- **2차 병합** (위 결과 + right_half): `utf16_end` 계산이 **trailing 컨트롤의
  8유닛을 포함**해야 right_half의 char_offsets가 컨트롤 갭 뒤로 이동되어, 갭
  분석이 컨트롤을 병합 지점(커서 위치)에 복원한다.

trailing 컨트롤 수는 `control_text_positions()`에서 `position >= self_text_len`인
컨트롤 수로 구한다 (중간 컨트롤의 갭은 이미 char_offsets에 인코딩되어 있어 마지막
문자 offset에 반영됨). self/other 양쪽에 boundary 컨트롤이 있어도 controls 배열
순서(self → other)와 갭 분석의 순차 배정이 일치하므로 정합하다.

## 2. 세부 단계

### Stage 1 — `merge_from` 컨트롤 병합 보강 + 단위 테스트

**파일**: `src/model/paragraph.rs`, `src/model/paragraph/tests.rs`

`merge_from` 수정 (기존 단계 번호 기준):

1. **early-return** (L717-719):
   `if other.text.is_empty() && other.controls.is_empty() { return ...; }`
2. **utf16_end 계산 보강** (L724-730): 기존 계산값에
   `8 × (self의 trailing 컨트롤 수)` 가산.
   `let trailing = self.control_text_positions().iter().filter(|&&p| p >= self_text_len).count()`
3. **(기존 유지)** 텍스트/char_offsets/char_shapes/line_segs/range_tags 결합 —
   other의 offsets에 보강된 utf16_end가 가산되므로 컨트롤 갭이 자동 인코딩됨.
4. **field_ranges 결합** (L791-799): `ctrl_offset = self.controls.len()` 캡처는
   **controls 병합 전** 위치 유지 (현 위치 그대로 두고 controls 병합을 그 뒤에
   배치).
5. **controls / ctrl_data_records / control_mask 병합** (신규):
   ```rust
   if !other.controls.is_empty() {
       // 인덱스 정렬: self 쪽 None 패딩 후 other 이어붙이기
       while self.ctrl_data_records.len() < self.controls.len() {
           self.ctrl_data_records.push(None);
       }
       for i in 0..other.controls.len() {
           self.ctrl_data_records
               .push(other.ctrl_data_records.get(i).cloned().flatten());
       }
       self.controls.extend(other.controls.iter().cloned());
       self.control_mask |= other.control_mask;
   }
   ```
   (other에 컨트롤이 없으면 기존 동작 — ctrl_data_records 불필요 패딩 안 함)
6. **char_count** (L802): 텍스트 part 의미는 기존 유지(chars().count()),
   `+ 8 × self.controls.len()`(병합 후 총수) 가산 — split_at L678-679와 정합.
7. **has_para_text**: 병합 후 텍스트 또는 컨트롤이 있으면 `true`로 갱신
   (split_at L682-687의 역방향 정합).

**단위 테스트** (`src/model/paragraph/tests.rs`, 기존 merge_from 테스트군 L476~ 옆):

- `merge_from_empty_text_with_control`: `text=""+controls=[Picture]` 병합 →
  controls 1개 보존, char_count = self_text + 8 + 1, has_para_text 유지
- `merge_from_control_then_right_half`: paste 3단 흐름 재현(split→merge컨트롤→
  merge right_half) → `control_text_positions() == [커서 위치]`, right 텍스트
  offsets가 +8 시프트
- `merge_from_ctrl_data_alignment`: self 컨트롤(레코드 없음) + other 컨트롤
  (CTRL_DATA 있음) 병합 → `ctrl_data_records[i]` 정렬 검증
- `merge_from_text_with_mid_control`: 중간 갭 컨트롤 보유 문단끼리 병합 →
  위치 보존
- `merge_from_field_ranges_ctrl_offset`: other의 field_ranges control_idx 보정이
  컨트롤 병합과 함께 정확한지
- 기존 텍스트 전용 테스트(`test_merge_from_basic/different_styles/empty`) 무회귀

### Stage 2 — 셀/글상자 paste 경로 통합 테스트 + 주석 갱신

**파일**: `src/wasm_api/tests.rs`, `src/document_core/commands/object_ops.rs`
(필요 시 `clipboard.rs` 보정)

- round-trip 통합 테스트 (기존 paste 테스트군 `wasm_api/tests.rs:2064~` 옆):
  - 본문 그림 `copy_control_native` → 표 셀 `paste_internal_in_cell_native` →
    셀 문단에 Picture 컨트롤 + ctrl_data 보존 검증
  - 글상자(Shape text_box) 동일 검증 — `object_ops.rs:8776`
    `paste_text_into_textbox` 옆에 이미지 버전 추가, L8772-8774 "별개 결함"
    주석을 해소 사실로 갱신
  - 중첩 셀 `paste_internal_in_cell_by_path_native` 동일 검증
- 부수 해소 잠재 결함 회귀 테스트:
  - `merge_paragraph_native`(본문 백스페이스 병합): 컨트롤 보유 문단 병합 시
    컨트롤 보존
  - `merge_paragraph_in_cell_native`(셀 백스페이스 병합): 동일
- 본문 무회귀: 기존 `paste_control_native` 테스트(L2064-2082),
  `paste_internal_native` 텍스트 테스트(L1890-1946) 통과 확인
- HWP5 저장 round-trip: 병합 문단을 포함한 문서 serialize→parse 시 컨트롤
  보존(셀 paste 후 저장 시나리오) — 기존 re_sample/serializer 테스트 패턴 활용
- 필요 시 보정: `reflow_cell_paragraph`가 셀 문단 line_segs에 컨트롤 치수를
  반영하지 못하면 paste 경로에서 보정 (Stage 2에서 실측 후 결정)

### Stage 3 — 전체 검증 + 시각 확인 + 최종 보고서

- `cargo test` 전체 (merge_from 호출처 광범위 — 백스페이스/HTML import/각주/
  머리말 경로 회귀 확인)
- `cargo clippy`
- `docker compose --env-file .env.docker run --rm wasm` → rhwp-studio에서 수동
  검증:
  - 본문 이미지 복사 → 글상자 안 붙여넣기 → 이미지 렌더 확인
  - 본문 이미지 복사 → 표 셀 안 붙여넣기 → 이미지 렌더 확인
  - 텍스트 붙여넣기(본문/셀/글상자), 본문 이미지 붙여넣기(pasteControl) 무회귀
  - 셀 안 백스페이스 문단 병합 시 그림 보존 확인
- 떠 있는 개체(treat_as_char=false) 셀 anchor 렌더링 시각 확인 (리스크 항목)
- 최종 보고서 `mydocs/report/task_m100_1323_report.md` 작성

## 3. 범위 밖 관찰 (별도 이슈 후보)

- `merge_from`이 `tab_extended`를 병합하지 않음 — other 텍스트에 탭이 있으면
  탭 확장 데이터(너비/종류) 소실. 본 타스크 범위 밖, 최종 보고서에 기록.
- 프론트 `clipboard_has_control_native`가 클립보드 첫 문단만 검사 — 다중 문단
  클립보드의 컨트롤 처리(본문 pasteControl이 첫 문단만 붙여넣는 문제 포함)는
  별도 이슈 영역.

## 4. 완료 기준

- 글상자/표 셀(중첩 포함) 이미지 copy→paste에서 그림 컨트롤·CTRL_DATA 보존
- `control_text_positions()`가 병합 지점에 컨트롤 위치 복원
- HWP5/HWPX 직렬화 정합 (char_count 역산 포함)
- `cargo test` 전체 통과, clippy 무경고
- 작업지시자 시각 판정 통과
