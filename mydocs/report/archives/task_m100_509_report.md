# Task #509 최종 보고서 — PUA 글리프 회귀 정합

**Issue**: #509 — PUA (Private Use Area) 글머리표 글리프 회귀
**Branch**: `local/task509`
**완료일**: 2026-05-02
**작업지시자 시각 검증**: 통과

## 본질 정합

PR #251 (외부 컨트리뷰터, 2026-04 영역) 의 일반 텍스트 PUA 변환 도입 후 KTX 샘플 등의 PUA 글머리표가 **☰ (폰트 폴백 결함 글리프)** 으로 잘못 출력되는 회귀 발생. 회귀 본질 — **매핑 표의 정확성** 영역.

## 진단 영역 (Stage 1 ~ Stage 3)

### Stage 1 — 본질 진단

광범위 PUA 사용 영역 정합 (14 샘플 / 18 codepoint / 111 회 사용).

**3개 영역 분기:**
1. Basic PUA 매핑 적용 (0xF020~0xF0FF) — 매핑 영역 정합 점검 영역
2. Basic PUA 매핑 외 (예: U+0F53A 한글 영역) — 매핑 표 범위 외, 폴백 영역
3. Supplementary PUA-A (0xF02B0~0xF02FF) — 한컴 자체 영역, 함수 매핑 추가 영역

### Stage 2 — 한컴 매뉴얼 정합

`mydocs/manual/hwp/Help/extracted/hwpbase/hncpuaconverter.htm` 정독:
- PUA E000~F8FF 영역의 본질 = **옛한글 영역** (Wingdings 가설 부정합)
- "정보교환용 한글 처리지침 (KS X 1026-1:2007)" 표준 정합

→ 별도 task #512 등록 (KS X 1026-1:2007 옛한글 자모 변환 영역).

### Stage 3 ~ 4 — Option 점검 + Option F 확정

**Option A** (포기 영역) — 함수 매핑 정합 강화 후 시각 검증 실패
**Option B** (포기 영역) — draw_text 변환 제거 → 회귀 그대로 ("다시 처음 버그 상태로 돌아왔군요")
**Option C** (포기 영역) — draw_text 변환 제거 + 매핑 표 정정 → 시각 검증 실패
**Option F (확정)** — draw_text 변환 보존 (PR #251 영역) + 매핑 표 한컴 PDF 정답지 정확화

**작업지시자 통찰 정합** (Option F 의 본질):
> "한컴의 경우 PUA 는 폰트지정과 상관없이 처리해야 할 것 같습니다. 지정된 폰트에 없는 경우 현재 문제가 되는 것으로 판단됩니다."

→ rhwp 도 한컴과 동일하게 PUA 자체 변환 발행 (폰트 영역 의존 회피).

### Stage 5 — 한컴 PDF 정답지 시각 검증

`gen-pua` 영역의 `samples/pua-test.hwp` 생성 → 한컴 2022 편집기 영역에서 시각 캡처 영역 정답지로 활용.

**한컴 PDF 정답지 캡처 정합 결과:**

| codepoint | 한컴 PDF 정답지 | 정정 전 매핑 | 정정 후 매핑 | 정합 |
|-----------|-----------------|--------------|--------------|------|
| U+0F076 | ❖ | ❖ U+2756 | (변경 없음) | ✅ |
| U+0F09F | • | • U+2022 | (변경 없음) | ✅ |
| **U+0F0A0** | **· (Middle dot)** | ▪ U+25AA | **· U+00B7** | ✅ 정정 |
| U+0F0A7 | ▪ | ▪ U+25AA | (변경 없음) | ✅ |
| **U+0F0E8** | **➔ (Heavy wide-headed arrow)** | (없음) → ➤ U+27A4 | **➔ U+2794** | ✅ 정정 |
| U+0F0F2 | ⇩ | ⇩ U+21E9 | (변경 없음) | ✅ |
| U+0F0FE | ☑ | ☑ U+2611 | (변경 없음) | ✅ |
| U+F02B1 ~ B9 | ① ~ ⑨ | ① ~ ⑨ U+2460~2468 | (변경 없음) | ✅ |
| U+F02EF | · | · U+00B7 | (변경 없음) | ✅ 함수 영역 |

## 적용 변경

### `src/renderer/layout/paragraph_layout.rs`

1. **`map_pua_bullet_char` 함수 영역 정정:**
   - `0xA0 => '\u{00B7}'` (이전 ▪ U+25AA → · U+00B7)
   - `0xE8 => '\u{2794}'` (이전 ➤ U+27A4 → ➔ U+2794)
   - Supplementary PUA-A 영역 (0xF02B0~0xF02FF) 추가:
     - 0xF02B1~F02B9 → ① ~ ⑨ (U+2460~U+2468)
     - 0xF02EF → · (U+00B7)
2. **5개 단위 테스트 추가** (`pua_mapping_tests`)

### `src/renderer/svg.rs`, `src/renderer/web_canvas.rs`, `src/renderer/html.rs`

3개 렌더러의 `draw_text` 영역에 PR #251 의 PUA 변환 영역 보존 (Option F 본질):

```rust
let text = &text
    .chars()
    .map(crate::renderer::layout::map_pua_bullet_char)
    .collect::<String>();
```

### `src/main.rs`

`gen-pua` 서브명령 추가 — 18 종 PUA 코드포인트 영역 검증용 HWP 생성 도구.

### `samples/pua-test.hwp`, `samples/pua-test.pdf`

검증 영역 자료 — gen-pua 영역에서 생성한 HWP + 한컴 출력 PDF.

## 검증 게이트

| 게이트 | 결과 |
|--------|------|
| `cargo build --lib` | ✅ 성공 |
| `cargo test --lib pua_mapping` | ✅ 5/5 통과 |
| `cargo test --test svg_snapshot` | ✅ 6/6 통과 (회귀 없음) |
| WASM 빌드 (Docker) | ✅ 성공 (4.18 MB) |
| rhwp-studio 배포 | ✅ rhwp-studio/public/ 갱신 (May 2 04:14) |
| **작업지시자 시각 검증** | ✅ **통과** |

## 별도 영역 결함 (본 task 너머)

**Supplementary PUA-A 의 SVG 출력 누락:**
- 함수 매핑은 정확 (① ~ ⑨, · 정합 출력)
- SVG 출력 영역에서 누락 — paragraph_layout 의 char_offsets ↔ chars() 정합 영역의 UTF-16 surrogate pair 처리 결함 추정
- 한컴 캡처 영역에서는 본 영역 모두 정상 출력 — rhwp SVG 영역만 누락
- **별도 task 분리 권장**

## 메모리 정합

- `feedback_pdf_not_authoritative` — 한컴 PDF 자체 환경 의존, 단 본 task 의 한컴 정답지는 **작업지시자가 한컴 2022 영역에서 직접 확인** 영역 → 정답지 영역 정합
- `feedback_hancom_compat_specific_over_general` — 매핑 표는 case-by-case 영역 정정 (일반화 회피)
- `feedback_search_troubleshootings_first` — Stage 1 의 PUA 비교 문서 = 결정적 진단 도구

## 결론

**Task #509 완료** — PR #251 의 draw_text PUA 변환 영역 보존 + 한컴 PDF 정답지 캡처 정합 영역의 매핑 표 정정 (U+0F0A0, U+0F0E8 두 영역).

본질 정합 — "한컴은 PUA 를 폰트 영역과 무관하게 자체 처리. rhwp 도 동일 영역 정합."
