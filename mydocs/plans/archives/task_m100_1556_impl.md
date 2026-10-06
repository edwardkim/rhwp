# 구현계획서 — Task #1556

다단락 누름틀 필드의 고아 `<hp:fieldEnd>` 8유닛 슬롯 소실 수정 (HWPX serializer).

근거: 수행계획서 `task_m100_1556.md` §2 (근본 원인 확정).

## 설계 요지
- begin/end 가 다른 문단인 다단락 필드의 **end 문단**에서, 고아 fieldEnd 가
  `\u{0004}`(8유닛)만 차지하고 IR 산출물(Control·FieldRange)이 없어 직렬화기가 소실.
- **해결**: `Paragraph` IR 에 고아 fieldEnd 를 기록 → 파서가 폐기 대신 기록 →
  직렬화기가 해당 위치에 `<hp:fieldEnd>` 방출(8유닛 소비).

## 핵심 정합 포인트 (직렬화기 슬롯 카운팅)
`render_runs` 의 `inferred_control_slot_count(para)` 는 char_offset 갭에서 추정한
슬롯 수에서 `field_ranges.len()`(컨트롤 없는 fieldEnd 슬롯)을 차감한다. 고아 fieldEnd
도 **컨트롤 없는 8유닛 슬롯**이므로 동일하게 `orphan_field_ends.len()` 을 차감해야
`slot_count == slots.len()` 메인 경로로 진입한다. (미차감 시 mismatch 경로로 빠져
현 버그가 유지됨 — para 0.16 은 현재 from_offsets=1, slots=0 → mismatch.)

---

## 단계 1 — IR 모델 + 파서 기록

### 1.1 `src/model/paragraph.rs`
- 신규 구조체:
  ```rust
  #[derive(Debug, Default, Clone)]
  pub struct OrphanFieldEnd {
      /// text 문자열 내 위치 (이 인덱스 직전에 8유닛 fieldEnd 슬롯이 놓임)
      pub char_idx: usize,
      pub begin_id_ref: u32,
      pub field_id: u32,
  }
  ```
- `Paragraph` 에 `pub orphan_field_ends: Vec<OrphanFieldEnd>` 추가
  (Default 파생으로 빈 벡터 기본값 — 기존 `..Default::default()` 생성부 무영향).

### 1.2 `src/parser/hwpx/section.rs`
- `parse_ctrl` 의 `b"fieldEnd"` 분기: 현재 `skip_element` 으로 버리는
  `beginIDRef`/`fieldid` 속성을 포착해 발생 순서대로 `field_end_attrs: Vec<(u32,u32)>`
  에 push (parse_ctrl 시그니처에 파라미터 1개 추가, 호출부 1곳 갱신).
- `visible_char_idx` 루프(현 600~628행)에서 `"\u{0004}"` 처리 시:
  - fieldEnd 카운터로 `field_end_attrs` 인덱싱.
  - `field_stack.pop()` 이 `Some` 이면 기존대로 `FieldRange` 생성(동일 문단 필드).
  - `None`(고아)이면 `OrphanFieldEnd { char_idx: visible_char_idx, begin_id_ref, field_id }`
    를 `para.orphan_field_ends` 에 push.
- `\u{0004}` 의 8유닛 가산(visual_text/char_offsets 조립 루프, 현 639행)은 불변.

### 1.3 단위 테스트 (파서)
- begin 문단 + end 문단 분리된 합성 section XML → end 문단의
  `orphan_field_ends` 위치·attrs·`char_count`(텍스트+8) 검증.
- 동일 문단 begin+end 는 종전대로 `field_ranges` 로만 처리(고아 0) 회귀 가드.

**커밋**: `Task #1556 Stage1: 고아 fieldEnd IR 기록 (파서)` + `_stage1.md`

---

## 단계 2 — 직렬화기 방출

### 2.1 `src/serializer/hwpx/section.rs`
- `inferred_control_slot_count`: 반환식에서 `field_ranges.len()` 과 함께
  `orphan_field_ends.len()` 도 `saturating_sub`.
