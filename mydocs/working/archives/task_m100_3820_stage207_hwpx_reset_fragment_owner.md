---
kind: working
status: completed
issue_or_pr: 3820
stage: 207
last_verified: 2026-08-14
---

# Stage 207: HWPX reset 전 물리 fragment owner 보존

## 목적

`text_footnote_tail_overpagination.hwpx`가 HWP와 HWP 2020 MCP 기준 PDF의 242쪽이
아닌 248쪽으로 분리되는 회귀를, 상수 allowance 없이 저장된 물리 fragment의
owner 규칙으로 해소한다.

## 관찰 근거

- `dump-pages`에서 HWP와 HWPX는 57쪽 `pi=1217` 직전까지 같은 흐름 높이
  `896px`를 가진다.
- HWP는 `pi=1217`의 `lines=0..3`을 57쪽에, `lines=3..5`를 58쪽에 둔다.
- 수정 전 HWPX는 `lines=0..1`, `lines=1..3`, `lines=3..5`로 세 조각을 만들며
  가운데 58쪽은 `pi=1217`만 가진 tail-only 쪽이다.
- `internal_vpos_page_break_line()`은 HWPX 일반 텍스트에서 첫 줄이 본문 하단
  70% 이후이고 다음 줄이 `vpos=0` 또는 상단으로 되감길 때만 physical reset을
  선언한다. 이 샘플의 reset은 line 3이다.

## 변경 계약

- HWPX·단일 단·control 없는 문단만 대상이다.
- 현재 조각의 첫 저장 줄이 현재 flow anchor에 맞고, reset 전 줄이 양수 vpos로
  단조 증가하며, 바로 다음 줄이 실제 `vpos=0`일 때만 reset 전 범위를 같은 쪽
  fragment owner로 유지한다.
- 같은 HWPX reset이라도 reset 전 첫 줄의 저장 좌표가 현재 flow anchor와 맞지
  않으면 writer-local cursor다. 이 경우 reset 자체를 physical page break로
  승격하지 않는다. `pi=4726`은 현재 흐름 `754.2px`, 저장 첫 줄 `974.6px`으로
  이 조건에 해당하며, 후속 `pi=4727..4731`과 같은 쪽에 남는다.
- 표·그림·각주·다단·양수 local rewind 및 일반 저장 tail에는 적용하지 않는다.
- 따라서 허용값을 더하지 않고도 저장 physical fragment가 측정 line budget보다
  클 때 발생하던 중간 tail-only 쪽을 방지한다.

## HWP 2020 MCP PDF 증거

두 원본을 원본 용지 설정으로 HWP 2020 `PrintToPDFEx` 1-up 경로로 다시 변환했다.

| 원본 | 보관 PDF | `file` 형식 | 쪽수 | SHA-256 |
| --- | --- | --- | ---: | --- |
| `samples/task1725/text_footnote_tail_overpagination.hwp` | `pdf/issue1733/text_footnote_tail_overpagination-hwp-2020-20260814.pdf` | PDF 1.7 | 242 | `d20505d2af9989c2fb03c9992ba50e01b938a86dba69a613b353f4a7bc22ba61` |
| `samples/task1725/text_footnote_tail_overpagination.hwpx` | `pdf/issue1733/text_footnote_tail_overpagination-hwpx-2020-20260814.pdf` | PDF 1.7 | 242 | `09caa268b790d553ad8aba6935e550d319bd518a8877654058d8639a886e4c2f` |

MCP 응답은 두 변환 모두 `pdf_page_match=ok`, `pdf_editor_pages=242`,
`pdf_pages=242`, `validation=ok`를 반환했다. 각 결과의 `file`은 사용자가 정한
MCP 식별 기준인 `PDF document, version 1.7`이다.

## 회귀 게이트

```sh
cargo test --profile release-test --test issue_1733 --test issue_1695 \
  --test issue_3820_rowbreak_rowspan_band -- --nocapture
```

`issue_1733`은 HWPX 242쪽과 함께 57쪽의 `pi=1217 lines=0..3`, 다음 쪽의
`pi=1217 lines=3..5` 및 `pi=1218` 배치를 확인한다.
