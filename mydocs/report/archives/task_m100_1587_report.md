# Task #1587 — 최종 결과보고서

**제목**: HWPX 저장 시 Ruby(덧말) 컨트롤 드롭 수정
**마일스톤**: M100 (v1.0.0) · **이슈**: edwardkim/rhwp#1587 · **브랜치**: `local/task1587`

---

## 1. 문제

HWPX 저장 시 본문/표셀의 Ruby(덧말, 한자 독음·위첨자) 컨트롤이 드롭. fidelity 잔여
IR_DIFF 10건 중 3건(36384160·36389301·36399208), **유일하게 시각 영향 있는 실버그**.

## 2. 근본원인 (2계층)

1. **즉시**: `render_control_slot`(serializer/hwpx/section.rs)에 `Control::Ruby` arm 부재 →
   `is_hwpx_inline_slot` 에는 등록(슬롯 인식)됐으나 방출되지 않아 드롭. (ColumnDef #1584 동형.)
2. **심층(그라운딩 발견)**: `Ruby` 모델이 손실 구조 — `mainText`(기준 텍스트) 미보존,
   `posType`/`align` 을 u8 1개로 병합, `szRatio`/`option`/`styleIDRef` 드롭. 단순 arm 추가만으로는
   무손실 불가 → 모델+파서+직렬화기 3계층 수정.

## 3. 해결

| 계층 | 파일 | 변경 |
|------|------|------|
| 모델 | `model/control.rs` Ruby | `alignment` 제거 → `main_text`/`pos_type`/`align`/`sz_ratio`/`option`/`style_id_ref` |
| 파서 | `parser/hwpx/section.rs` parse_dutmal | mainText 보존 + posType/align 분리 + szRatio/option/styleIDRef 파싱 |
| 직렬화 | `serializer/hwpx/section.rs` | `render_dutmal`(`<hp:dutmal>` 역매핑) + `Control::Ruby` arm |

- `alignment` 제거 외부 파급 0(parse_dutmal 한정 — main.rs 는 ruby_text 만, HWP5 는 ruby 미지원).

## 4. 검증

| 검사 | 결과 |
|------|------|
| 단위 RED→GREEN (전 필드 무손실) | PASS |
| `cargo test --lib` | 1961 passed, 0 failed |
| `hwpx_roundtrip_baseline` | 4/4 |
| opengov snapshot (36389301 가드) | PASS |
| **fidelity 통제 비교** | **개선 3 / 회귀 0 / 순효과 +3** (IR_DIFF 10→7) |
| Hangul 오라클 | 2건 OK(시각 정상) + 1건 별개 선존 붕괴(#1589) |

## 5. 부수 발견 — 36384160 페이지 붕괴 (#1589)

36384160 은 ruby 수정으로 IR PASS 가 됐으나 한글에서 29→3쪽 붕괴(IR 게이트 미검출).
**ruby 수정 전후 동일** → 선존 시각 버그로 확정, 이슈 #1589 분리 등록. 본 타스크 무관.

## 6. 산출물

- 소스: control.rs, parser/hwpx/section.rs, serializer/hwpx/section.rs
- 테스트: `task1587_ruby_control_roundtrips` + `samples/hwpx/opengov/36389301…` 가드
- 문서: 수행/구현 계획서, `_stage1~4`, 본 보고서, 잔존 분석(`tech/hwpx_residual_ir_diff_10.md`)

## 7. 후속

- #1588 선 도형 shapeComment 드롭(Class B) — 1줄 수정 대기.
- #1589 페이지 붕괴(시각 갭) — 오라클 전수 군집 조사 필요.
- 잔여 IR_DIFF 7건: char_shape 시프트(para0) + spurious(0,0) 등(별건).
