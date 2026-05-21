# Task #1058 구현 계획서 — 글상자 LIST_HEADER 13 byte 한컴 contract 정합

- 이슈: [#1058](https://github.com/edwardkim/rhwp/issues/1058)
- 수행 계획서: [task_m100_1058.md](task_m100_1058.md)
- 브랜치: `local/task1058`
- 일시: 2026-05-21
- 채택 결정: B (13 byte 의미 완전 규명) + B (광범위 sweep)

## 1. 4 단계 구현 계획

### Stage 1 — 13 byte 의미 완전 규명 (`hwplib` 권위 + 다중 fixture)

**목표**: 글상자 LIST_HEADER 의 추가 13 byte 필드 의미 완전 식별.

**수행**:
1. **hwplib 권위 참조** (`/home/edward/vsworks/shwp/hwplib`):
   - `ListHeaderForGso` 또는 `ListHeaderForTextBox` 클래스 찾기
   - `ForListHeaderForGso` reader / writer 분석
   - 13 byte 의 각 필드 (name, type, default value) 정확화
2. **다중 fixture raw byte 비교**:
   - `samples/footnote-tbox-01.hwp` (본 sample, 1 글상자)
   - `samples/table-in-tbox.hwp` (글상자 + 안 표 다수)
   - `samples/pic2.hwp` 또는 다른 글상자 보유 fixture
   - 각 fixture 의 글상자 LIST_HEADER raw byte 추출 + 13 byte 패턴 비교
3. **본문 paragraph 의 13 byte 와 비교** — 본문 paragraph 의 LIST_HEADER 도 같은 형식인지 차이 분석

**산출물**:
- `mydocs/working/task_m100_1058_stage1.md` — 13 byte 필드 map + 다중 fixture 패턴 비교

### Stage 2 — Model / Parser / Serializer 정정

**model 변경** (`src/model/shape.rs` 의 `TextBox`):
- Stage 1 결과에 따라 필드 추가
- 또는 기존 `raw_list_header_extra` (이미 존재) 보존 + serializer 가 정합 사용

**parser 변경** (`src/parser/control/shape.rs` 또는 `src/parser/body_text.rs` 의 LIST_HEADER 파싱):
- 13 byte 추가 파싱 + raw_list_header_extra 보존 (이미 보존 시 정합 확인만)

**serializer 변경** (`src/serializer/control.rs::serialize_text_box_if_present`):
```rust
fn serialize_text_box_if_present(drawing: &DrawingObjAttr, level: u16, records: &mut Vec<Record>) {
    if let Some(ref text_box) = drawing.text_box {
        let mut w = ByteWriter::new();
        w.write_u32(text_box.paragraphs.len() as u32).unwrap();
        w.write_u32(text_box.list_attr).unwrap();
        w.write_i16(text_box.margin_left).unwrap();
        w.write_i16(text_box.margin_right).unwrap();
        w.write_i16(text_box.margin_top).unwrap();
        w.write_i16(text_box.margin_bottom).unwrap();
        w.write_u32(text_box.max_width).unwrap();
        // 라운드트립 보존
        if !text_box.raw_list_header_extra.is_empty() {
            w.write_bytes(&text_box.raw_list_header_extra).unwrap();
        } else {
            // [Task #1058] HWPX 출처: 한컴 default 13 byte (모두 zero)
            // Stage 1 결과 — list_id/numbering/level 등 추가 필드의 default value
            w.write_bytes(&[0u8; 13]).unwrap();
        }
        records.push(Record {
            tag_id: tags::HWPTAG_LIST_HEADER,
            level,
            size: 0,
            data: w.into_bytes(),
        });
        serialize_paragraph_list(&text_box.paragraphs, level, records);
    }
}
```

**산출물**:
- 코드 변경
- 정량 입증: hwp5-inventory-diff 의 LIST_HEADER tuple=0 size 33→33 정합 + raw byte 동일
- `mydocs/working/task_m100_1058_stage2.md`

### Stage 3 — 회귀 가드 + 광범위 sweep + WASM

**회귀 가드** `tests/issue_1058_textbox_list_header.rs`:
- 글상자 LIST_HEADER size=33 단언
- 한컴 정답지 raw byte 정합
- 본문 paragraph LIST_HEADER 회귀 부재 (size 비교)
- Task #1050 회귀 가드 7/7 양립

**광범위 sweep fixture** (작업지시자 선택 B 광범위):
- 글상자 보유: `samples/footnote-tbox-01.hwpx` + `samples/table-in-tbox.hwp` + 다른 글상자 fixture
- 변환본 9종 (HWP3→HWP5 변환본): hwp3-sample/10/11/13/14/16/19-hwp5
- 일반 fixture: aift / KTX / biz_plan / exam_kor / footnote-01 / 2010-01-06

**자동 검증**: cargo test --lib + --tests + clippy + fmt + WASM Docker 빌드.

**산출물**:
- `tests/issue_1058_textbox_list_header.rs`
- `output/poc/issue_1058/{before,after}/`
- `mydocs/working/task_m100_1058_stage3.md`

### Stage 4 — 한컴 시각 판정 + merge + close + orders + archives

- WASM + rhwp-studio 동기화
- 작업지시자 한컴 한글 2020 직접 검증:
  - footnote-tbox-01.hwpx → HWP 저장 → 한컴 열기 → 신규 각주 추가
  - 다단계 목록 "1.1.1.1.1.1." **부여 안 됨** 확인
  - Task #1050 통과 영역 회귀 부재 (각주 영역 정상 조판 유지)
- 최종 보고서 + no-ff merge → devel push + close #1058
- archives + orders 갱신

## 2. 위험 / 완화

| 위험 | 완화 |
|------|------|
| hwplib 의 글상자 LIST_HEADER 클래스 미발견 또는 형식 다름 | 다중 fixture raw byte 비교로 패턴 추출 + raw 보존 fallback |
| 13 byte 모두 zero 가 한컴 default 아니고 fixture별 다름 | Stage 1 다중 fixture 비교로 default value 확정 |
| 본문 paragraph LIST_HEADER 와 형식 차이 가능성 | parser/serializer 의 caller 분기 명확화 (text_box only) |
| 광범위 sweep 결과 변환본/일반 fixture 회귀 발생 | raw_list_header_extra 보존 우선 → HWP 출처는 기존 raw 그대로 (회귀 부재) |
| 한컴 다단계 목록 본질이 13 byte 외 다른 영역 (PARA_HEADER #0 break_type 등) 일 수 있음 | Stage 4 한컴 재검증으로 정확화 — 부분 통과 시 추가 진단 |

## 3. 비범위 / 후속

- 본문 PARA_HEADER #0 의 break_type/num_char_shapes 차이
- FOOTNOTE_SHAPE tuple=2 (endnote shape) 1건 잔여
- 광범위 HWPX→HWP 호환 (Task #178 영역)

## 4. 작업지시자 결정 요청

| 결정 | 옵션 |
|------|------|
| 본 구현 계획 승인 | A. 승인 / B. 수정 |
