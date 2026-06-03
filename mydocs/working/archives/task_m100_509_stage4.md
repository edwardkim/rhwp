# Task #509 Stage 4 — Option F 확정 + 한컴 PDF 정답지 정합

## Option 점검 영역 (A → B → C → F)

| Option | 본질 | 결과 |
|--------|------|------|
| A | 함수 매핑 정합 강화 | 시각 검증 실패 |
| B | draw_text 영역 PUA 변환 제거 | "다시 처음 버그 상태로 돌아왔군요" — 회귀 그대로 |
| C | draw_text 영역 PUA 변환 제거 + 매핑 표 정정 | 시각 검증 실패 |
| **F** | **draw_text 영역 PUA 변환 보존 (PR #251) + 매핑 표 한컴 PDF 정답지 정확화** | ✅ 시각 검증 통과 |

## Option F 의 본질 정합

**작업지시자 통찰:**
> "한컴의 경우 PUA 는 폰트지정과 상관없이 처리해야 할 것 같습니다. 지정된 폰트에 없는 경우 현재 문제가 되는 것으로 판단됩니다."

→ 한컴은 PUA 영역의 글머리표 글리프를 폰트 영역과 무관하게 자체 영역에서 발행. rhwp 도 동일 영역 정합 — `draw_text` 의 PUA 변환 영역 보존 (PR #251) + 매핑 표 정확성 영역 정정.

## 적용 변경 (3개 영역)

### 1. `src/renderer/svg.rs`, `src/renderer/web_canvas.rs`, `src/renderer/html.rs`

3개 렌더러의 `draw_text` 함수 영역에 PUA 변환 영역 보존:

```rust
let text = &text
    .chars()
    .map(crate::renderer::layout::map_pua_bullet_char)
    .collect::<String>();
```

### 2. `src/renderer/layout/paragraph_layout.rs` — 매핑 표 정정

| codepoint | 정정 전 | 정정 후 |
|-----------|---------|---------|
| **0xA0** (U+0F0A0) | ▪ U+25AA | **· U+00B7** |
| **0xE8** (U+0F0E8) | ➤ U+27A4 (이전 PR 임시) | **➔ U+2794** |

### 3. Supplementary PUA-A 영역 (0xF02B0~0xF02FF)

신규 분기 영역 추가:
- 0xF02B1 ~ F02B9 → ① ~ ⑨ (U+2460 ~ U+2468)
- 0xF02EF → · (U+00B7, Middle dot)

## 검증 도구 영역 — `gen-pua`

`src/main.rs` 영역의 `gen_pua_test` 함수 영역에서 18 종 PUA 코드포인트 영역 검증용 HWP 생성:

```bash
cargo run --bin rhwp -- gen-pua samples/pua-test.hwp
```

작업지시자가 한컴 2022 편집기 영역에서 본 파일 영역을 시각 캡처 → 정답지 영역으로 활용.

## 검증 게이트

| 게이트 | 결과 |
|--------|------|
| cargo build --lib | ✅ 성공 |
| cargo test --lib pua_mapping | ✅ 5/5 통과 |
| cargo test --test svg_snapshot | ✅ 6/6 통과 (회귀 0) |
| WASM 빌드 (Docker) | ✅ 4.18 MB |
| rhwp-studio 배포 | ✅ rhwp-studio/public/ 갱신 |

## 별도 영역 결함 (본 task 너머)

**Supplementary PUA-A 의 SVG 출력 누락:**
- 함수 매핑 정확 (① ~ ⑨, · 영역 정합)
- SVG 출력 영역에서 누락 — UTF-16 surrogate pair 처리 결함 추정
- 한컴 PDF 영역에서는 본 영역 모두 정상 출력 — rhwp SVG 영역만 누락
- **별도 task 분리 권장**

## 다음 단계 (Stage 5)

작업지시자 시각 검증:
1. rhwp-studio dev 영역 실행 (`cd rhwp-studio && npx vite --host 0.0.0.0 --port 7700`)
2. KTX p10 시각 영역 점검 — 글머리표 정상 출력 확인
3. samples/pua-test.hwp 시각 영역 점검 — 18 종 PUA 매핑 정합 확인
