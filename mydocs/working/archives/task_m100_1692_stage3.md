# Task #1692 Stage 3 - 하단 빈 문단 페이지 밀림 1차 보정

## 변경 내용

- `typeset.rs` 실제 CLI 렌더 경로와 `pagination/engine.rs` 단위 경로에 같은 보호 조건을 추가했다.
- 조건:
  - 현재 문단이 빈 문단이고 컨트롤이 없음
  - 다음 문단이 saved LINE_SEG 기준 `vpos=0` 재시작으로 새 페이지 본문을 시작함
  - 현재 빈 문단의 vpos가 본문 하단부에 있음
- 이 경우 빈 문단을 표시 페이지로 만들지 않고 `hidden_empty_paras`로 흡수한다.

## 검증

```bash
env CARGO_INCREMENTAL=0 cargo test page_bottom_empty_paragraph_before_vpos_reset_does_not_create_blank_page --lib
env CARGO_INCREMENTAL=0 cargo test --test issue_1692
env CARGO_INCREMENTAL=0 cargo build --bin rhwp
./target/debug/rhwp export-svg samples/SO-SUEOP.hwpx -o tmp/visual-1692-fix3/hwpx_svg
./target/debug/rhwp export-svg samples/SO-SUEOP.hwp -o tmp/visual-1692-fix3/hwp3_svg
```

## 결과

- HWPX: 48쪽 -> 47쪽
- HWP3: 48쪽 유지
- HWPX 4쪽은 더 이상 빈 문단 단독 페이지가 아니고, `pi=91` 본문 페이지로 시작한다.
- HWPX는 PDF 1~42쪽까지 같은 번호로 대응한다.

## 다음 원인

- HWPX 마지막 미주 tail:
  - PDF 기준은 46쪽
  - HWPX는 47쪽에 `pi=1258~1259` 미주 222~223만 남는다.
  - 따라서 후반 미주 조판 밀도 또는 미주 column flow 높이 계산이 PDF보다 느슨하다.
- HWP3:
  - 초반 본문 흐름부터 HWPX/PDF와 다르게 조밀하게 들어가며 48쪽 유지
  - HWP3 LINE_SEG/문단 모양 변환 또는 HWP3 variant pagination 보정이 별도 원인으로 보인다.
