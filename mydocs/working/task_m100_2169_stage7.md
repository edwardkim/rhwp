# Task M100 #2169 — Stage 7 완료보고서 (PDF 육안 대조: 제4축 = 줄나눔 기준 '글자')

**날짜**: 2026-07-10 / **브랜치**: `local/task2169`

## 1. 한글 PDF 육안 대조 (80168 자체 생성 PDF 157쪽 = 공식 PDF 일치, 47쪽)

r10 셀(11.비용편익분석) 5줄의 줄 경계:
- L3 끝 "…부동산개발업 또" / L4 시작 "는 주택건설사업…" — **어절 '또는' 글자 분리**
- L4 끝 "…위해 필요" / L5 시작 "한 업종…" — **어절 '필요한' 글자 분리**

**한글은 이 문단을 글자 단위 줄나눔으로 조판** — 어절 단위인 우리(6줄)보다
줄당 수용량이 커 5줄. 문자폭·압축 가설이 모두 기각된 모순의 해답.

## 2. 속성 소재

- HWP5 ParaShape attr1 bit 3~4 = 한글(비라틴) 줄나눔 기준 (0=어절, 1=글자).
- IR 은 `break_latin_word`(라틴)만 보존 — **비라틴 기준 미반영** (attr1 에는
  존재). 우리 recompose(split_composed_line_by_width)는 항상 어절 우선.
