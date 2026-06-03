# 구현계획서 — Task #1256: 미주 답안 제목 위 between-notes(7mm) 간격 누락

- 이슈: edwardkim/rhwp#1256 · 브랜치: `local/task1256` (from `devel`)
- 수행계획서: `task_m100_1256.md`
- 관련: #1253, #1248, #1246, PR #1232 · 근간 메모리: #1184 절대-vpos 아키텍처

## 0. 정밀 근본원인 (RHWP_VPOS_DEBUG 확정)

수행계획서의 1차 가설(`prev_is_multiline` 게이트)보다 더 구체적인 진짜 원인을 디버그로 확정:

문N) 제목 경계(예 pi=511 문7, prev_pi=510 빈 separator)에서:
- **typeset 은 between-notes 7mm 을 정상 주입** → `prev_ls=1984`, render 의 순차 `y_offset(=y_in)`
  은 7mm 을 이미 포함(500.13).
- 그러나 제목의 **stored 절대-vpos(94604)** 에는 7mm 이 없음(=prev_vpos+1502, 자연 20px gap).
  → `end_y=479.71` = y_offset − 20.42px.
- `compact_endnote_safe_vpos_backtrack`(height_cursor.rs:367-372)가 true →
  분기(407-412)가 **`end_y` 를 반환** → 주입된 7mm 을 버리고 제목을 ~20px 위로 끌어올림.

**즉 진짜 원인 = `compact_endnote_safe_vpos_backtrack`(및 backtrack 류)가 새 미주 제목 경계에서
typeset 이 주입한 7mm between-notes 를 덮어씀.** 영향 제목 시그니처(전 페이지 공통):
`prev_ls≈endnote_between_notes_hu(1984)` + `compact_safe_backtrack=true` + `end_y≈y_in−20.4px`.

페이지별 확인된 대상(idx): p9 pi=499/511/567, p12 pi=696/712/718/734, p13 pi=757,
p17 pi=935/956 … (작업지시자 지목 문번호와 일치).

> 다줄 prev(문22) 시나리오는 별개: render 가 다줄 마지막 seg trailing 을 누락해 y_offset 이
> 7mm 을 **못 가짐** → 기존 line 433 rescue 가 `+prev_ls` 로 보충. 단일 줄 prev 는 y_offset 이
> 7mm 을 **이미 가졌는데** backtrack 이 버리는 것 → 해법이 다름(아래).

## 1. 핵심 correctness 고려 (desync 가드)

제목을 `end_y`(절대) 대신 `y_offset`(7mm 포함)에 두면, 후속 문단의 stored 절대-vpos 흐름이
제목보다 ~20px 위에 있어 **겹침/되감김** 발생 가능(#1184/#1246 desync). 따라서 제목을 아래로
내린 만큼 **`shift_vpos_base_for_rendered_delta(y_offset − end_y)` 로 vpos base 를 이동**해
후속 항목이 동일 기준을 따르게 해야 한다(기존 suppressed_hu base-shift 의 역방향).
이 base-shift 누락이 과거 +132px overflow 회귀의 본질이므로 **반드시 동반**한다.

## 2. 단계 계획

### Stage 1 — 진단 고정 + 회귀 베이스라인
- RHWP_VPOS_DEBUG 로 영향 경계 전수 목록화(스크립트화): `prev_ls≈1984 && safe_backtrack=true`.
- before 캡처: p8/9/12/13/17/18/19/27 SVG→PNG + 한컴 `pdf/3-09월_교육_통합_2022.pdf` 96dpi
  red-header 좌표 비교표(문6→문7 264px → 목표 287px 등).
- 회귀 베이스라인 수치 고정: `3-09월_교육_통합_2022.hwp/.hwpx` 페이지수(23),
  `*-미주사이20.hwp`/`*-구분선아래20.hwp` 페이지수·분기, 문13(의도적 소간격)·문15·문22 좌표.
- `cargo test` green 확인. → `task_m100_1256_stage1.md`

### Stage 2 — 판별자 + backtrack 제외(핵심 수정)
- 판별자 추가(prev seg 에 주입된 7mm 경계):
  ```
  let injected_between_notes = self.endnote_between_notes_hu > 0
      && seg.line_spacing >= self.endnote_between_notes_hu;   // 자연 trailing(~180)·문13 소간격과 구분
  let between_notes_title_boundary = compact_endnote_question_title && injected_between_notes;
  ```
- `compact_endnote_safe_vpos_backtrack`(필요 시 `compact_endnote_deep_backtrack`)에
  `&& !between_notes_title_boundary` 추가 → 해당 경계는 backtrack 하지 않음.
- 결과적으로 result=y_offset(7mm 포함). **동시에** between-notes 경계에서
  `shift_vpos_base_for_rendered_delta((y_offset−end_y).max(0))` 호출(Section 1).
- 단독 커밋(기능 변경). → `task_m100_1256_stage2.md`

### Stage 3 — 한컴 정합 + 회귀 검증
- after 캡처/비교표: 영향 8페이지가 한컴 PDF 와 96dpi 정합(문N) 위 7mm 복원, 문6→문7≈287px).
- 회귀: 페이지수 23 유지(.hwp/.hwpx), `미주사이20`/`구분선아래20` 분기 유지,
  문13/문15/문22 무변경, 다줄 prev rescue(line 433) 영향 없음.
- RHWP_VPOS_DEBUG 로 vpos_base 누적·후속 겹침 없음 확인.
- `cargo test` green. → `task_m100_1256_stage3.md`

### Stage 4 — 단위테스트 + 정리
- height_cursor 단위테스트 추가: between-notes 경계 + 단일 줄 prev(ls=1984) →
  result=y_offset, base-shift 적용. 자연 trailing/문13 소간격은 기존 동작 유지(회귀 테스트).
- 변경 파일만 `cargo fmt`(--all 금지). 이슈 #1256 근본원인 보강 코멘트.
- `task_m100_1256_stage4.md` + 최종 `task_m100_1256_report.md`.

## 3. 리스크 / 대안
- **R1 desync/overflow 회귀**: base-shift 동반으로 완화, Stage3 페이지수·겹침 검증으로 차단.
- **R2 판별자 오탐**(자연 trailing 이 우연히 ≥1984): 미주 흐름 + 제목('문' 시작) + ls≥between_notes
  3중 조건으로 한정. 본문 일반 문단은 endnote_between_notes_hu=0 이라 무영향.
- **대안 A**(채택): backtrack 제외 + base-shift. **대안 B**: line 433 rescue 를 단일 줄까지
  일반화. → B 는 y_offset 이 이미 prev_ls 포함이라 `+prev_ls` 이중가산 위험 → 비채택(단, Stage2
  실험에서 A 가 회귀 시 재검토).

## 4. 검증 커맨드
```
cargo test
RHWP_VPOS_DEBUG=1 rhwp export-svg samples/3-09월_교육_통합_2022.hwpx -p N
rhwp export-svg samples/3-09월_교육_통합_2022.{hwp,hwpx}   # 페이지수 23
rhwp export-svg samples/3-09월_교육_통합_2024-미주사이20.hwp  # 분기 회귀
```

---
승인 요청: 위 4단계로 구현을 진행해도 될까요? 승인 시 Stage 1부터 착수합니다.
