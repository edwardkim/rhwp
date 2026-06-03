# Task #1058 Stage 5~15 보고서 — reopen 후 추가 4 정정

- 이슈: [#1058](https://github.com/edwardkim/rhwp/issues/1058) (reopen → resolved)
- 단계: Stage 5~15 (reopen 후 추가 영역)
- 일시: 2026-05-22

## 1. 배경

Stage 4 (글상자 LIST_HEADER 13 byte) 머지 후 작업지시자 한컴 시각 판정에서 다단계 목록 부여 잔존 확인 → 본 task reopen. **글상자 없는 케이스도 발생** + **HWP→HWP 라운드트립 정상** → HWPX→HWP 만의 본질.

작업지시자 통찰 (Stage 4-pivot 패턴): **"정답지와 저장 버전과 차이를 통해 추론"** + "스타일 바인딩 확인" + "정답 hwp와 비교".

## 2. 4 라운드 시각 판정 + 4 정정

| Round | 작업지시자 보고 | 본질 식별 | 정정 |
|-------|----------------|----------|------|
| 1 | "각주 추가 시 1.1.1.1.1.1 부여" | PARA_HEADER instance_id MSB (HWPX `<hp:p id>`) | parser/hwpx/section.rs::parse_paragraph — raw_header_extra |
| 2 | "왼쪽 여백 60.0pt + 줄간격 160%" | Style record 의 lang_id (INT16) + trailing 2 byte 누락 | model/style.rs + parser + serializer + parser/hwpx |
| 3 | "각주 스타일 다른 기존 각주와 다름" | 부작용 — Style.lang_id 정합으로 해결 (정답지 raw byte 정합) | (위 2 와 통합) |
| 4 | "일반 문단 시작에 글머리표" | HWPX `<hh:bullet>` 4개 파싱 누락 | parser/hwpx/header.rs — parse_bullet_hwpx |

추가: ParaShape line_spacing_v2 보정 (5.0.2.5 이상). attr1 bit 7 강제 set 시도는 부작용 (ps[5]=0 정답지 정합 깨짐) 으로 제거.

## 3. 정정 영역 매트릭스

### 3.1 `src/model/style.rs`
- Style struct 에 `lang_id: i16` 필드 추가 (HWP5 spec 표 47)

### 3.2 `src/parser/doc_info.rs::parse_style`
- lang_id (INT16, default 1042) 읽기
- trailing 2 byte 흡수 (스펙 미문서화)

### 3.3 `src/serializer/doc_info.rs::serialize_style`
- lang_id (INT16) + trailing 2 byte zero 작성
- lang_id == 0 default 1042 (한국어)

### 3.4 `src/parser/hwpx/header.rs`

**parse_style**: HWPX `langID` → `lang_id`, default 1042.

**parse_para_shape**: line_spacing_v2 == 0 시 line_spacing 값 동기화 (UINT32).
attr1 bit 7 강제 set 시도 → **제거** (정답지 ps[5]=0 회귀 방지).

**parse_bullet_hwpx (신규)**: HWPX `<hh:bullet id="N" char="❏" useImage="0">` 파싱
→ IR `Bullet { bullet_char, image_bullet }`. main loop 의 Empty event 분기에 추가.

### 3.5 `src/parser/hwpx/section.rs::parse_paragraph`
- HWPX `<hp:p id>` 속성 → `para.raw_header_extra` 의 offset 6..10 에 instance_id (UINT32 LE) 작성
- HWPX 의 id="2147483648" (=0x80000000) MSB set 보존

## 4. 정량 입증

### 4.1 정답지 record-level 정합

**Style record raw byte 완전 동일**:
```
ORACLE style[0] size=32:  03 00 ... 12 04 07 00 00 00 00 00
rhwp   style[0] size=32:  03 00 ... 12 04 07 00 00 00 00 00 ← 정합
```

**DocInfo Tag 24 (BULLET) 개수**:
- ORACLE: 4
- rhwp 이전: 0 → 정정 후: **4** ✓

**ParaShape line_spacing_v2 정합** (line_spacing 130/160 정확 보존).

**PARA_HEADER instance_id 35×MSB + 23×0 패턴** 정답지 정합.

### 4.2 회귀 가드 9/9 통과

`tests/issue_1058_textbox_list_header.rs`:
- issue_1058_textbox_list_header_size_33
- issue_1058_textbox_list_header_byte_contract
- issue_1058_hwp_textbox_roundtrip
- issue_1058_footnote_list_header_size_16_preserved
- **issue_1058_paragraph_instance_id_mapped** (신규)
- **issue_1058_para_shape_line_spacing_v2_synced** (신규)
- **issue_1058_style_lang_id_preserved** (신규)
- **issue_1058_bullet_records_preserved** (신규)
- **issue_1058_serialized_bullet_count** (신규)

### 4.3 CI 패턴

| 항목 | 결과 |
|------|------|
| cargo test --release --lib | **1323 passed** |
| cargo test --release --tests | FAILED 0 |
| cargo clippy --release --lib -D warnings | clean |
| cargo fmt --all --check | clean |

### 4.4 광범위 sweep (12 fixtures, 229 SVG)

| Fixture | BEFORE/AFTER diff |
|---------|-------------------|
| samples/hwpx/footnote-tbox-01.hwpx | 0 |
| samples/footnote-tbox-01.hwp | 0 |
| **samples/hwpx/footnote-01.hwpx** | **4** (의도된 본질 정정) |
| samples/footnote-01.hwp | 0 |
| samples/2010-01-06.hwp | 0 |
| samples/table-in-tbox.hwp | 0 |
| samples/hwp3-sample-hwp5.hwp | 0 |
| samples/hwp3-sample16-hwp5.hwp | 0 |
| samples/aift.hwp | 0 |
| samples/KTX.hwp | 0 |
| samples/biz_plan.hwp | 0 |
| samples/exam_kor.hwp | 0 |

→ **footnote-01.hwpx (HWPX 출처) 만 변동** (의도 — ParaShape/Style/Bullet 본질 정정), **HWP 출처 11 fixture 회귀 부재**.

### 4.5 WASM Docker 빌드 + 동기화

- `pkg/rhwp_bg.wasm` 4.91 MB
- rhwp-studio/public 동기화

### 4.6 작업지시자 한컴 한글 2020 시각 판정 4 라운드 통과

| Round | 영역 | 결과 |
|-------|------|------|
| 9 | 다단계 목록 1.1.1.1.1.1 부작용 | ✓ |
| 12 | 왼쪽 여백 60.0pt + 줄간격 160% | ✓ |
| 14 | 일반 문단 글머리표 | ✓ |
| 15 | 종합 성공 | ✓ "성공입니다" |

## 5. 메모리 룰 정합

- ✅ `feedback_visual_judgment_authority` — 4 라운드 작업지시자 시각 판정 게이트, 매 라운드 정확한 정정으로 통과
- ✅ `feedback_diagnosis_layer_attribution` — Stage 4-pivot 패턴 (정답지 vs 저장본 record-level diff) 으로 본질 위치 정확 식별
- ✅ `feedback_self_verification_not_hancom` — rhwp 자기 정합 ≠ 한컴 호환. 한컴 시각 판정 게이트 필수
- ✅ `feedback_hancom_compat_specific_over_general` — Style.lang_id 1042 default + Bullet 정확 매핑 — case-specific contract
- ✅ `feedback_push_full_test_required` — lib + tests + clippy + fmt 모두 통과
- ✅ `project_hwpx_to_hwp_adapter_limit` 정합 + **단순 어댑터 한계 점진 돌파** — Task #1050/#1052/#1058 의 contract unit 누적

## 6. `hwpx2hwp-rule.md` contract unit 추가 (3번째)

Task #1050 의 footnote contract + Task #1058 1차의 TextBox LIST_HEADER + **Task #1058 reopen 의 4 영역**:

- **PARA_HEADER instance_id** (HWPX `<hp:p id>` → UINT32 LE 6..10 offset)
- **Style record lang_id INT16 + trailing UINT16 zero** (HWP5 spec 표 47, hwplib 정합)
- **ParaShape line_spacing_v2 UINT32** (5.0.2.5 이상)
- **HWPX `<hh:bullet>` → HWP BULLET record** (한컴 default 글머리표 부재 시 자동 부여 결함 정정)

작업지시자 통찰의 결정성 — Stage 4-pivot 의 "HWP IR oracle 방식" 이 본 task reopen 의 4 라운드에서도 정확한 본질 식별 방법론.