- `render_runs`:
  - fast-path 조건에 `&& para.orphan_field_ends.is_empty()` 추가.
  - 메인 루프에서 기존 fieldEnd(field_ranges) 방출과 동형으로, 각 문자 idx 에서
    `orphan_field_ends` 중 `char_idx == idx`(pre-char) 항목을 8유닛 슬롯으로 방출.
  - text 끝(루프 후) 위치의 고아(`char_idx == text.chars().count()`)는 post-loop
    블록(현 601~607행 인근)에서 방출. ← para 0.16 케이스.
  - 방출 순서: 동일 위치에서 슬롯/fieldEnd 와의 순서는 원본 XML(run 말미 fieldEnd)
    재현 기준으로 텍스트 직후·다음 run 경계 앞.

### 2.2 `src/serializer/hwpx/field.rs`
- `write_field_end` 가 `beginIDRef` 만 방출 → `fieldid` 동반 방출 변형 추가
  (`write_field_end_full(w, begin_id_ref, field_id)` 또는 인자 확장).
  원본 `<hp:fieldEnd beginIDRef=".." fieldid="..">` 속성 정합.

### 2.3 단위 테스트 (직렬화기)
- `orphan_field_ends` 가진 `Paragraph` → 직렬화 → `<hp:fieldEnd beginIDRef=..>`
  존재·위치 검증, 재parse char_count 정합(IR diff=0).

**커밋**: `Task #1556 Stage2: 고아 fieldEnd 직렬화 방출` + `_stage2.md`

---

## 단계 3 — 합성 roundtrip + 실문서 회귀 가드

### 3.1 합성 회귀 (둘 다 — 합성)
- 단계 1·2 단위 테스트로 parser/serializer 양측 합성 커버.
- 추가: 다단락 fieldEnd 최소 HWPX 합성 → parse→serialize→parse IR diff=0 테스트
  (`src/serializer/hwpx/roundtrip.rs` 또는 section 단위 테스트).

### 3.2 실문서 샘플 추가 (둘 다 — 실문서)
- 대표 실문서 `dt2854`(≈39KB, diff=1 단일 결함)를 `samples/hwpx/` 에 추가.
  파일명: 영문 식별자 부여(예: `field-multipara-clickhere.hwpx`) — 한글 장문 회피,
  의미 명시. (서울 열린데이터 = 공개 행정문서.)
- `cargo test --test hwpx_roundtrip_baseline` 자동 포함 → 수정 후 diff=0 통과 확인.

**커밋**: `Task #1556 Stage3: 합성 roundtrip + 실문서 회귀 샘플` + `_stage3.md`

---

## 단계 4 — 전수 검증 + 최종 보고

### 4.1 검증
- `rhwp ir-diff` 로 코퍼스 reproducer 다건(dt2854/dt2906/dt2952/dt3004/…) diff=0 확인.
- 2023-01-05 폴더 재스캔: IR_DIFF 24건 → 0건(또는 잔여 패턴 분리 보고).
- `cargo test --test hwpx_roundtrip_baseline` (전수, 신규 샘플 포함) 무회귀.
- 전체 `cargo test` 무회귀 (레이아웃/미주 회귀 가드 — `feedback_full_cargo_test_before_pr`).
- 수정 파일 한정 `cargo fmt`·`cargo clippy` 점검.

### 4.2 보고
- `task_m100_1556_report.md` 작성 (결과·검증 수치·잔여사항).

**커밋**: `Task #1556 Stage4: 전수 검증 + 최종 보고서` + `_report.md`

---

## 리스크 / 주의
- **slot 카운팅 정합**: §핵심 정합 포인트의 `orphan_field_ends.len()` 차감 누락 시
  mismatch 경로 유지로 무효과 — 단계 2 필수 검증 항목.
- **begin 문단 무영향**: begin 문단은 `Control::Field` 보존 경로 불변(고아 기록은 end 문단
  한정). 스퓨리어스 fieldEnd 방출 없음 — ir-diff 로 확인.
- **HWP5/HWP3 무관**: HWPX 전용. `OrphanFieldEnd` 는 HWPX 파서만 채움(다른 파서는 빈 채로).
- 신규 샘플이 이 결함 외 다른 roundtrip 결함을 보유하지 않음을 단계 3 에서 확인
  (dt2854 는 현재 diff=1 = 본 결함 단일).
