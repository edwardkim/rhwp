# Task #937 Stage 1 완료보고서 — 재현 고정 + 현재 경로 확인

## 작업 범위

복학원서 서명란 `(인)` 렌더링 불일치의 원문 코드포인트와 현재 렌더 치환 누락을 테스트로 고정했다.

## 확인 결과

`samples/복학원서.hwp`의 문제 위치는 1페이지 서명란 표 셀 텍스트이며, 원문에는 한컴 PUA `U+F012B`가 들어 있다.

```text
셀[1] r=0,c=1 text="                          󰄫 (Signature)"
```

현재 `rhwp::renderer::composer::pua_to_display_text('\u{F012B}')`는 `None`을 반환한다. 따라서 SVG/Canvas 렌더에서 `(인)` 표시 문자열로 치환할 근거가 아직 없다.

## 변경 파일

- `tests/issue_937.rs`

## 추가 테스트

1. `issue_937_bokhakwonseo_signature_cell_contains_f012b`
   - `samples/복학원서.hwp`를 파싱한다.
   - 문서 전체 문단/표 셀/머리말/꼬리말/각주/미주/숨은설명/도형 글상자를 순회한다.
   - 서명란 텍스트에서 `U+F012B`와 `(Signature)`를 확인한다.

2. `issue_937_f012b_display_text_should_be_signature_seal`
   - `U+F012B` 표시 문자열이 `(인)`이어야 한다는 기대를 고정한다.
   - 현재 결과는 `None`이므로 RED 상태가 정상이다.

## 검증

```bash
cargo test --test issue_937
```

결과:

- `issue_937_bokhakwonseo_signature_cell_contains_f012b` — 통과
- `issue_937_f012b_display_text_should_be_signature_seal` — 실패

실패 메시지 핵심:

```text
left: None
right: Some("(인)")
```

## 결론

Stage 1 목표는 달성했다. 다음 Stage 2에서는 `U+F012B -> "(인)"` 표시 문자열 헬퍼를 추가하고, 일반 TextRun의 SVG/Canvas/Skia 렌더링 및 폭 측정 경로가 같은 치환을 사용하도록 정정한다.

## 승인 요청

Stage 2 구현을 진행해도 되는지 승인 요청한다.
