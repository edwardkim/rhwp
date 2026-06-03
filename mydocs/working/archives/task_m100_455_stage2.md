# Task #455 Stage 2 — 수정 적용

## 변경 파일

`src/renderer/layout/paragraph_layout.rs`

## 변경 내용

`skip_text_for_inline_shape` 플래그 제거. 인라인 글상자(Shape with TextBox + treat_as_char) 가 있는 줄에서도 외부 문단 본문 텍스트를 스킵하지 않고 정상 렌더한다.

### 제거된 코드

```rust
// 인라인 Shape 중 글상자(TextBox)가 있는 경우에만 텍스트 스킵
// (글상자 텍스트는 table_layout에서 렌더링)
// 단순 도형(사각형, 원 등)은 TextBox가 없으므로 텍스트를 여기서 렌더링
let skip_text_for_inline_shape = has_tac_shape && para.map(|p| {
    tac_offsets_px.iter().any(|(_, _, ci)| {
        if let Some(Control::Shape(s)) = p.controls.get(*ci) {
            s.drawing().map(|d| d.text_box.is_some()).unwrap_or(false)
        } else { false }
    })
}).unwrap_or(false);
```

`if !skip_text_for_inline_shape { ... }` 두 곳도 무조건 렌더하도록 변경 (블록만 유지하여 변수 스코프 보존).

### 주석 갱신

```rust
// [Task #455] 외부 문단 본문 텍스트는 글상자 유무와 무관하게 항상 렌더한다.
// 글상자(TextBox) 자체와 그 내부 텍스트("개화" 같은)는
// shape_layout 이 inline_shape_position 을 보고 별도 패스에서 렌더하므로 중복되지 않는다.
```

## 검증

`./target/release/rhwp export-svg samples/exam_kor.hwp` 후 `output/exam_kor_002.svg` 좌측 단:

수정 전:
```
y=295.1 "서양의 과학과 기술... 이항로를 비롯한"
y=321.4 "개화"        ← 본문 39자 누락
y=347.6 "수없는 대세로 자리잡았다..."
```

수정 후:
```
y=295.1 "서양의 과학과 기술... 이항로를 비롯한"
y=321.4 "개화"        ← 글상자 (별도 패스)
y=322.6 "척사파의 주장은 개항 이후에도 지속되었지만, 는 거스를"   ← 본문 24자 + 5자 복원
y=347.6 "수없는 대세로 자리잡았다..."
```

본문이 글상자 좌·우로 분리되어 정상 렌더된다 (`척사파의 ... 지만,` + 글상자 + `는 거스를`). 24+5=29자이며, 글상자가 인라인 글자처럼 점유하는 폭만큼 좌·우로 텍스트가 흐른다.

## 알려진 미세 차이

본문 글자 baseline y=322.6, 글상자 내부 "개화" baseline y=321.4 — 1.2px 차이. 줄 높이가 5mm 글상자에 맞춰 1417 HU 로 늘어난 상황에서 baseline 정렬을 본문 폰트 기준으로 다시 계산하지 않아 생기는 미세 어긋남. 시각상 큰 문제는 아니나 후속 개선 여지 있음 (본 타스크 스코프 외).

## 다음

Stage 3 회귀 검증.
