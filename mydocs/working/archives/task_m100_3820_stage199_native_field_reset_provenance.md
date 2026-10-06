# #3820 Stage 199: native HWP Field reset provenance

## 관찰

`issue_1035_alignment`에서 HWP5 sample16 원본과 2022 변환본의 쪽수가 각각 65쪽,
66쪽으로 증가했다. 기준은 둘 다 64쪽이다. `upstream/devel`부터 Stage 198까지 같은
단일 테스트로 이분 탐색한 결과, 최초 회귀 커밋은 `cbc323fc`(`저장 line reset을 공통
계약으로 판정`)이다. 직전 `8d9c1bc`는 64쪽으로 통과한다.

## 원인

`cbc323fc`는 과거 sample16 전용 예외를 제거하면서 `Field`와 hyperlink를 모두 인라인
텍스트 메타데이터로 일반화했다. 그 결과 native HWP5의 Field 문단에 있는 저장 vpos
되감김과 누락 LINE_SEG tail까지 물리 페이지 경계로 판정했다. HWPX는 Field가 저장
flow reset을 나타낼 수 있지만, native HWP5의 Field는 같은 의미를 보장하지 않아
source flow가 조기 분할됐다.

## 변경

저장 형식 provenance를 `internal_vpos_page_break_line`과
`missing_lineseg_trailing_line_break`에 명시한다. HWPX 저장 레이아웃과 HWP3 원본만
Field/Hyperlink 기반 vpos reset과 누락 LINE_SEG tail을 사용한다. native HWP5는
Field/Hyperlink가 있거나 음수로 되감긴 저장 좌표만으로 일반 물리 페이지 경계나
LINE_SEG 없는 tail을 소유하지 않는다. 문서명이나 페이지 수, 임의 허용치에 의존하지
않는다.

## HWP 2020 증적

새 KoPub 글꼴이 반영된 HWP 2020 MCP에서 다음 두 입력을 다시 PDF로 변환한다.

- `samples/2025 행정업무운영 편람(최종).hwp`
- `samples/2025 행정업무운영 편람(최종).hwpx`

완료한 PDF는 각각 PDF 1.7, 383쪽인지 확인하고 기존 HWP 2020 증적의 SHA 및 쪽수와
비교해 이 Stage 커밋에 포함한다.

- HWP: `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2020.pdf`
  - PDF 1.7, 383쪽, SHA-256 `6c7be7602cb92bb9b5e6a0b66e9cd80700fceabeade89fabd1a0fcd32adc4413`
- HWPX: `pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2020.pdf`
  - PDF 1.7, 383쪽, SHA-256 `5c11205cb43ba3a1ca3e607e4019b69a937332526a1b740d3dda754dcc4e3f0a`

두 출력은 모두 HWP 2020 MCP의 `PrintToPDFEx`, `validation: ok`, editor/PDF page
match 383으로 완료됐다. 새 KoPub 글꼴 12종은 Ubuntu `hwp-convert-mcp-kopub`에서
가져와 로컬 `~/Library/Fonts/hwp-convert-mcp-kopub`에 설치하고 fontconfig 캐시를
갱신했다.

## 검증

- `cargo test --profile release-test --test issue_1035_alignment -- --nocapture`
- `cargo test --profile release-test --test issue_3930_hwpx_hwp_save_layout -- --nocapture`
- `cargo test --profile release-test --lib renderer::typeset::tests::saved_line_clears_footnote_area_requires_every_boundary -- --exact`

## 결과

- `issue_1035_alignment`: 4 passed. HWP5 sample16 원본·2018·2022·2024가 모두 64쪽이다.
- `issue_3930_hwpx_hwp_save_layout`: 3 passed. HWP와 HWPX의 #3820 Q&A가 모두 383쪽이다.
- `issue_3820_rowbreak_rowspan_band`: 4 passed.
- `issue2214`: 1 passed, `issue2424`: 6 passed.

전체 integration에서 발견한 HWP spec 176/178 회귀는 이 Stage 이전
`cf61bd4af`에서 시작됐음을 별도 bisect로 확인했다. Stage 200에서 저장 tail fit
계약으로 분리해 해결한다.
