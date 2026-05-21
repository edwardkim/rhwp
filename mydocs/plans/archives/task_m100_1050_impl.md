# Task #1050 구현 계획서 — [hwpx2hwp] 각주/미주 저장 정정

- 이슈: [#1050](https://github.com/edwardkim/edward/rhwp/issues/1050)
- 수행 계획서: [task_m100_1050.md](task_m100_1050.md)
- 브랜치: `local/task1050`
- 일시: 2026-05-21
- 채택 결정: B (Footnote + Endnote 동시) + C (의미 규명 우선)

## 1. 5 단계 계획

### Stage 1 — 정밀 진단 (의미 규명)

**목표**: CTRL_FOOTNOTE / CTRL_ENDNOTE 의 추가 14 byte payload + LIST_HEADER (각주 안) 추가 10 byte 의미 규명. HWP5 spec / pyhwpx / hwp2hwpx 참조 + 다중 fixture raw byte 비교.

**수행**:
- 본 sample (`samples/footnote-tbox-01.hwp`) 의 footnote payload 2 케이스 (글상자 안 + 본문) 비교
- `samples/footnote-01.hwp` (본문 직속 footnote 다수) 의 payload 패턴 추출
- `samples/2010-01-06.hwp` (multi-paragraph footnote 5개) 의 패턴 추출
- HWP5 스펙 문서 / pyhwpx + hwp2hwpx 참조 — footnote CTRL 의 field 구조 확인
- Endnote 보유 sample 검색 + 동일 분석
- LIST_HEADER 짧은 형식 (6 byte) vs footnote 형식 (16 byte) 차이 분석

**산출물**:
- `mydocs/working/task_m100_1050_stage1.md` — payload field map (각 byte 의 의미 + 값 분포)
- 모델 확장 결정 (raw_extra 보존 vs 구조 필드)

### Stage 2 — Model / Parser / Serializer 정정

**model 변경** (`src/model/footnote.rs`):
```rust
pub struct Footnote {
    pub number: u16,
    pub paragraphs: Vec<Paragraph>,
    // [Task #1050] CTRL_FOOTNOTE payload 추가 영역 (한컴 정답지 14 byte 보존)
    // Stage 1 진단 결과에 따라 구조 필드 또는 raw 보존
    pub raw_ctrl_extra: Vec<u8>,
    // [Task #1050] LIST_HEADER (footnote 안) 추가 영역 (한컴 정답지 10 byte 보존)
    pub raw_list_header_extra: Vec<u8>,
}
// Endnote 동일 변경
```

**parser 변경** (`src/parser/control.rs`):
```rust
fn parse_footnote_control(ctrl_data: &[u8], child_records: &[Record]) -> Control {
    let mut footnote = Footnote::default();
    if ctrl_data.len() >= 2 {
        let mut r = ByteReader::new(ctrl_data);
        footnote.number = r.read_u16().unwrap_or(0);
        // [Task #1050] 나머지 14 byte 보존
        if ctrl_data.len() > 2 {
            footnote.raw_ctrl_extra = ctrl_data[2..].to_vec();
        }
    }
    footnote.paragraphs = find_list_header_paragraphs(child_records);
    // LIST_HEADER 의 추가 10 byte 도 추출 보존 — find_list_header_paragraphs 확장 필요
    Control::Footnote(Box::new(footnote))
}
```

**serializer 변경** (`src/serializer/control.rs`):
```rust
fn serialize_footnote(fn_: &Footnote, level: u16, records: &mut Vec<Record>) {
    let mut w = ByteWriter::new();
    w.write_u16(fn_.number).unwrap();
    // [Task #1050] 추가 14 byte 직렬화 (raw 보존 + HWPX default)
    if !fn_.raw_ctrl_extra.is_empty() {
        w.write_bytes(&fn_.raw_ctrl_extra).unwrap();
    } else {
        // HWPX 출처: 한컴 default payload
        w.write_bytes(&FOOTNOTE_DEFAULT_CTRL_EXTRA).unwrap();
    }
    records.push(make_ctrl_record(tags::CTRL_FOOTNOTE, level, w.as_bytes()));
    // footnote LIST_HEADER 는 16 byte 형식 (raw_list_header_extra 포함)
    serialize_footnote_list_header_with_paragraphs(&fn_.paragraphs, &fn_.raw_list_header_extra, level + 1, records);
}
```

**산출물**:
- 코드 변경 (model + parser + serializer)
- `mydocs/working/task_m100_1050_stage2.md` — Stage 2 정량 입증 (hwp5-inventory-diff = 0 또는 footnote 차이 0)

### Stage 3 — 회귀 가드 + 광범위 sweep + WASM

**회귀 가드** `tests/issue_1050_footnote_serialize.rs`:
- footnote 라운드트립 후 footnote 본문 보존 (글상자 안 + 본문 양쪽)
- HWP→HWP 회귀 부재 (footnote-01 / 2010-01-06)
- Endnote 보유 sample 라운드트립 (있으면)

**sweep fixture** (작업지시자 선택 A 기본 범위):
- `samples/hwpx/footnote-tbox-01.hwpx` + `samples/footnote-tbox-01.hwp` (본 sample)
- `samples/footnote-01.hwp` (본문 footnote)
- `samples/2010-01-06.hwp` (multi-paragraph footnote)
- `samples/table-in-tbox.hwp` (글상자 컨테이너 회귀 부재)
- 일반 fixture (aift / KTX / biz_plan / exam_kor)

**자동 검증**: cargo test --lib + --tests + clippy + fmt + WASM Docker 빌드.

**산출물**:
- `tests/issue_1050_footnote_serialize.rs`
- `output/poc/issue_1050/{before,after}/` 사진본
- `mydocs/working/task_m100_1050_stage3.md`

### Stage 4 — 한컴 시각 판정 + 머지 + close + orders + archives

- WASM + rhwp-studio 동기화
- 작업지시자 한컴 2020 직접 열기 + 각주 출력 정합 확인
- 최종 보고서 + no-ff merge + push + close #1050
- archives 이동 (계획서 / 검토 문서)

## 2. 위험 / 완화

| 위험 | 완화 |
|------|------|
| Stage 1 의미 규명 시간 소요 | HWP5 spec / pyhwpx 참조로 시간 단축. 못 풀어도 raw 보존 fallback |
| HWPX 출처 IR 에 14 byte 정보 부재 → default 필요 | 한컴 정답지의 가장 일반적 default 패턴 추출 (`samples/footnote-01.hwp` 등 다중 fixture 평균) |
| LIST_HEADER 짧은 형식이 다른 컨테이너와 공유 | find_list_header_paragraphs / serialize_list_header_with_paragraphs 의 caller 가 footnote 인 경우만 16 byte 형식 분기 |
| Endnote 의 본질이 Footnote 와 다를 가능성 | Stage 1 에서 양쪽 동시 분석. 다르면 별도 분기 |
| 기존 HWP 출처 footnote 회귀 가능 | raw_ctrl_extra 가 비어있을 때만 default 적용 → HWP 출처는 기존 raw 그대로 보존 |

## 3. 작업지시자 결정 요청

| 결정 | 옵션 |
|------|------|
| 본 구현 계획 승인 | A. 승인 / B. 단계/접근 수정 / C. 보류 |
| Stage 1 진단 시간 한도 | A. 자율 (의미 규명 우선) / B. 30분 / 1시간 등 명시 |
