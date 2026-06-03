# Task #1116 Stage 8 보고서 — 목차 페이지 번호 정렬 완결

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: 목차 페이지 번호 오른쪽 정렬 보정 완료

## 1. 범위

작업지시자 지시에 따라 p3 세로 위치 분석은 보류하고, p2 목차 페이지 번호 정렬만 우선 완결했다.

비교 산출물:

```bash
target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp \
  -o output/poc/render-spacing/hwp3-sample16-hwp5-page2-grid-3mm-clean \
  -p 1 \
  --show-grid=3mm
```

## 2. 원인

목차 문단의 탭은 `tab_extended`가 `ext[2]=3` 형태로 들어와 기존 코드가 이를 리더 있는 오른쪽 탭이 아니라 저장된 inline tab 폭으로만 처리했다.

그 결과 짧은 목차 항목은 우연히 맞았지만, `Consolidation`, `DR`, `S/W`처럼 앞 텍스트 폭이 한컴과 달라지는 행에서는 페이지 번호의 오른쪽 끝이 흔들렸다.

수정 전 p2 SVG에서 페이지 번호 오른쪽 끝 범위:

```text
min=627.08px, max=639.20px, spread=12.12px
```

## 3. 구현

- inline tab 뒤 suffix가 ASCII 숫자 페이지 번호인 경우, 저장된 inline tab advance 대신 TabDef의 오른쪽 리더 탭 기준선을 사용한다.
- 탭과 페이지 번호가 같은 run에 있는 경우는 `text_measurement.rs`에서 문자 위치와 폭 계산을 함께 보정한다.
- 탭 run과 숫자 run이 분리된 경우는 `paragraph_layout.rs`에서 다음 숫자 run을 같은 기준선으로 이동한다.
- 숫자 앞 선행 공백은 보이지 않더라도 정렬 폭에는 포함되도록 처리했다.
- 기존 목차 leader 선은 다음 실제 텍스트 시작점 앞에서 끊기도록 유지했다.

## 4. 검증

수정 후 p2 SVG 페이지 번호 오른쪽 끝 범위:

```text
min=636.827px, max=637.493px, spread=0.667px
```

통과:

```bash
cargo fmt --all -- --check
cargo test --test issue_1116 -- --nocapture
cargo test --test issue_874_ktx_toc_page_number_right_align -- --nocapture
cargo test --test issue_1105 -- --nocapture
cargo check --target wasm32-unknown-unknown --lib
wasm-pack build --target web --out-dir pkg
git diff --check
```

참고:

- #874 테스트의 기존 `LAYOUT_OVERFLOW` 진단 1건은 계속 출력되지만 테스트는 통과한다.
- `wasm-pack build --target web --out-dir pkg`는 prebuilt `wasm-bindgen` 미제공 경고 후 fallback으로 성공했다.
