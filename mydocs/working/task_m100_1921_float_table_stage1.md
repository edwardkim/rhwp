# Stage 1 완료보고 — #1921/#2004 트랙 A: 부동 표 콘텐츠 페이지네이션

- 구현계획서: `mydocs/plans/task_m100_1921_float_table_impl.md`
- 브랜치: `local/task1921-float-table-pagination` (base: local/devel 통합 = origin/devel R11 + 열린 PR 6건)
- 수정 파일: **`src/document_core/queries/rendering.rs` 단독** (+88/−8, 정규화 전용 — 원본 IR 무손상)

## 1. 수행 내용

계획 Stage 1(계측·측정 실증)을 수행한 결과, 설계한 결합 수정(A+B)이 측정→분할→렌더 3경로에
연쇄 정합되어 **Stage 2(분할 진입·컷 분배)·Stage 3(렌더 정합)의 목표까지 동시 달성**되었다.
분할 스캔·렌더 코드는 한 줄도 수정하지 않았다 — 측정이 참값을 보고하자 기존 SSOT 머시너리가
그대로 올바르게 동작했다.

### (A) 셀 재귀 스택 검출 + 이미지 1장/문단 N분할 재분류
- `para_is_floating_image_stack`: 동일-offset 요구를 **겹침 band**(offset spread ≤ min 이미지
  높이)로 완화 — 156714340 varying offset(0/−3360/−2940…) 대응. 세로로 이미 벌어진 정상
  배치(spread ≥ 이미지 높이)는 제외.
- `reclassify_cell_floating_stacks` 신설: 표 셀 내 스택 문단을 이미지 1장짜리 inline(tac=true)
  문단 N개로 분할(정규화본 전용). `compute_render_normalized`에서 셀 스택 검출 시 적용.

### (B) 분할 문단에 합성 line_seg 부여 (3차 실증 무효 원인 해소)
- 종전 `line_segs.clear()`는 셀 composition 을 placeholder 1줄(400HWPU)로 붕괴시켜 측정 불변
  (871px)의 원인이었다. 각 분할 문단에 `line_height = 이미지 높이(HWPU)` 합성 line_seg 를
  부여 → 셀 측정 `text_height`가 스택 총높이를 자연 반영.
- `corrected_line_height`는 raw_lh ≥ font size 면 원값 유지 — 줄간격 팽창 없음(계측 확인).

## 2. 실증 (RHWP_TABLE_DRIFT 계측)

| 지표 | 수정 전 | 수정 후 |
|---|---|---|
| pi=42 측정 (mt_sum) | 871.9px (저장 높이) | **4310.6px** (이미지 5장 합) |
| 분할 스캔 | 미진입 (원자 배치) | **진입** — fragment 5개, 각 843~870px = 이미지 1장 |
| 컷 경계 | — | start_cut [], [1], [2], [3], [4] — 이미지 문단 단위 정확 분배 |
| 페이지 수 | 4쪽 | **8쪽 = 한글 8쪽 정합** |

### 렌더 검증 (export-svg, output/poc/task1921_floattable/)
- p4~p8 각 쪽 `<image>` 정확히 1개, **5장 전부 상이한 이미지**(data URI 전체 MD5 상이,
  429KB/538KB/551KB/565KB/497KB), y=84~125px(본문 상단), 높이 841~868px(전면급).
- 한글 권위 PDF(survey10k_0708/visual, PyMuPDF 래스터) 대조: p4 = 제목 문단 + 프레임 이미지
  1장, p5~p8 = 프레임 이미지 1장씩 — 구조 완전 일치. 겹침 소거.

## 3. 핀 회귀 (부동표 문서군)

| 핀 | 기대 | 결과 |
|---|---|---|
| 156714340.hwp | 8 (한글) | **8** ✅ (4→8) |
| 156714340.hwpx (쌍둥이) | — | **8** ✅ (동반 수정) |
| 59043_규제영향분석서.hwp | 42 유지 | **42** ✅ |
| 1790387 PrEP.hwpx | 141 유지 | **141** ✅ |
| 전체 테스트 (`cargo test --release`) | 전량 green | **2948 / 0** (219 스위트, 통합 베이스에 PR #2088 테스트 +2 포함) |

## 4. 유의 사항 (Stage 4 회귀에서 중점 확인)

- band 완화는 **본문 레벨 스택 검출에도 적용**됨(#2006/#1995 게이트 확대) — varying offset
  본문 스택이 새로 재분류될 수 있다. 1790387 불변이 1차 증거이나, **10k 표본 A/B 2500으로
  변화 문서 전수 판독 필요**.
- 분할 문단은 원본 text(제어문자 5개)를 유지한 채 controls 1개만 남긴다 — tac 위치 매핑은
  첫 제어문자 위치로 정렬되며 렌더 검증에서 문제 없음을 확인.

## 5. 다음 단계

- Stage 4: 10k 표본 A/B 2500 + 변화 문서 전수 판독.
- Stage 5: 최종 보고서 + fork push + PR.
