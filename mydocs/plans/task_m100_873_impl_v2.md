# Task #873 구현계획서 v2 (HWPX 포함)

**선행**: `task_m100_873.md` (수행계획서), `task_m100_873_impl.md` (v1, HWP5 only)
**브랜치**: `local/task873`
**작성일**: 2026-05-13

## v1 대비 변경

Stage A 진단 후 **HWPX 도 동일 결함 보유** 발견 (사용자 추가 지시). 범위 확장.

| | v1 (HWP5 only) | v2 (HWP5 + HWPX) |
|---|---|---|
| 정정 위치 | 1곳 (`src/parser/mod.rs`) | 3곳 (`src/parser/mod.rs` + `src/parser/hwpx/content.rs` + `src/parser/hwpx/mod.rs`) |
| 변경 파일 | 1 | 3 |
| 검증 sample | hwp3-sample10-hwp5.hwp | + hwp3-sample10-hwpx.hwpx |

## 단계 분해 (3 단계 + 승인 게이트)

### Stage B: 정정 구현 + 검증

**B.1 HWP5 정정**:

`src/parser/mod.rs` — `parse_from_bytes` 의 `assign_auto_numbers(&mut doc);` 다음에 `populate_link_image_paths(&mut doc);` 호출 추가.

새 함수:
```rust
/// [Task #873] BinData Link 타입의 외부 file path 를 Picture.image_attr.external_path
/// 로 전달. Task #741 의 populate_external_images_from_dir fallback 이 같은 dir 에서
/// basename 매칭으로 자동 load 한다.
fn populate_link_image_paths(doc: &mut Document) {
    use crate::model::control::Control;
    use crate::model::shape::ShapeObject;
    use crate::model::bin_data::BinDataType;

    let bin_data = doc.doc_info.bin_data_list.clone(); // borrow 충돌 회피
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
                if pic.image_attr.external_path.is_some() { continue; }
                let bin_idx = (pic.image_attr.bin_data_id as usize).saturating_sub(1);
                if let Some(bd) = bin_data.get(bin_idx) {
                    if matches!(bd.data_type, BinDataType::Link) {
                        let path = bd.abs_path.clone()
                            .filter(|p| !p.is_empty())
                            .or_else(|| bd.rel_path.clone().filter(|p| !p.is_empty()));
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

**B.2 HWPX 정정**:

**B.2.1** `src/parser/hwpx/content.rs`:

`PackageItem` 에 `is_embedded: bool` 추가:
```rust
pub struct PackageItem {
    pub href: String,
    pub media_type: String,
    pub id: String,
    pub is_embedded: bool,  // 신규 — isEmbeded attribute (true if embedded)
}
```

`parse_content_hpf`:
- `isEmbeded` attribute 파싱 (`"0"` → false, `"1"` → true, default true)
- bin_data_items 수집 조건 변경: media_type 이 `image/` 시작이거나 기존 BinData/ 경로

**B.2.2** `src/parser/hwpx/mod.rs:82-89`:

```rust
for (i, item) in package_info.bin_data_items.iter().enumerate() {
    let ext = item.href.rsplit('.').next().unwrap_or("dat").to_string();
    let (data_type, abs_path) = if item.is_embedded {
        (BinDataType::Embedding, None)
    } else {
        (BinDataType::Link, Some(item.href.clone()))
    };
    doc_info.bin_data_list.push(BinData {
        data_type,
        storage_id: (i + 1) as u16,
        extension: Some(ext),
        abs_path,
        ..Default::default()
    });
}
```

또한 BinData 이미지 로딩 (line 114~130) 시 Link 타입은 skip:
```rust
for (i, item) in package_info.bin_data_items.iter().enumerate() {
    if !item.is_embedded { continue; }  // Link 는 외부 file
    // ... 기존 로드 로직
}
```

**B.3 정정 검증**:

1. cargo build --release
2. cargo test --release --lib (1246 passed 회귀 0)
3. **hwp3-sample10-hwp5.hwp**:
   - `rhwp export-svg samples/hwp3-sample10-hwp5.hwp -p 0 -o /tmp/v2_h5/`
   - `grep -c 'data:image' /tmp/v2_h5/*.svg` = 3
   - rsvg-convert PNG 변환 후 시각 확인 (oracle.gif + 다이어그램 + 사람 그림)
4. **hwp3-sample10-hwpx.hwpx**:
   - `rhwp export-svg samples/hwp3-sample10-hwpx.hwpx -p 0 -o /tmp/v2_hx/`
   - `grep -c 'data:image' /tmp/v2_hx/*.svg` = 3
   - rsvg-convert PNG 변환 후 시각 확인
5. **hwp3-sample10.hwp** (HWP3 원본) — 회귀 0
6. **hwp3-sample14.hwp / -hwp5.hwp / -hwpx.hwpx** — Task #864 정합 유지 (회귀 0)
7. cargo clippy --release --lib 경고 0
8. 임시 진단 print (main.rs 의 DocInfo BinData dump) 제거

**B.4 회귀 우려**:

- 일반 HWP5 sample (BinDataType::Embedding) → 영향 없음 (matches!(Link, _) = false)
- 일반 HWPX sample (BinData/ 내장, isEmbeded="1") → 영향 없음 (is_embedded = true → Embedding 유지)
- Chart/OOXML (bin_data_id = 60000+) → 영향 없음 (range 다름)

**산출**: `mydocs/working/task_m100_873_stage_b.md`

### Stage C: 종합 보고서 + 커밋

**C.1 종합 보고서**: 본질 / 정정 (HWP5 + HWPX) / 검증 / 영향도

**산출**: `mydocs/report/task_m100_873_report.md`

**커밋 메시지 (안)**:
```
Task #873: HWP5/HWPX 외부 참조 image 정합 — Picture.external_path 전달 (closes #873)

본질 1 (HWP5): BinDataType::Link 의 abs_path/rel_path 가 Picture.image_attr.external_path
에 전달되지 않아 Task #741 fallback (populate_external_images_from_dir) 미동작.
src/parser/mod.rs 에 populate_link_image_paths 후속 함수 추가.

본질 2 (HWPX): content.hpf parser 가 BinData/ 폴더 외 image 항목 (예:
isEmbeded="0" + Windows absolute path) 을 필터 아웃 + isEmbeded attribute 미파싱.
src/parser/hwpx/content.rs 에서 image/* media-type 모두 수집 + is_embedded 추가.
src/parser/hwpx/mod.rs 에서 is_embedded=false 인 경우 BinDataType::Link + abs_path
설정.

이후 populate_link_image_paths (모든 포맷 공통) 가 Picture.external_path 설정 →
Task #741 fallback 이 basename 매칭으로 같은 dir 에서 image load.

검증: cargo test 1246 passed (회귀 0), clippy 경고 0.
hwp3-sample10-hwp5.hwp / hwp3-sample10-hwpx.hwpx 모두 3 image 정합.

closes #873
```

## 작업 순서 + 승인 게이트

```
B 정정 구현 + 검증 → 산출 → 단계 완료 → 승인
                                        ↓
C 종합 보고서 + 커밋 → 단계 완료 → 승인
```

## 위험 + 회피

| 위험 | 회피 |
|---|---|
| HWP5 일반 Embedding 회귀 | matches!(Link, _) 분기로 격리 |
| HWPX 일반 BinData/ 내장 회귀 | is_embedded=true 기본 + Embedding 유지 |
| Chart bin_data_id (60000+) 충돌 | bin_data_id range 분리 |
| Windows path Mac 접근 | Task #741 의 basename + same-dir fallback |
| content.hpf XML attr 대소문자 | `isEmbeded` (한컴 spec 정합 — embed 의 d 누락) 정확 사용 |

## 본 단계 범위 외

- HWP5 Embedding/Storage 의 image 데이터 로드 (이미 동작)
- HWPX BinData/ 내장 image (이미 동작)
- Renderer / 공통 모듈 변경 (Task #741 fallback 재사용)

## 승인 요청

본 구현계획서 v2 승인 후 → Stage B 부터 진행.

📋 **Task #873 구현계획서 v2 (HWPX 포함) 승인 요청드립니다.**