- 방증: 우리 HWPX export 의 breakSetting breakNonLatinWord="KEEP_WORD" 고정
  (원본 속성 미전달 — 직렬화 손실 의심, #1986 라틴만 처리).

## 3. 다음 단계 (구현)

1. 원본 80168 의 해당 ParaShape attr1 bit3~4 확인 (DIAG or dump-records).
2. composer 줄분할: attr1 줄나눔 기준 '글자' 문단은 글자 단위 분할.
3. HWPX 직렬화 breakNonLatinWord 원문 보존 (파서·직렬화 왕복).
4. 3축+줄나눔으로 페이지 pin 재판정 (80168=157, issue_1891, 21761835, recount).

## 4. r11/r6 분해 (추기, stage8 후)

- PDF 좌표 실측(fitz bbox): r10 한글 행높이 85.9pt = 우리 114.5px 정합 확인. r11은 47쪽 최하단 경계 행(대분류/소분류) - +14.1은 페이지 경계 분배 특이 케이스 의심, 우선순위 하향.
- 잔여 본류 = r6형 중첩 셀 -2.2 계통 (다음 분해 대상).
- 4축 세트 현황: pi271 r4=231.9 정확, pi400 r10=114.5 정확, 80168 153쪽(잔여 과소축 노출).

## 5. r6형 -2.2 해명 (추기)

anchor 사다리(TAC nested, outMargin=0): anchor 빈 문단 몫 = **0** (row = nested+pad 정확, 10/11pt 동일). 80168 nested도 treatAsChar=1, outMargin top/bottom 283+283=7.5px.

검산: 한글 r6 = 텍스트 46.9 + nested 99.7 + **outMargin 7.5** + pad 6.0 = 160.1 (관측 160.2). 우리 158.0 = outMargin 미반영 + anchor placeholder 5.33 오부여.

**수정 사양**: NO_LS 셀 중첩 가산에서 (a) nested 몫 = total + outMargin(top+bottom), (b) TAC nested anchor 빈 문단 placeholder = 0.

## 6. 5축 게이트 판정 (stage9)

- pi=271/400 전 규명 행 한글 정확 일치 (r4 231.9/r6 160.2/r10 114.5/r6' 194.9).
- 게이트: 1623/1842/byeolpyo4 통과, **1891 실패**(80168 153 vs 157), 21761835 cutΔ한글 +10.3 -> **-82.9 과소 전환**, recount IMP1/WOR5.

**판정: 일괄 정식화 보류.** 글자 채움(kbu==1)이 21761835 계열에서 과소 유발 - 그 문서의 한글 실측은 어절 래핑과 정합했으므로 kbu 단독 조건 불충분(문서/문단별 실동작 차 - HWPX 왕복 손실 or 부가 조건). 후속: (a) 21761835 문단 kbu 실측+PDF 육안으로 글자채움 적용 조건 정밀화, (b) 문서 무관 참인 축(outMargin/anchor-0, 빈문단 em)별 개별 recount 검증으로 부분 랜딩 경로.

## 7. 21761835 재분해 (stage10)

- 정렬 결합 기각 (kbu2 사다리: KEEP=글자/BREAK=어절, JUSTIFY/LEFT 동일).
- **21761835도 글자 채움 확인** (PDF 3쪽 r40 셀: '응급/구조사' 글자 분리).
- 잔여 회계: r40 한글 178.4 = 9x16.9 + 2x13.3 + 3.8 = 178.7 적합 - **문단별 마지막 줄 em** 모델. 80168 r4는 셀-마지막-만-em(=all em+gap)이 정합 - 두 문서 모델 충돌. 차이 후보: ls% (127 vs 160) 또는 문단 수/빈 문단 개입.
- 다음: em 적용 단위(셀/문단) 판별 사다리 - 2문단x다줄 셀을 ls 127/160 각각으로 실측.

## 8. em 적용 단위 판별 (stage10 추기)

emunit 사다리(2문단x3줄, ls160/127): 실측 123.7/101.9 = **셀별-em 예측(124.3/101.5) 적합**, 문단별-em(115.7/97.9) 기각. 우리 규칙(셀 마지막 줄만 em) 참 재확인.

21761835 r40 잔여(-14.3)는 모델이 아니라 **한글 실제 줄수 미확정**(178.4 역산 비정수) - 다음: PDF에서 r40 c3 줄수 직접 세기(fitz y-범위) 후 우리 글자채움 줄수와 대조.

## 9. 안전 부분집합 게이트 (stage11)

- 구성: 폭(머지) + 중첩 outMargin/max/가산 + anchor-0 + 빈문단 em 승격 (em 마지막줄/글자채움 보류).
- **뷰어 코어(wasm_api CLI): 80168=157 그린**, 359 recount **IMP3/WOR0** (63607 58->59, 10665653 1->2=pdf, 18092931 65->66), 1623/1842/2146/byeolpyo4 통과.
- **1891 실패(156)는 DocumentCore(편집기 코어) 레짐** - CLI(157)와 코어별 페이지네이션 갈림. skia 피처 무관 확인.
- 다음: DocumentCore 페이지 맵 vs 뷰어 맵 diff로 -1 지점 특정(발산 46쪽 pi=400 rows 0..8 vs 0..12) -> 편집기 코어의 해당 축 미반영/이중반영 지점 정합 -> 전 게이트 그린 후 정식화.

## 10. 1891 -1의 최종 정체 (stage12)

- 실패 샘플 = samples/issue1891/80168_...hwpx (**자기-export HWPX 재파싱 픽스처**). .hwp 직파싱은 뷰어/편집기 코어 모두 157 그린.
- HWPX 픽스처: 뷰어도 156, pi=271 r4/r5/r7 = .hwp 대비 **+8.8씩** (synthetic lineseg 경로 줄높이 차), cut_sum 1613.9.
- 즉 부분집합 자체는 .hwp 전 코어 그린 + recount IMP3/WOR0이고, 잔여는 **HWP5<->HWPX 왕복 시멘틱**(is_hwpx_source 이원화 기지 계열 #1770/#1811)의 nested/합성 lineseg 경로 1건.
- 다음: HWPX 합성 lineseg 셀의 nested om/가산 조건을 원본(.hwp) 시멘틱과 정합(#1811 가드 확장) -> 1891 그린 -> 부분집합 정식화.

## 11. PR 재구성 메모 (stage13)

- 07cee586(pr/task2169 스냅샷)은 upstream 최신 델타(main 542/table_layout 540줄 리팩토링, contains_old_hangul_jamo 등)를 클로버해 빌드 깨짐 - force-push로 교체 예정, 리뷰 금지.
- 재구성 경로: upstream/devel을 local/task2169로 merge(3-way 충돌 해소: src 4파일 + add/add 문서) 후 재검증 -> PR 재생성. 델타 패치 백업: output/poc/task2169/t2169_delta.patch.
- 정식화 대상 훅 목록: mod.rs(para_has_no_stored_line_segs), measurer(additive nested_sum max+om / promotion x2 / anchor-0 x2 / max_fs 폴백 x3), table_layout(calc_para para 인자+폴백, cut 폴백, cut promotion, nested-fit strict gate), diagnostics(core-pages), composer(제외 - 비활성 char_break), 픽스처(task2169 2종 + issue1891 80168 재생성).
