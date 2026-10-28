# Stage 1 완료보고서 — Task #1562

> HWPX 폼 컨트롤 caption `&&` 표시 정합 — red 테스트 추가

- **이슈**: [#1562](https://github.com/edwardkim/rhwp/issues/1562)
- **브랜치**: `local/task1562`
- **작성일**: 2026-06-26

---

## 1. 수행 내용

폼 컨트롤 caption 표시 문제를 재현하는 targeted 회귀 테스트를 추가했다.

추가 파일:

- `tests/issue_1562_hwpx_form_caption_display.rs`

테스트 대상:

- `samples/hwpx/form-002.hwpx`
- page 0 SVG 렌더 결과

검사 조건:

- `IP R&&D연계` 저장값은 SVG 표시에서 `IP R&D연계`로 보여야 한다.
- `R&&D 자율성트랙(일반)`은 `R&D 자율성트랙(일반)`로 보여야 한다.
- `R&&D 자율성트랙(지정)`은 `R&D 자율성트랙(지정)`로 보여야 한다.
- SVG 출력에 `R&&D` 표시 문자열이 그대로 남으면 실패한다.

SVG XML 문자열 기준으로는 각각 다음 escape 형태를 검사한다.

- 기대 포함: `IP R&amp;D연계`
- 기대 포함: `R&amp;D 자율성트랙(일반)`
- 기대 포함: `R&amp;D 자율성트랙(지정)`
- 금지: `R&amp;&amp;D`

## 2. 현재 기대 상태

현 코드에서는 SVG renderer가 `form.caption` 저장값을 그대로 `escape_xml()`에 넘긴다.
따라서 `R&&D`가 SVG에서 `R&amp;&amp;D`로 출력되므로 Stage 1 테스트는 red가 되어야 한다.

실행 결과:

```text
cargo test --test issue_1562_hwpx_form_caption_display
```

- 결과: 실패(red) 확인
- 실패 지점: `IP R&amp;D연계` 기대 문자열 미검출
- 의미: 현재 SVG 출력이 한컴 표시 문자열이 아니라 저장값 기반 `R&amp;&amp;D`를 그대로 표시함

## 3. 다음 단계

Stage 2에서 renderer 공통 helper를 추가한다.

- `src/renderer/form_caption.rs`
- `src/renderer/mod.rs`
- `src/renderer/svg.rs`
- `src/renderer/web_canvas.rs`
- `src/renderer/skia/renderer.rs`

저장값과 serializer는 수정하지 않는다.

## 4. 승인 요청

Stage 1 red 확인 후 Stage 2 구현으로 진행한다.
