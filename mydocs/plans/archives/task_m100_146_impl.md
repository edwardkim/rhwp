# 구현계획서: text-align.hwp SVG ↔ 한컴 PDF 렌더링 차이 보정

- **타스크**: [#146](https://github.com/edwardkim/rhwp/issues/146)
- **마일스톤**: M100
- **브랜치**: `local/task146`
- **작성일**: 2026-04-23
- **상위 문서**: `mydocs/plans/task_m100_146.md`

## 0. 진입점 요약 (사전 조사 결과)

| 후보 | 파일 | 주요 라인 | 현재 동작 요지 |
|------|------|----------|--------------|
| ① 공백 x-advance 누락 | `src/renderer/layout/text_measurement.rs` | 260–290 (`compute_char_positions`, `char_width`) | 모든 글자에 `letter_spacing + extra_char_spacing` 를 더함. 음수 자간 시 `base_w * ratio * 0.5` 로 min-clamp. 공백 분기 없음 → 특정 조합에서 공백 advance 가 유효 폭에 못 미쳐 사라짐. |
| ② Justify 미반영 | `src/renderer/layout/paragraph_layout.rs` | 902–1042 (`layout_paragraph_internal`) | `needs_justify` 시 `extra_word_sp` 를 `text_style.extra_word_spacing` 으로 전달은 하지만, x_start 계산(1031–1042)은 `Alignment::Justify` 를 `Left` 와 동일 분기로 처리. TextRun 내부 단어 간격만 확대되고 줄 전체가 실제로 양끝까지 퍼지지 않는 상태. |
| ③ Hanging indent 후속 줄 x 기준 | `src/renderer/layout/paragraph_layout.rs` | 735–742 (line_indent) | 음수 indent 시 첫 줄 `line_indent=0`, 후속 줄 `line_indent=|indent|`. 의미상 정확해 보이나 실제 SVG 상 x 기준점이 PDF와 어긋나는 관찰 → LineSeg `text_start` 또는 `render_tree.rs` 에서 line_indent 반영 누락 가능성. |

## 1. 실행 순서 판단

Explore 에이전트 권고(②→①→③)에 동의한다. 이유:
- ②는 가장 광범위한 시각적 영향(모든 Justify 문단). 먼저 해결하면 ①/③ 시각 검증이 쉬워진다.
- ①은 한 문자 단위 이슈로 ②와 독립이며 파급이 작다.
- ③은 ①/② 수정 후 잔존 차이 측정에 유리.

## 2. 구현 단계 (5단계)

### 1단계: 재현 샘플·비교 파이프라인 고정

- `samples/text-align.hwp` 로 이동(또는 복사), git 편입 여부 결정 (용량 32KB → OK)
- `output/compare/text-align/` 비교 스크립트/수동 커맨드 정리: 
  - mutool 로 PDF→PNG (150dpi)
  - Chrome headless 로 SVG→PNG (1.5625× scale factor, 794×1123 window)
- 현재 상태의 SVG를 `output/re/` 에 baseline 으로 보관(참조용, gitignore 대상)
- **코드 수정 없음**. 현상 재현 커맨드와 진단용 dump 캡처만 정리.
- **커밋**: 샘플 추가 + `task_m100_146_stage1.md` 단계별 보고서

### 2단계: Justify 정렬을 SVG에 반영 (② 수정)

- `src/renderer/layout/paragraph_layout.rs:1031-1042` x_start match 문에 `Alignment::Justify` 분기 추가:
  - 마지막 줄(강제 줄바꿈 없이 파라 끝) 은 Left 처리
  - 그 외 줄: `extra_word_sp` 를 **공백 글자 advance 에 누적 반영**하거나, 글자 x 좌표 전체 재분배
- `text_style.extra_word_spacing` 실제 소비 경로 검토 (text_measurement 까지 전달되는지) — 전달 안되면 TextRun → 글자 x 계산 함수까지 전파
- Sub-task:
  - 마지막 줄 판정 로직 확인 (`paragraph_layout` 상 line_idx 와 total_lines 관계)
  - 강제 줄바꿈(`\n`) 이후 줄은 Justify 대상에서 제외하는 HWP 관행 확인
- 검증:
  - `text-align.hwp` 본문 첫 문단 라인 1("…시범")의 "시범" 오른쪽 끝이 body right margin에 근접
  - `cargo test` 스위트 영향 파악, svg_snapshot 차이 정량화
- **커밋**: ② 수정 + 단위 테스트 (가능하면) + `task_m100_146_stage2.md`

### 3단계: 공백 x-advance 누락 수정 (① 수정)

- `src/renderer/layout/text_measurement.rs:260-290` `char_width` 로직을 공백 친화적으로 수정:
  - 공백 문자(U+0020, U+00A0 등) 판별 시 `extra_char_spacing` 감소를 **공백 폭까지는 절대 갉아먹지 않도록** 하한을 공백 기본 폭으로 보정
  - 또는 CharShape.spacing 적용을 "비공백 글자 간 간격"으로 한정 (HWP 원래 의미에 더 가까울 수 있음 — HL 명세/동작 재확인 필요)
- 단, 2단계에서 Justify 가 `extra_word_spacing` 을 공백에 적용한다면 그 파이프라인을 손상시키지 않도록 주의
- 검증:
  - 제목 "□ 국어…" 에서 □ → 국 x 간격이 약 31px (≈ em + space) 수준으로 증가
  - 본문 단어 간 공백도 가시 폭 유지
- **커밋**: ① 수정 + 단위 테스트 + `task_m100_146_stage3.md`

### 4단계: Hanging indent 후속 줄 기준점 검증·보정 (③ 검증)

- 2·3단계 수정 후에도 잔존 차이가 있는지 150dpi 비교 이미지로 판정
- 차이 존재 시:
  - `paragraph_layout.rs:735-742` line_indent 를 `render_tree.rs` TextLineNode.bbox 및 x_start 에 반영하는 전체 경로 추적
  - LineSeg.text_start / ls 값과의 의미 일치 확인
  - 필요 시 후속 줄 x = body_left + |indent| 로 명시 계산
- 차이 없을 시: 이 단계는 "현재 구현 정확함" 을 재확인한 것으로 단계 보고서에 기록
- **커밋**: (수정 필요 시) ③ 수정 + `task_m100_146_stage4.md`

### 5단계: 통합 검증 + 회귀 방지

- `cargo test` 전체 통과 확인, `cargo clippy -- -D warnings` 통과
- `svg_snapshot` golden 변경이 있으면 사유 정리:
  - 어떤 문서의 어느 좌표가 왜 바뀌었는지 샘플 3개 이상 기술
  - 변경이 정답에 가까워지는 방향임을 비교 이미지로 증빙
- `text-align.hwp` 최종 SVG 와 PDF 150dpi 이미지 나란히 캡처 → `mydocs/report/task_m100_146_report.md` 의 증빙으로 사용
- `samples/` 내 주요 문서 스모크 스위프: 무작위 10개 문서 150dpi 비교 후 명백한 회귀 없는지 확인
- **커밋**: svg_snapshot golden 업데이트(필요시) + 최종 결과보고서 + orders 상태 갱신

## 3. 테스트 전략

- **단위 테스트**: `text_measurement::compute_char_positions` 에 대해 "자간 -8% + 공백" 조합 케이스 신규 추가 (3단계)
- **통합 테스트**: `paragraph_layout` 에 대해 Alignment::Justify + 일반 줄 + 마지막 줄 조합 케이스 (2단계)
- **스냅샷 테스트**: 기존 `svg_snapshot` 유지, golden 업데이트는 5단계에서 일괄
- **시각 회귀**: `samples/` 내 10개 문서 150dpi 비교 (5단계)

## 4. 리스크 대응

| 리스크 | 대응 |
|--------|-----|
| ② 수정이 svg_snapshot 대량 diff 유발 | 단계별 커밋으로 원인 구간 구분, 5단계에서 일괄 golden 재생성 + 사유 기술 |
| ① 공백 처리 변경이 폰트별로 다르게 동작 | 공백 판정을 유니코드 카테고리(`Zs`) 기준 + ASCII 공백 화이트리스트로 두고, HWP 관례 주석화 |
| ③ 수정이 "정답"이 아닌 샘플에 회귀 | 4단계 전에 대표 샘플 5개 사전 비교, 수정 전후 diff 이미지 보관 |

## 5. 산출물 체크리스트

- [ ] `task_m100_146_stage1.md` ~ `task_m100_146_stage4.md`
- [ ] `task_m100_146_report.md` (`mydocs/report/`)
- [ ] 샘플 편입: `samples/text-align.hwp`
- [ ] 단위 테스트 2건 이상 추가
- [ ] svg_snapshot golden 업데이트 (해당 시) + 사유 기록
- [ ] orders/20260423.md 상태 갱신 (타스크 진행 중 → 완료)
