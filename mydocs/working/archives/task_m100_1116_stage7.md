# Task #1116 Stage 7 보고서 — 페이지 번호 정렬과 empty lineSeg 재검토

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: 한컴 3mm 안내선 재비교 피드백 반영

## 1. 작업지시자 피드백

- 2페이지 목차의 페이지 번호 오른쪽 정렬이 한컴오피스와 다름.
- 3페이지 내용이 한컴오피스보다 위로 올라가 보임.
- sample16 HWP3-origin HWP5 변환본에서 텍스트가 있는데 `line_segs`가 비어있는 문단 59건을 한컴처럼 계산해서 채울 수 있는지 재검토 필요.
- clean 비교는 다음 명령을 우선 사용.

```bash
target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp \
  -o output/poc/render-spacing/hwp3-sample16-hwp5-page3-grid-3mm \
  -p 2 \
  --show-grid=3mm
```

## 2. 결론

### empty lineSeg 59건

59건은 확인했지만, 기본 파싱 경로에서 모두 계산해 삽입하지 않았다.

이유:

- HWP3-origin HWP5 변환본의 일부 본문 문단은 `PARA_LINE_SEG`가 비어 있어도 기존 renderer/composer fallback으로 페이지 경계가 맞는 legacy 패턴이다.
- 기본 경로에서 강제로 reflow를 넣으면 sample16 계열의 저장된 페이지 경계가 깨져 #1105 회귀 테스트가 실패한다.
- 따라서 이 59건은 사용자 자동 보정 경고 대상에서 제외하고, 기존 fallback을 유지하는 쪽으로 고정했다.

적용:

- `DocumentCore::from_bytes()`에서 HWP 파일이고 `document.is_hwp3_variant`인 경우 `LinesegArrayEmpty` 경고를 제거한다.
- 테스트는 59건이 계속 존재하지만 validation warning으로 노출되지 않는지 가드한다.

### 페이지 번호 정렬

실제 보정 대상은 SVG/Web Canvas의 ASCII 숫자 glyph 폭이었다.

- 기존 SVG 보정은 라틴 알파벳 중심이어서 목차 페이지 번호 숫자가 브라우저 폰트 폭에 따라 흔들릴 수 있었다.
- SVG renderer에서 ASCII 알파벳과 숫자 cluster에 `textLength` + `lengthAdjust="spacingAndGlyphs"`를 부여했다.
- Web Canvas renderer도 ASCII 알파벳과 숫자를 layout advance에 맞춰 축소/확대하도록 보정했다.
- 한글 cluster에는 이 보정을 적용하지 않아 기존 한글 좌표와 폰트 렌더링을 유지했다.

## 3. 산출물

clean 3mm SVG:

```text
output/poc/render-spacing/hwp3-sample16-hwp5-page2-grid-3mm-clean/hwp3-sample16-hwp5_002.svg
output/poc/render-spacing/hwp3-sample16-hwp5-page3-grid-3mm/hwp3-sample16-hwp5_003.svg
```

3mm 격자는 SVG pattern `width="11.3386" height="11.3386"`로 확인했다.
96dpi 기준 3mm에 해당한다.

## 4. 검증

통과:

```bash
cargo fmt --all -- --check
cargo build --bin rhwp
cargo test --test issue_1116 -- --nocapture
cargo test --test issue_1105 -- --nocapture
cargo check --target wasm32-unknown-unknown --lib
wasm-pack build --target web --out-dir pkg
```

추가 확인:

- p2 SVG 목차 숫자 `1`, `2`, `3` 등에서 `textLength="8.6667" lengthAdjust="spacingAndGlyphs"` 확인.
- p3 SVG heading 숫자 `1`, `2`, `3`에서도 `textLength` 확인.
- `wasm-pack build --target web --out-dir pkg`는 prebuilt `wasm-bindgen` 미제공 경고 후 fallback으로 성공.

## 5. 남은 판단

이번 stage는 페이지 번호 정렬과 legacy empty lineSeg 처리 범위를 안정화했다.
3페이지 전체 세로 위치가 여전히 한컴보다 높게 보이면 다음 단계의 원인은 empty lineSeg 기본 삽입이 아니라 저장된 `line_segs.vpos`, 문단 간격, line height 변환, 또는 페이지 body 영역 계산 쪽에서 별도 비교해야 한다.
