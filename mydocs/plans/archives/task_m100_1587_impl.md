# 구현 계획서 — Task #1587

**제목**: HWPX Ruby(덧말) 컨트롤 드롭 수정 — 모델+파서+직렬화기 3계층
**브랜치**: `local/task1587` · **이슈**: edwardkim/rhwp#1587
**전제**: 수행계획서(`task_m100_1587.md`) 승인됨

---

## 1. 파급 범위 (그라운딩 확정)

| 계층 | 파일 | 현 상태 | 변경 |
|------|------|---------|------|
| 모델 | `src/model/control.rs` Ruby | ruby_text+alignment(2필드) | 필드 확장(아래) |
| 파서(HWPX) | `parser/hwpx/section.rs` parse_dutmal | mainText skip, posType/align 충돌 | 전 속성/요소 보존 |
| 직렬화(HWPX) | `serializer/hwpx/section.rs` | Ruby arm 부재(드롭) | write_ruby + arm |
| HWP5 | — | ruby 파서 **부재**(미지원) | **무영향**(extra 필드 무시) |

`.alignment` 읽기는 parse_dutmal 한 곳뿐(main.rs 는 ruby_text 만, body_text 는 `_` 매칭) →
모델 필드 교체의 외부 파급 없음.

## 2. 모델 변경 (C1)

```rust
pub struct Ruby {
    pub main_text: String,   // mainText 기준 텍스트 — 신규(시각 충실도 핵심)
    pub ruby_text: String,   // subText 덧말
    pub pos_type: u8,        // posType: 0=TOP, 1=BOTTOM — 신규(alignment 분리)
    pub align: u8,           // align: 0=LEFT, 1=RIGHT, 2=CENTER — 신규
    pub sz_ratio: u8,        // szRatio — 신규
    pub option: u32,         // option — 신규
    pub style_id_ref: u16,   // styleIDRef — 신규
}
```
- `alignment` 제거(pos_type+align 로 분리). `#[derive(Default)]` 유지 → 기존 호출 호환.

## 3. 파서 변경 (C2) — `parse_dutmal`

- 속성: `posType`→pos_type, `align`→align, `szRatio`→sz_ratio, `option`→option,
  `styleIDRef`→style_id_ref (문자열→정수 파싱).
- 자식: `mainText`→`read_dutmal_text`로 `main_text` 채움(현 skip 제거), `subText`→ruby_text.
- 호출부(section.rs:515)의 `\u{0002}` 마커 push 는 유지(슬롯 위치 보존).

## 4. 직렬화 변경 (C3) — `write_ruby` + arm

- 신규 `write_ruby(ruby) -> String`: `<hp:dutmal posType= align= szRatio= option= styleIDRef=>`
  `<hp:mainText>{main_text}</hp:mainText><hp:subText>{ruby_text}</hp:subText></hp:dutmal>`.
  속성/텍스트 XML escape. parse_dutmal 의 정확한 역매핑.
- `render_control_slot` 에 `Control::Ruby(r) => out.push_str(&write_ruby(r))` arm 추가.
  Ruby 는 이미 `is_hwpx_inline_slot` 포함 → 슬롯 위치 자동.

## 5. 구현 단계 (4단계)

### Stage 1 — 재현 테스트 (RED)
- `serialize_hwpx→parse_hwpx` roundtrip 단위 테스트: ruby 포함 문단 → reparse 후 controls 에
  Ruby 보존 + ruby_text 일치 검증. 현재 RED(controls=[]).
- 테스트는 `Ruby { ruby_text, ..Default::default() }` 형태로 작성(모델 변경에 견고).
- 커밋: `Task #1587: Ruby 드롭 재현 테스트 (RED)` + `_stage1.md`.

### Stage 2 — 모델+파서 확장 (C1, C2)
- C1 모델 필드 교체, C2 parse_dutmal 전 속성/요소 보존.
- `cargo build` + 기존 테스트 영향 없음 확인(파급 parse_dutmal 한정).
- 커밋: `Task #1587: Ruby 모델+파서 확장` + `_stage2.md`.

### Stage 3 — 직렬화기 (C3)
- write_ruby + arm. Stage1 GREEN + 신규 필드(main_text/pos_type/align/sz_ratio/option/
  style_id_ref) 무손실 단언 추가.
- `cargo test --lib` + `hwpx_roundtrip_baseline` 회귀 0.
- 커밋: `Task #1587: Ruby 직렬화 (write_ruby + arm)` + `_stage3.md`.

### Stage 4 — 통제 비교 (채택 게이트)
- fidelity 전수 재측정: 3건(36384160·36399208·36389301) 해소 확인, 악화 0, 순효과>0.
- opengov 가드(36389301) 편입 + snapshot 갱신.
- 커밋: `_stage4.md` + `_report.md`.

## 6. 롤백 기준
Stage 4 통제 비교 악화 ≥1 또는 baseline 회귀 시 전량 되돌리고 재분석(부분 채택 금지).

## 7. 산출물
- 소스: control.rs, parser/hwpx/section.rs, serializer/hwpx/section.rs
- 테스트: 신규 roundtrip 단위 + opengov 가드
- 문서: `_stage1~4`, `_report`
