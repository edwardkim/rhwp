# 최종 결과보고서 — #1994 절대위치 글뒤로 표 겹침

- 이슈: #1994 [Rendering] Text overlap on absolutely positioned textbox
- 브랜치: `fix/1994-behindtext-table-overlap` (origin/devel 기준)
- 재현: `20200830.hwp` (이슈 첨부, 가로 2단 교회 주보), 참조 `issue_1994.pdf`(한글 2022)

## 결과 요약

**3페이지 텍스트 겹침 완전 소거 + 페이지 수 5→4(한글 정합).** 무회귀 0건.

## 근본원인 (계측 확정)

3페이지에 **글뒤로(wrap=BehindText) + 용지(Paper)-앵커 1×1 표** 2개:

| pi | 내용 | vert(용지) | 쪽나눔 | 수정 전 배치 |
|----|------|-----------|--------|-------------|
| 33 | 교역자 명단 | 13.5mm | None | 절대 Y(상단) 정상 |
| 34 | 예배 스케줄 | **134.1mm** | **RowBreak** | 흐름 상단 + 2단 분할 → pi=33 **겹침** |

- 용지-앵커 부동 표의 절대배치 경로(`typeset.rs is_paper_topbottom_block`, Task #321/#1858)가
  **자리차지(TopAndBottom)에만** 게이트되어 있었다.
- pi=34는 **글뒤로(BehindText)**라 이 게이트에서 탈락 → 아래 RowBreak 흐름 분할 경로로 빠져
  절대 Y(134.1mm)를 무시하고 흐름 상단에 2단 컬럼 분할 배치 → pi=33(교역자 명단) 위에 겹침.
- (#2019 ⑤ = Paper-앵커 개체 절대배치 미준수 클래스와 동류이나, BehindText+RowBreak 표로 국소.)

## 수정

`typeset.rs`의 용지-앵커 부동 표 절대배치 게이트를 **글뒤로/글앞으로(BehindText/InFrontOfText)까지
확장**:

- `is_paper_floating_block` = 비-TAC + (TopAndBottom | BehindText | InFrontOfText) + vert=용지.
- 글뒤로/글앞으로는 본문을 밀어내지 않으므로 `is_paper_behind_infront` 는 **current_height sync 없이**
  절대 배치(0 flow 소비) → RowBreak 분할·흐름 배치를 막고 절대 좌표(vert=용지)에 통째 렌더.
- co-anchored 선행 float 판정도 세 wrap 으로 확장.

수정 파일: `src/renderer/typeset.rs` (1파일).

## 검증

### 시각 (한글 2022 PDF 대조)
- 3페이지 export-png ↔ `issue_1994.pdf` 대조: 겹침 소거, 예배 스케줄이 절대 Y(좌측 하단)에 비겹침
  배치 — 참조 PDF와 일치. 페이지 수 4 정합.

### 무회귀 (0건)
- `cargo test --lib` **2143 passed / 0 failed**. `hwpx_roundtrip_baseline` 4/4.
- 표/부동 통합 테스트: `issue_1488_rowbreak`·`issue_1749_saved_bounds`·`issue_1611_footer`·
  `issue_1658_page_bottom`·`issue_1156_chart_column` 전건 통과.
- `svg_snapshot` 8/0, `opengov_corpus_snapshot` 2/0.
- **랜덤 코퍼스 150문서 페이지수 변동 0건** → 용지-앵커 글뒤로 표에만 국소 작용.

### 회귀테스트
- `tests/issue_1994_behindtext_table_overlap.rs`: 페이지수==4 + pi=34 PartialTable 분할 0 assert.
- 픽스처 `samples/basic/issue1994_behindtext_table_20200830.hwp` (hwp5-roundtrip PASS).

## 결론

용지-앵커 부동 표의 절대배치 게이트를 글뒤로/글앞으로까지 확장하여 #1994 겹침을 소거하고
페이지 수를 한글 정합(4쪽)으로 되돌렸다. 저장 무영향, 표/부동 회귀 스위트 전건 그린, 무회귀 0건.
**#1994 해소.**
