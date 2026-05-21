# Stage 1 완료보고서 — #1049 근본 원인 격리 (소스 무수정)

- 타스크: #1049 (M100), 브랜치 `local/task1049`
- 단계: Stage 1 — 근본 원인 격리 (진단, 소스 무커밋)
- 작성일: 2026-05-21
- 대상: 비공개 185p "재정통합 제안요청서" hwpx, page 111(0-base) sec0 pi=781 본문 4.6px 초과
- 검증 권위: `pdf/2. 인공지능(AI) … 제안요청서-2022.pdf` (한컴 2022)

## 0. 요약 (결론 먼저)

**#1046 진단(`_v2_stage3_781diag.md`)의 가설은 반증되었다.** 렌더러는 pi=760 줄높이를
**정확히 20.0px** 로 계산하며(과대 계산 아님), 줄 advance 도 22.03px 로 정확하다. 잔여
4.6px overflow 의 진짜 원인은 **`vpos_adjust`(VPOS_CORR, `height_cursor.rs`)** 가 인라인
TAC 표(pi=758) 직후 `lazy_base` 를 잘못 산출해 pi=760 을 **+12.8px 앞으로 점프**시키는 것이다.

## 1. 재현 (대상·측정)

- 대상 파일은 #1046 과 동일한 **비공개 185p HWPX**(aift.hwp(74p)는 #874 회귀용 별도 샘플).
- `LAYOUT_OVERFLOW=1 export-svg`:
  - page=111 sec=0 **pi=781 FullParagraph overflow=4.6px** (in-scope, 본 타스크 대상)
  - page=61 pi=323 / page=92 pi=567 = page-larger 2건 (범위 외)
- `LAYOUT_OVERFLOW_DRAW` 도 pi=781 line=0 y=1051.5 col_bottom=1046.9 = 4.6px 동일.

## 2. 드리프트 분해 — 렌더러 y vs 페이지네이터 vpos

페이지네이터(typeset, vpos 기반)는 정확. 렌더러(layout) 누적 드리프트가 원인.

| pi | vpos(rel, px) | 페이지네이터 cur_h | 렌더러 시작 y(rel) | drift |
|----|------|------|------|------|
| 757 | 0 | 0.0 | 0.0 | 0 |
| 758 | 31.2 | 31.2 | 31.2 | 0 |
| 759 | 54.5 | 54.5 | 62.1 | **+7.6** (pi=758 TAC 표 과대렌더) |
| 760 | 110.0 | 110.0 | 118.9 | +8.9 |
| **761** | 132.0 | 132.0 | **153.7** | **+21.7** (pi=760 처리 중 +12.8) |
| 762 | 152.0 | 152.0 | 173.7 | +21.7 |
| 781 | 908.0 | 908.0 | 929.7 | +21.7 → 본문 4.6px 초과 |

드리프트는 두 토막: **pi=758 TAC 표 +7.6** + **pi=760 vpos_adjust +12.8** = +20.4(+잔여 ≈21.7).

## 3. 줄높이는 정확 — #1046 가설 반증 (임시 계측 LH_DIAG)

pi=760 "(용역업체 대표자용)" align=Center, line=110%, indent=-13100, char 15pt:
- LINE_SEG: lh=1500 HU(=20.0px), ls=152 HU(=2.03px). run 2개 모두 15pt(id=122, 끝마커 id=450).
  - 끝마커 id=450 은 composer 가 빈 run 으로 제외 → comp_line.runs=1개(fs=20px).
- 렌더러 `corrected_line_height` = **20.0px** (raw_lh=20=max_fs → 재계산 안 함, 정확).
- 줄 advance = line_height(20.0) + comp_line.line_spacing(2.03) = **22.03px** (typeset 22.0 일치).

→ 줄높이 모델·`corrected_line_height`·끝마커·hanging indent·small-% 모두 **무죄**.

## 4. 진짜 원인 — vpos_adjust lazy_base 오산출 (+12.8px 전진)

임시 계측(VADJ): pi=760 진입 시 `vpos_adjust` 가 y_offset 224.67 → **237.47 (+12.80)**.

`RHWP_VPOS_DEBUG` (실제 배치 패스, col_y=105.81):

```
pi=758 path=page  base=5961289  y_in=137.01 end_y=137.01 applied  (정확)
pi=759 (VPOS_CORR 미호출 — 직전 TAC 로 prev_tac_seg_applied=true → vpos 보정 skip)
pi=760 path=lazy  base=5959663  y_in=224.67 end_y=237.47 applied  (+12.8 잘못된 전진)
pi=761 path=lazy  base=5959663  y_in=259.49 end_y=259.49 ...
```

### 메커니즘 (인과 사슬)
1. **pi=758 = 인라인 1×1 TAC 표**(빈 문단). 처리 후 `layout.rs:2538` 에서 was_tac →
   `vpos_page_base=None` 리셋.
