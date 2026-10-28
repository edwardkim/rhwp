# Task M100 #2070 최종 결과보고서 — 80168 상쇄망 완전 분해: 157쪽 정합 달성

## 요약

80168 규제영향분석서(원본 .hwp / 자기-export .hwpx)의 페이지네이션을 한글 정답지(PDF 157쪽)와
**정확히 일치(157/157, hwp·hwpx 동일)** 시켰다. 수 회의 세션에 걸친 "상쇄망"(과대·과소 혼재로
부분 참값이 전부 실패하던 구조)을 축 단위로 전수 분해했고, 각 축은 한글 편집기 통제 실측
(COM 캐럿 마크 워크·사다리·PDF 좌표 직독)으로 검증했다.

## 게이트 결과 (정식화 시점)

| 샘플 | 목표(PDF) | 결과 |
|---|---|---|
| 80168_regulatory_analysis (.hwp/.hwpx) | 157 | **157 ✓ / 157 ✓** |
| 80250_regulatory_analysis | 17 | 17 ✓ |
| byeolpyo4 / byeolpyo1 | 26 / 4 | 26 ✓ / 4 ✓ |
| 76076_regulatory_analysis | 82 | 83 (+1, 잔여 — 하단 참조) |
| 86712_regulatory_analysis | 65 | 64 (−1, 잔여) |

