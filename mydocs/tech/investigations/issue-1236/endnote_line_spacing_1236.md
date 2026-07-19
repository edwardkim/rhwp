---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1236/README.md
last_verified: 2026-07-16
---

# 조사 보고서 — Task #1236 Stage 1: 미주 줄간격 압축 원인

작성일: 2026-06-02
대상: #1236 — 미주(해설) 영역 줄간격이 한컴보다 좁게 렌더

## 1. 증상 재확인

`3-11월_실전_통합_2022.hwpx` 풀이(미주) 페이지(10~14쪽) 줄간격이 PDF 정답지보다 좁다.
12쪽 문19 시각 대조에서 rhwp 줄이 촘촘함 확인. **문제 페이지(1~9쪽)는 PDF 와 정합(정상)**.

## 2. 배제된 가설

- **HWPX 파싱 아님**: `ir-diff 3-11…hwpx 3-11…hwp` → 차이 14건 전부 `indent`,
  ParaShape line spacing 차이 **0건**. 미주 ParaShape 도 `line=150/Percent` 로 동일 파싱.
- **일반 문단 줄간격 정상**: `RHWP_TYPESET_DRIFT_LINES` 로그에서 일반 문단은
  `fmt_lh==seg_lh`, `fmt_ls==seg_ls`, `diff=+0.0` — 적용 줄간격이 원본 LINE_SEG 와 일치.

## 3. 원인 위치 (특정)

**미주 문단은 일반 `format_paragraph` 경로를 우회하고, `typeset.rs` 의 별도 미주 repacking
경로로 배치된다.**

근거:
- 구조: 미주는 문제 문단에 붙은 `Control::Endnote(미주: paragraphs=K)` 의 **중첩 문단**이며,
  렌더 시 문서 끝(10쪽~)에 **가상 삽입**된다(`pagination.rs` `EndnoteParaSource`,
  `endnote_paragraphs`, `endnote_flow`).
- `typeset.rs` L1885+ : 미주 전용 루프. **LINE_SEG vpos-델타 누적**으로 줄/문단 위치 재구성
  (`prev_en_bottom_vpos`, `s.vertical_pos + s.line_height + s.line_spacing`).
  인라인 객체(표) 주변 보정 플래그 존재: `prev_endnote_had_vpos_rewind`,
  `prev_endnote_had_inline_object_vpos_overestimate`, `rewind_group_advance_threshold`.
- **드리프트 로그 미포착**: `RHWP_TYPESET_DRIFT` 로그에 미주 영역(first_vpos>40000) 문단이
  **0건** → 미주는 `format_paragraph`+drift-check 경로를 타지 않음을 역으로 입증.
- 선행 타스크: #836(미주 paragraphs lookup), #1022(`lazy_base` trailing-ls 게이트),
  #1082(다단 미주 vpos-delta 누적) — **반복 패치된 고난도 영역**.

→ 미주 줄간격은 ParaShape 가 아니라 **원본 LINE_SEG vpos-델타**로 재구성되며, 표 인접
보정(rewind/overestimate)이 특정 줄/문단 델타를 좁히는 것이 유력. 사용자 지적("표 인접
좁음·간헐적 불일치")과 정합.

## 4. 회귀 여부

- 본 영역은 #836→#1022→#1082 로 점진 패치된 누적 구조. 단일 회귀 커밋보다 **장기 누적
  근사 한계**일 가능성. (정밀 bisect 는 미주 줄간격 자동 판정 기준이 필요해 Stage 2 로 이월.)

## 5. 미해결 — Stage 2 로 이월

- **정밀 under-spacing 지점**: 미주 각 줄의 렌더 Y 를 PDF 와 1:1 대조해, 어느 델타가
  좁아지는지(표 인접 보정인지, 일반 줄 델타인지) 특정 필요.
- 전역 압축(줄간격 비율) vs 국소 압축(표 인접) 구분 — black-box 픽셀 측정은 수식 첨자
  노이즈로 부정확, **Y-좌표 직접 추출**(렌더 트리/드리프트 로그 확장) 필요.

## 6. 리스크 평가

- 미주 repacking 은 페이지 분할(24쪽 회귀 사례, L1916 주석)·다단 배치와 강결합. 줄간격
  수정이 **페이지 수/단 배치 회귀**를 유발할 수 있어 매우 신중한 게이트 필요.
- 메모리 `tech_lazy_base_trailing_ls_gate`(#1022): 무조건 적용/제거는 양방향 회귀 — 조건
  게이트가 정답. 본 건도 동일 원칙 적용.

## 7. Stage 2 제안

1. 미주 줄 Y-좌표를 렌더 트리에서 추출하는 진단(임시 로그) 추가 → PDF 와 1:1 대조.
2. 좁아지는 델타의 정확한 조건(표 인접 보정 분기 등) 특정.
3. 해당 분기에 **미주 한정·조건 게이트** 수정 + 페이지수/단배치 회귀 가드.