2. **pi=759**(제목 "보안 서약서", 32pt, line=130%): 직전이 TAC → `prev_tac_seg_applied=true`
   가드로 `vpos_adjust` **건너뜀**. 드리프트된 sequential y(167.87)에 그대로 배치
   (참값 160.32 대비 +7.6, pi=758 TAC 과대렌더분).
3. **pi=760**: page_base=None → **lazy path** 첫 진입. `height_cursor.rs:131` 식
   `lazy_base = prev_vpos_end − (y_delta_hu + trailing_ls_hu)`:
   - prev_vpos_end = 5969537, y_delta_hu = (224.67−105.81)×75 ≈ 8915,
     **trailing_ls_hu = pi=759 제목 trailing 줄간격 = 960 HU(=12.8px)**.
   - lazy_base = 5969537 − (8915 + **960**) = **5959663**.
   - 올바른 base(=page base pi=757 vpos) = **5961289**. 차이 = 1626 HU(=21.7px).
4. lazy path end_y = anchor + (vpos_end − base)/scale 이므로 base 가 1626 HU 작으면
   pi=760 이후 모든 lazy 문단이 **+21.7px** 앞으로(특히 pi=760 에서 224.67→237.47 = +12.8
   forward 점프, forward 라 클램프 무관 → 무조건 적용). → pi=781 본문 4.6px 초과.

### 핵심: `+ trailing_ls_hu`(Task #1022 v2 trailing-ls bridge) 의 오적용
`height_cursor.rs:120-136` 의 trailing-ls bridge 는 컬럼이 vpos≠0 에서 시작하는 경우
(footnote-01 p1 등) 필요하나, **컬럼 중간 TAC 리셋 직후 lazy 재산출** 에는 pi=759 제목의
trailing 줄간격(960 HU)을 base 에서 빼버려 12.8px 과대 전진을 만든다.

## 5. 백워드 클램프 확인 — 올바른 base 면 무해

`vpos_corrected_end_y` 백워드 허용 = **8.0px**(`MAX_BACKWARD_PX`).
- 올바른 lazy_base(5961289)면 pi=760 end_y=215.78, y_in=224.67 → 8.89px 백워드 →
  8px 초과로 **거부** → y_in 유지(드리프트 +8.9 잔존, sub-threshold). pi=781 콘텐츠 바닥
  1051.5−12.8 = 1038.7 < 1046.9 → **overflow 없음**.
- trailing_ls bridge 만 빼도 lazy_base=5960622, end_y=224.68 ≈ y_in → no-op, 동일 결과.

→ +12.8px 만 제거하면 pi=781 overflow 해소. (잔여 +8.9 는 본문 안, 시각 무영향.)

## 6. 영향 — 구현 계획서 수정 필요

구현 계획서(`task_m100_1049_impl.md`) Stage 2 는 `corrected_line_height`/끝마커/hanging
indent/small-% 줄높이 모델 수정을 가정했으나, **그 축은 무죄로 확정**. Stage 2 의 실제 수정
대상은 **`height_cursor.rs::vpos_adjust` 의 lazy_base 산출**(trailing-ls bridge 의 TAC-리셋
직후 오적용)로 **재정의 필요**. 작업지시자 승인 후 impl 계획서를 갱신한다.

수정 후보(Stage 2 에서 택1·정밀화):
- (a) lazy_base 산출 시 **직전 항목이 인라인 TAC 표였던 경로**에서는 trailing-ls bridge 미적용.
- (b) 인라인(작은) TAC 표 후 `vpos_page_base` 리셋 억제(page base 5961289 유지). — #539 영향 큼.
- (c) lazy_base 가 page-path 로 복원 가능한 경우(직전 컬럼-top page_base 존재) 우선 사용.

회귀 리스크: VPOS_CORR 은 #412/#643/#874/#991/#1022/#1027 가 누적된 핵심 경로 → 골든 SVG
전수 + footnote-01 p1·exam_kor p5 등 회귀 케이스 정밀 검증 필수.

## 7. 검증 재현 명령 (상주 계측만 사용, 소스 무수정)

```bash
LAYOUT_OVERFLOW=1 rhwp export-svg "<185p hwpx>" -o output/poc/ 2>&1 | grep LAYOUT_OVERFLOW
RHWP_VPOS_DEBUG=1 rhwp export-svg "<185p hwpx>" -p 111 2>&1 | grep "VPOS_CORR.*col_y=105.81.*pi=76"
RHWP_TABLE_DRIFT=1 rhwp export-svg "<185p hwpx>" -p 111 2>&1 | grep "LAYOUT_Y.*pi=76"
RHWP_TYPESET_DRIFT_LINES=1 rhwp export-svg "<185p hwpx>" -p 111 2>&1 | grep "TYPESET_DRIFT.*pi=76"
```

(LH_DIAG/VADJ 는 본 진단용 임시 계측 — Stage 1 종료 시 전량 제거, `git diff` clean 확인.)