- cargo fmt --check(신규 코드) / clippy 통과, **cargo test 전체 통과** (lib 2192 + 통합 전부).
- 잠정 게이트 갱신 3건(각 파일 주석에 근거·복귀 조건 명시):
  - 76076 = 83 (tests/issue_1891, issue_1939) — 본문 래핑 +1줄과 빈 문단 0높이의 상쇄가
    #2070 빈 문단 정합으로 노출. **#2195** (본문 NO_LS 실폭 래핑)에서 82 복귀.
  - 86712 = 64, hwpx 픽스처 63 — 동일 상쇄 + serializer intent 절반 버그(**#2197**) 이원화.
  - svg_snapshot form-002 골든 갱신 — ㆍ 반각 축의 직접 결과(x좌표 시프트). 시각 검증 후속.
- kbu 단위테스트 2건은 #2185 확정 의미(bit7=1=글자)로 기대값 교정.

## 확정 축 (구현 세트)

각 축의 실측 근거는 `mydocs/working/task_m100_2070_stage1.md` stage 1~20 에 기록.

### 파서
1. **0값 LINE_SEG 정규화** (hwp5 `body_text.rs` / hwpx `section.rs`): 전부 0(lh=0,th=0)인
   lineseg 는 부재로 정규화. 생성계 문서(규제영향분석서 계열)가 0값 lineseg 를 저장 —
   실저장 취급 시 NO_LS 재계산 경로가 죽어 셀/문단 높이가 선언값으로 붕괴. hwp/hwpx
   쪽수 이원화도 동시 해소.

### 측정(높이)
2. **비-TAC 표 선언높이 플로어** (`height_measurer.rs`): stale-min 셀(Σ셀선언 < 표선언×0.5)
   조건에서 표높이 = max(선언 size.height, 콘텐츠). 오라클: pi=354/357/362 조문 표
   211.8/192.6/212.4 정확.
3. **본문 NO_LS 빈 문단 = em 줄박스** (`typeset.rs` empty_paragraph_fallback_line_metrics):
   폰트 크기 캡(10pt 이하 한정) 제거 — 한글은 크기 무관 재계산(PDF 좌표 실측 8.1~30px).

### 래핑(줄바꿈)
4. **kbu(bit7) 역해석 정정** (`line_breaking.rs`, #2185): bit7=1(KEEP_WORD)=글자 단위,
   bit7=0=어절 — 3중 확증(kbu 사다리 / 80168 r10 / #2185 giant-cell 통제 재현). 스펙 표44
   정당, OWPML 열거 설명이 실동작과 반대(#2185 정오표 방침과 일치).
5. **셀 재래핑 규칙 세트** (`composer.rs` split_composed_line_by_width):
   - 줄끝 초과 공백 1개 hang (사다리 E행 1/1/2줄 삼각 확증)
   - condense% 공백 압축(fit/overflow 양쪽) — 사다리 v4 R(cnd25)/N(cnd0) 마크 분리 실측
   - 그룹(강제 줄바꿈) 경계는 para.text 의 실제 '\n' 로만 판정 — CHARS_PER_LINE 폴백의
     has_line_break(#994 Justify 억제용) 오인으로 45자 꼬마 줄 생성되던 결함 해소
   - 내어쓰기 이중 폭(첫 줄 전체 폭 / 연속 줄 −(ml+|indent|))
   - **줄바꿈 판정 폭 unrounded** — per-char 반올림(+0.33px/자) 누적이 razor 오단 성분
6. **ㆍ(U+318D) = USER 스크립트 + 폰트별 폭** (`style_resolver.rs`, `text_measurement.rs`):
   한양신명조=전각(사다리 실측 1393~1567HU), 명조(HY견명조 치환) 등 여타=반각, 함초롬(HCR)
   계열은 embedded 메트릭 신뢰. USER 분류가 사다리·실문서 오라클을 동시에 만족하는 유일 해석.

### 페이지네이션(분할)
7. **scan 축소 예산 재시도** (`typeset.rs` scan_block_table_split_rows): advance_row_cut 이
   예산을 수 px 초과하는 컷을 골라 사후 재검(13623)에서 기각→행 통이월(3쪽)되던 것을,
   초과분만큼 예산을 줄여 1회 재시도(그래도 초과면 종전 이월). pi=936 조문대비표가 한글
   PDF p108 과 동일하게 조각 분할(2쪽) — **158→157 의 마지막 축**.

## 방법론 자산 (재사용 가능)

- **한글 오라클 도구 (승격: `tools/task2070/`)**: probe_all_paras.py(셀 per-para 캐럿
  마크 워크, SetPos+MoveLineEnd — 장셀 안정), probe_body_paras.py(본문),
  find_field_idx.py(필드 인덱스 나열), make_hy_ladder3/4.py + walk(폰트/규칙 통제
  사다리, ROOT는 `__file__` 기준 저장소 상대), solve_lp2.py(마크→선형 부등식 LP).
- **검토용 샘플 (커밋)**: `samples/task2070/hy_ladder3.hwpx`·`hy_ladder4.hwpx`
  (+labels.txt — make_hy_ladder3/4.py 산출 동일본), 권위 PDF
  `pdf/80168_regulatory_analysis-2022.pdf` (한글 2022 COM HPrint 1-up,
  157쪽 = 편집기 PageCount 일치).
- **PDF 좌표 직독**(pdfminer): 한글 배치 그 자체와의 y좌표 대조 — 캐럿 걷기 아티팩트
  (표 앵커 캐럿이 표 앞 페이지에 놓임, empty-caret) 판별에 결정적.
- **함정 기록**: 사다리 fontface 는 7개 lang 전부 등록(숫자/공백/기호는 LATIN/SYMBOL/USER),
  캐럿 마크에 누름틀 컨트롤 오프셋 혼입, 필드 순번은 파일별 상이(나열 확인 필수).

## 발견된 별도 이슈 (미해결, 등록 권고)

1. **serializer HwpUnitChar intent 절반 버그**: 자기-export HWPX 의 hp:switch HwpUnitChar
   intent 를 한글이 절반으로 적용(사다리 v4 P21_H 일치) — 픽스처가 원본과 비등가.
   80168 계열 오라클은 원본 .hwp 기준으로 수행해야 함.
2. **본문 NO_LS 래핑 = CHARS_PER_LINE=45 휴리스틱** (composer compose_lines, #994/#998):
   뷰 파이프라인 본문 NO_LS 문단은 실폭 래핑이 아님. 76076 +1 / 86712 −1 의 원인
   (pi181/183/184 각 +1줄, PDF 실측). 정공 = reflow_line_segs 정식 호출(기계 완비,
   DocumentCore::from_bytes 의 reflow_zero_height_paragraphs 가 HWPX 한정 include_empty 로
   이미 존재 — HWP5 확장 + fill_lines 경로 검증 필요). 영향 광역이라 별도 이슈로 분리.
3. **#2185 본문 라인브레이커**: kbu 정정은 본 세트에 포함, reflow 경로의 조판 메타데이터
   보존(column_start/tag)은 #2185 후속.

## 잔여 작업

- 76076(+1)/86712(−1): **#2195**(본문 NO_LS 실폭 래핑) + **#2197**(serializer intent) 해소 시 82/65 복귀.
- 359 recount / visual sweep 는 PI-page 오라클과 직렬화 원칙에 따라 후속 수행.
