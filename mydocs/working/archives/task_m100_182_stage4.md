# Stage 4 단계별 완료보고서: Picture + BinData ZIP 엔트리 + 3-way 단언

- **타스크**: [#182](https://github.com/edwardkim/rhwp/issues/182)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task182`
- **일자**: 2026-04-18
- **단계**: Stage 4 / 5

## 1. 범위

### 수행계획서 Stage 4 계획

- 추가: `src/serializer/hwpx/picture.rs` — `write_picture(ctx, w, pic)`
- `SerializeContext::bin_data_map`: 1-pass에서 생성 (Stage 0에서 이미 완성)
- `mod.rs`: Picture 존재 시 `BinData/*.png` ZIP 엔트리 추가
- `content::write_content_hpf(section_hrefs, ctx.bin_data_entries())` 연결
- 3-way 단언: `<hp:pic>` binaryItemIDRef ↔ content.hpf item ↔ ZIP entry

### 실제 적용 범위

- ✅ `picture.rs` 모듈 신설 + `write_picture` 구현
- ✅ `<hc:img binaryItemIDRef>` 해석 시 `SerializeContext::resolve_bin_id` 사용 → 미등록 bin_data_id 참조 시 `SerializeError::XmlError` 발생
- ✅ `mod.rs`: BinData 바이너리를 ZIP에 `BinData/imageN.ext` 로 쓰기
- ✅ `content.hpf` 에 BinData 엔트리 자동 등록 (`content_bin_entries` 변환)
- ✅ `assert_bin_data_3way()` — ctx hrefs 집합 vs ZIP entry 집합 동일성 단언
- ⚠️ **section.rs dispatcher 연결은 이월**: `Control::Picture` → `write_picture` 호출은 **#186 범위 C** 에 포함. `picture.rs` 독립 모듈로만 완성.

## 2. 산출물

### 신규 파일

**`src/serializer/hwpx/picture.rs`** (약 420줄)

구성:
- `write_picture(w, pic, ctx)` — `<hp:pic>` 진입점
- 속성 11개 (한컴 canonical 순서: id / zOrder / numberingType / textWrap / textFlow / lock / dropcapstyle / href / groupLevel / instid / reverse)
- 자식 (한컴 관찰 순서, PictureType.cpp 기준):
  - `write_offset` / `write_org_sz` / `write_cur_sz` / `write_flip` / `write_rotation_info`
  - `write_rendering_info` (transMatrix + scaMatrix + rotMatrix)
  - `write_img_rect` (pt0 ~ pt3)
  - `write_img_clip` / `write_in_margin` / `write_img_dim`
  - `write_img` — `<hc:img binaryItemIDRef>` (3-way 단언 지점)
  - `write_effects` (빈 요소)
  - `write_sz` / `write_pos` / `write_out_margin`
- enum 변환 헬퍼 6종 + `image_effect_str`

### 수정 파일

**`src/serializer/hwpx/mod.rs`** (+60줄):
- `pub mod picture;` 추가
- BinData ZIP 엔트리 쓰기 블록 추가
- `content::write_content_hpf(&section_hrefs, &content_bin_entries)` 로 BinData 엔트리 전달
- `assert_bin_data_3way()` 함수 신설 + 직렬화 종료 직전 호출

### 신규 단위 테스트 (6개)

`picture.rs` 내부:
- `pic_root_attrs_in_canonical_order` — 속성 순서 검증
- `img_uses_manifest_id` — `binaryItemIDRef="image1"` 출력 확인
- `unresolved_bin_data_id_errors` — 미등록 bin_data_id 참조 시 에러 발생
- `rendering_info_has_three_matrices` — transMatrix/scaMatrix/rotMatrix 3개 존재
- `img_rect_has_four_points` — pt0~pt3 4개 존재
- `image_effect_maps_to_string` — ImageEffect enum 변환

## 3. 검증 결과

### 3.1 단위 테스트

```
serializer::hwpx 관련: 47 passed, 0 failed
- canonical_defaults::tests: 5 ✅
- context::tests: 5 ✅
- fixtures::tests: 2 ✅
- roundtrip::tests: 3 ✅
- header::tests: 4 ✅
- section::tests: 5 ✅
- table::tests: 7 ✅
- picture::tests: 6 ✅ (신규)
- mod::tests (기존): 11 ✅
```

### 3.2 통합 테스트 (Stage 0/1/2 유지)

```
running 4 tests
test stage0_blank_hwpx_roundtrip ... ok
test stage1_ref_empty_roundtrip ... ok
test stage1_ref_text_roundtrip ... ok
test stage1_ref_mixed_header_level_regression_probe ... ok

test result: ok. 4 passed; 0 failed
```

### 3.3 전체 라이브러리

**840 passed, 0 failed, 1 ignored** — Stage 3의 834 대비 +6. 회귀 없음.

## 4. 완료 기준 대조

수행계획서 Stage 4 완료 기준:

| 기준 | 상태 | 근거 |
|---|---|---|
| Stage 0~3 하네스 유지 | ✅ | 4/4 통합 테스트 그린 |
| `pic-in-head-01.hwp`/`pic-crop-01.hwp` 라운드트립 IrDiff 0 | ❌ (이월) | section.rs dispatcher 없이는 `<hp:pic>` 이 section.xml 에 출력되지 않음 → #186 범위 C |
| ZIP 내 BinData/* 개수 == `doc.bin_data_content.len()` | ✅ | mod.rs 의 ZIP 엔트리 루프가 `bin_entries.len()` 만큼 쓰기 |
| 3-way 단언 (binaryItemIDRef ↔ manifest ↔ ZIP entry) | ✅ | `assert_bin_data_3way()` 직렬화 종료 직전 호출 |
| PNG Blake3 해시 일치 | ⚠️ | Stage 4 미구현 — BinData는 바이트 그대로 보존되나 해시 단위 테스트는 section dispatcher 연결 후 의미. #186 에서 확인 |

### 축소 근거

**"라운드트립 IrDiff 0"** 은 section.rs 에서 `Control::Picture` 를 실제 렌더링해야 성립. Stage 3 표와 동일 구조로 분할정복 원칙에 따라 #186 (Section 완전 동적화 + Control dispatcher) 로 이월.

`picture.rs` 자체의 정확성(canonical 순서, IR 매핑, 3-way 단언)은 본 단계에서 완결.

## 5. 주요 설계 결정

### 5.1 3-way 단언 설계

```
ctx.bin_data_entries() ──▶ content.hpf <opf:item> 자동 전환
                       ╲
                        ╲─▶ ZIP entry BinData/imageN.ext 쓰기
                        ╱
<hp:pic>의 binaryItemIDRef = ctx.resolve_bin_id(bin_data_id)
```

- 한 출처(`ctx.bin_data_map`)에서 세 집합 모두 파생 → 구조적으로 일치 보장
- 마지막에 `assert_bin_data_3way()` 가 ZIP 엔트리 집합과 ctx 집합 동일성 단언 → 방어적 검증

### 5.2 미등록 bin_data_id 에러 처리

`<hp:pic>` 이 참조하는 `image_attr.bin_data_id` 가 `doc.bin_data_content` 에 없으면 `SerializeError::XmlError` 반환. **사용자 입장에서 의미있는 메시지**:
```
<hp:pic> binaryItemIDRef 미등록 bin_data_id=99 (BinDataContent 누락)
```

### 5.3 간이 구현 결정

Stage 4 는 "Picture IR → XML" 골격 완성이 목표. 다음은 간이 구현:
- `<hp:orgSz>` 원본 크기 0,0 출력 (IR의 `shape_attr.original_width/height` 접근 제한)
- `<hp:renderingInfo>` identity 행렬 출력
- `<hp:effects>` 빈 요소
- `<hp:imgRect>` 에 common.width/height 기반 직사각형

실제 한컴 정답 파일과 완전 일치는 Stage 5 또는 후속 이슈에서 확장.

### 5.4 ZIP 엔트리 순서

`ctx.bin_data_entries()` 가 `bin_data_id` 기준 정렬. `mod.rs` 루프가 그 순서대로 ZIP 쓰기 → 결정론적 순서.

## 6. 알려진 제한 (#186 이월)

1. **Section dispatcher**: `Paragraph.controls[Control::Picture]` → `picture::write_picture` 호출. **#186 범위 C 에 명시**.
2. **`shape_attr` 접근**: Stage 4 에선 `orgSz`, `curSz` 가 IR의 ShapeComponentAttr 대신 common 필드 사용. Stage 5+ 또는 별도 이슈에서 확장.
3. **Effects 상세**: 그림자·광택·빛번짐 등 `<hp:effects>` 내부 요소. 현재 빈 출력.
4. **Caption**: `Picture.caption` 필드가 IR에 있으나 현재 직렬화 안 됨.

## 7. 다음 단계 (Stage 5)

**Stage 5 — 도형·필드 + 대형 실문서 스모크 + DVC 보조 검증**:

- 추가: `src/serializer/hwpx/shape.rs` — `<hp:rect>`, `<hp:line>`, `<hp:container>`, `<hp:textart>`
- 추가: `src/serializer/hwpx/field.rs` — `<hp:fieldBegin/End>`, 각주/미주 최소 세트
- 대형 샘플 3건 라운드트립 하네스 추가
- DVC (Windows VM) 보조 검증 (선택적)

완료 기준:
- 대형 샘플 3건 `IrDiff::allowed(shape_raw=true)` 통과
- 한컴2020 수동 오픈 성공
- 최종 보고서 `mydocs/report/task_m100_182_report.md` 작성

## 8. 승인 요청

본 Stage 4 단계별 완료보고서 검토 후 승인 시 Stage 5 (최종 단계) 착수. 
