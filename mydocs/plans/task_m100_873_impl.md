# Task #873 구현계획서

**선행**: `mydocs/plans/task_m100_873.md` (수행계획서, 승인 완료)
**브랜치**: `local/task873`
**작성일**: 2026-05-13

## 단계 분해 (3 단계 + 승인 게이트)

### Stage A: 본질 정밀 진단

**A.1 hwp3-sample10-hwp5.hwp 의 DocInfo BinData dump**:

- 임시 debug print 또는 dump 확장으로 DocInfo BinData list 출력
- 각 entry 의 `data_type`, `storage_id`, `abs_path`, `rel_path`, `extension` 확인
- BinDataType::Link 의 abs_path 가 `C:\Users\admin\Desktop\hwp3sample\rdb02.gif` 등으로 매칭됨을 확인

**A.2 Picture `bin_data_id` ↔ BinData index 매핑 검증**:

- `rhwp dump` 출력에서 각 Picture 의 `bin_data_id` 확인 (이미 bin_id=1, 2, 3)
- DocInfo BinData list 의 같은 index 와 매칭 (1-based or 0-based 확인)
- HWP5 spec 의 BinData index 규약 검증

**A.3 정정 후보 위치 결정**:

- 후보 1: `src/parser/control/shape.rs` `parse_picture` 후속에서 BinData lookup
- 후보 2: `src/parser/mod.rs` 의 document 조립 직후 post-process loop
- 후보 3: `load_bin_data_content_lenient` 의 `BinDataType::Link` 분기를 Link entry 도 BinDataContent 에 placeholder 로 push (단, data 는 empty)

후보 1/2 가 본질 정합 — Picture 의 external_path 설정이 목표.

**산출**: `mydocs/working/task_m100_873_stage_a.md`
- BinData dump 결과
- 정정 후보 결정
- 회귀 영향 분석 (HWP5 native Embedding/Storage 무영향 확인)

### Stage B: 정정 구현 + 검증

**B.1 정정 구현**:

가장 깔끔한 위치 — `src/parser/mod.rs` 의 document 조립 직후, `assign_auto_numbers` 와 동일 레벨에 후속 함수 `populate_link_image_paths` 추가:

```rust
fn populate_link_image_paths(doc: &mut Document) {
    use crate::model::control::Control;
    use crate::model::shape::ShapeObject;
    use crate::model::bin_data::BinDataType;

    let bin_data = &doc.doc_info.bin_data;
    for section in &mut doc.sections {
        for para in &mut section.paragraphs {
            for ctrl in &mut para.controls {
                let pic = match ctrl {
                    Control::Picture(p) => p,
                    Control::Shape(s) => match s.as_mut() {
                        ShapeObject::Picture(p) => p,
                        _ => continue,
                    },
                    _ => continue,
                };
                let bin_idx = (pic.image_attr.bin_data_id as usize).saturating_sub(1);
                if let Some(bd) = bin_data.get(bin_idx) {
                    if matches!(bd.data_type, BinDataType::Link)
                        && pic.image_attr.external_path.is_none() {
                        let path = bd.abs_path.clone()
                            .or_else(|| bd.rel_path.clone());
                        if let Some(p) = path {
                            pic.image_attr.external_path = Some(p);
                        }
                    }
                }
            }
        }
    }
}
```

`parse_from_bytes` 의 `assign_auto_numbers(&mut doc);` 다음에 `populate_link_image_paths(&mut doc);` 호출.

**B.2 검증**:

1. cargo build --release
2. cargo test --release --lib (1246 passed 회귀 0)
3. hwp3-sample10-hwp5.hwp page 1:
   - `rhwp export-svg samples/hwp3-sample10-hwp5.hwp -p 0 -o /tmp/h5/`
   - `grep -c 'data:image' /tmp/h5/*.svg` = 3 (image 3개 정상 임베드)
   - PNG 변환 후 시각 확인 (oracle.gif + 다이어그램 + s1.jpg 표시)
4. hwp3-sample10.hwp (HWP3 원본) 회귀 0
5. 다른 HWP5 sample 회귀 0 (특히 native HWP5 의 Embedding 케이스)
6. cargo clippy --release --lib 경고 0

**B.3 회귀 우려 영역**:

- HWP5 native sample 의 BinDataType::Embedding 케이스: `external_path` 미설정 (matches!) → 영향 없음
- 다른 sample 의 Link 타입 (만약 있다면): 한컴 fallback 정합 (basename 매칭) → 동등 또는 개선

**산출**: `mydocs/working/task_m100_873_stage_b.md`

### Stage C: 종합 보고서 + 커밋

**C.1 종합 보고서**: 본질 / 정정 / 검증 / 영향도

**산출**: `mydocs/report/task_m100_873_report.md`

**커밋**: `Task #873: HWP5 BinData Link 타입 → Picture external_path 설정 (closes #873)`

## 작업 순서 + 승인 게이트

```
A 본질 정밀 진단 → 산출 → 단계 완료 → 승인
                                        ↓
B 정정 구현 + 검증 → 산출 → 단계 완료 → 승인
                                        ↓
C 종합 보고서 + 커밋 → 단계 완료 → 승인
```

각 단계 후 승인 게이트.

## 위험 + 회피

| 위험 | 회피 |
|---|---|
| HWP5 spec 의 BinData index 0-based / 1-based 차이 | Stage A 에서 실제 dump 로 검증 |
| `abs_path` 가 Windows path 인 경우 Mac 에서 미접근 | Task #741 fallback 의 basename 추출 + 같은 dir 매칭으로 자동 처리 |
| 다른 HWP5 sample 회귀 | Stage B 의 회귀 검증 + Embedding/Storage 분기 미수정 |
| Document IR 변경 회귀 | Document IR 변경 없음 (existing `external_path` field 만 사용) |

## 본 단계 범위 외

- HWP5 BinData Embedding / Storage 의 image 데이터 로드 (이미 동작)
- HWPX 의 동등 처리 (별도 본질, 본 task 범위 외)
- Renderer / 공통 모듈 변경

## 승인 요청

본 구현계획서 승인 후 → Stage A 부터 진행.

📋 **Task #873 구현계획서 승인 요청드립니다.**
