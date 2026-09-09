# Stage 231: native RowBreak host line source frame

## 목적

전체 integration 회귀에서 발견된 Issue #2020 FSC HWP의 6쪽 과다 조판을 해결한다.
HWP와 HWPX 원본 모두 한글 기준 5쪽이며, HWP의 `pi=24` 14×15 RowBreak 표는 2쪽
하단 source frame에서 시작해야 한다.

## 분리 결과

- 최신 `upstream/devel`의 `issue_2020`은 4건 모두 통과한다.
- Stage 230 직전 revision도 같은 #2020에서 6쪽으로 실패했다. 따라서 Stage 230의
  HWPX object-frame provenance 제한과는 독립된 누적 회귀다.
- HWP/HWPX의 `pi=24` host LineSeg와 table 선언 geometry는 같다.
  HWP의 저장 anchor line은 `611.08..627.08px`, object top/bottom은
  `614.40..933.55px`, 현재 flow는 `607.4px`다.
- HWP 경로는 `DIAG_ADVC`로 표 전체를 이월했지만, HWPX는 같은 저장 frame에서
  RowBreak scanner에 진입해 `320.9px`를 2쪽에 소비했다.

## 원인

native HWP 예외는 object top이 현재 flow보다 앞서는 경우만 fragment scan을 허용했고,
rowspan이 있는 FSC 표는 제외했다. 실제 저장 object는 host LineSeg 내부에서 시작할 수
있으므로, 현재 flow보다 조금 뒤의 object top도 같은 source line 안이면 별도 fragment의
시작점을 뜻한다. 이때 declared overrun으로 통째 이월하면 HWP/HWPX page owner가 갈린다.

## 구현

- native 비-TAC TopAndBottom RowBreak 표에서 다음 host의 source reset, cell 내부 reset
  부재, 표 각주 부재, 저장 object bottom의 현재 body 포함을 모두 요구한다.
- 현재 flow와 object top이 같은 비합성 host LineSeg 안에 있을 때만 early declared defer를
  건너뛰고 기존 RowBreak scanner에 맡긴다.
- HWPX, 내부 reset 표, 일반 float, 표 각주 및 host line 밖 anchor는 기존 경로를 유지한다.
- 고정 px allowance나 문서명·페이지·행 번호 기반 selector는 추가하지 않았다.

## 검증 범위

- `issue_2020`: HWP/HWPX 5쪽 및 HWP p2 `pi=24` table owner.
- `issue_1921_59043_pagination_pin`: 한글 2022 37쪽과 p8/p11/p12/p35-p36 containment.
- #3820 집중 게이트: `issue_2006_1790387_prep_pagination_pin`,
  `issue_3820_rowbreak_rowspan_band`, `issue_3930_hwpx_hwp_save_layout`, `issue_1733`.
- 집중 게이트 후 전체 `--lib`과 `--tests`를 다시 수행한다.
