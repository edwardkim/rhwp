# 단계4 완료 보고서: 통합 검증 + 스모크 스위프 + 최종 보고서

- **타스크**: [#257](https://github.com/edwardkim/rhwp/issues/257)
- **마일스톤**: M100
- **브랜치**: `local/task257`
- **작성일**: 2026-04-23
- **단계**: 4 / 4

## 1. 통합 테스트

| 테스트 | 결과 |
|--------|------|
| `cargo test --lib` | **937 pass / 14 fail** (14건 본 PR 전부터 실패) |
| `cargo test --lib text_measurement::` | **22 pass** (신규 4건 + Task #229 회귀 4건 포함) |
| `cargo test --lib renderer::` | **285 pass** |
| `cargo test --test svg_snapshot` | **3 pass** (form-002 golden 재생성 완료) |
| `cargo clippy --lib -- -D warnings` | **clean** |

사전 존재 14건 실패(`serializer::cfb_writer` · `wasm_api`) 는 단계 1 baseline 에서도 동일 재현. 본 타스크 수정과 무관.

## 2. 스모크 스위프 (narrow glyph 다수 샘플)

각 샘플 전체 페이지 SVG 렌더 후 `<circle>`·`<text>·</text>`·`<text>,</text>` 집계:

| 샘플 | 페이지 | circle(`·`) | text(`·`) | text(`,`) | 특이사항 |
|------|-------|------------|----------|----------|---------|
| `biz_plan` | 6 | **591** | 0 | 13 | TOC 리더 도트 다수, 본문 `·` 균일 렌더 |
| `exam_kor` | 25 | 0 | 0 | 326 | 콤마 다수, 회귀 없음 |
| `exam_eng` | 11 | 0 | 0 | 265 | 콤마 다수, 회귀 없음 |
| `exam_math` | 20 | 0 | **1** | 99 | 1건은 수식 렌더 경로 (Latin Modern Math 폰트), draw_text 미경유 → 정상 |
| `footnote-01` | 6 | 0 | 0 | 34 | 콤마만, 회귀 없음 |
| `field-01` | 3 | 0 | 0 | 0 | 해당 글자 없음 |

**검증**: 모든 `·` (U+00B7) 가 `<circle>` 로 렌더되며 `<text>` 로 남은 1건은 수식 경로(Latin Modern Math / STIX Two Math / Cambria Math 폰트)로, `draw_text` 와 다른 경로. 본 fix 의 범위 밖(수식은 자체 math font 가 있어 `·` 위치 정상).

## 3. 시각 비교 (PDF 150dpi ↔ SVG 150dpi)

### 3.1 `samples/text-align-2.hwp`

- `mutool convert -O resolution=150 -o output/compare/text-align-2/pdf-%d.png samples/text-align-2.pdf`
- Chrome headless 150dpi (`output/compare/text-align-2/svg-chrome150.png`)

**관찰**:
- `어휘·표현` 표 헤더: PDF·SVG 모두 `·` 중앙에 위치
- `1,000항목` `30,000항목`: PDF·SVG advance 동등
- `세대별·지역별` 본문: PDF 와 시각 근사 (rhwp 는 벡터 circle, PDF 는 휴먼명조 글리프 — 시각 인상 유사)
- `시·청각장애인의` 본문: 동일
- 나머지 레이아웃 (여백·장평·섹션 간 간격·표 위치) 은 Task #146 수정분 그대로 PDF 와 수렴

### 3.2 기타 샘플 (시각 확인)

- `biz_plan` 1~4 페이지: TOC 리더 도트·본문 `·` 모두 정상. "온·오프라인", "문화적·교육적" 등 `·` 중앙 배치

## 4. 산출물 확인

| 파일 | 상태 |
|------|------|
| `src/renderer/layout/text_measurement.rs` | 단계 2 수정분 반영 |
| `src/renderer/svg.rs` | 단계 3 `·` `<circle>` 분기 반영 |
| `tests/golden_svg/form-002/page-0.svg` | 재생성 완료 |
| `samples/text-align-2.hwp`, `.pdf` | 단계 1 편입 |
| `mydocs/plans/task_m100_257.md`, `_impl.md` | 단계 1 작성 |
| `mydocs/working/task_m100_257_stage{1,2,3}.md` | 단계별 보고서 완료 |
| `mydocs/working/task_m100_257_stage4.md` | **본 보고서** |
| `mydocs/report/task_m100_257_report.md` | **최종 결과보고서** 신규 |
| `mydocs/tech/text_align_2_svg_pdf_compare.md` | 사전 조사 (단계 1 갱신) |
| `mydocs/orders/20260423.md` | Task #257 진행 상태 추가 |

## 5. 요청 사항

모든 단계 완료. 최종 결과보고서 `mydocs/report/task_m100_257_report.md` 승인 후 `local/task257` → `local/devel` 머지 요청.

(커밋 squash 여부 — A안 시도 2건을 남길지/정리할지 판단 필요)
