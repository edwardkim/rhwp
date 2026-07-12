# Task #2097 재현 샘플

## none_table_declared_fits.hwpx (합성 — rhwp 선언높이 신뢰 시맨틱 핀)
- 출처: 수작업 합성 (`samples/tac-host-spacing.hwpx` 골격). 실문서 재현원은
  `1730000_selection_report.hwp` (아래, rhwp 2쪽 → 1쪽 = 한글 1쪽).
- 형상: 3행 표(pageBreak=NONE, 자리차지) — 선언 높이 69700HU(929.3px) ≤ 본문 933.6px,
  r1/r2 셀은 저장 높이(1200/500HU)보다 내용 실측이 커서 측정 합 946.2px 로 본문 초과.
  표 뒤 "AFTER TABLE" 문단 1개.
- 결함(수정 전): 측정 fit 실패 → 쪽나눔=None 임에도 행 분할(PartialTable rows 0..2/2..3),
  마지막 행이 2쪽으로 조각.
- 기대(rhwp, 선언높이 신뢰): 표는 1쪽 통째, AFTER TABLE 은 2쪽, 전체 **2쪽**.
- **oracle 주의**: 이 fixture 의 "내용 실측 > 저장 셀 높이" 상태는 rhwp 의 측정
  드리프트(실문서에서 한글 910.6px vs rhwp 954.1px)를 모사한 수작업 값이다. 한글
  편집기는 열 때 셀을 **재실측·확장**하므로 표가 본문을 초과해 통째 2쪽으로 밀리고
  AFTER TABLE 이 1쪽으로 백필된다 — 한글 2020 MCP
  (`pdf/task2097/none_table_declared_fits-2020.pdf`) / 한글 2022 COM
  (`...-2022.pdf`) 공히 p1=AFTER TABLE, p2=표. 한글이 저장하는 실문서에서는
  선언 = 한글 실측이라 이 상태 자체가 발생하지 않으므로, 한글 재변환 PDF 는 이
  fixture 의 정답지가 아니다. fixture 는 rhwp 시맨틱(None 표 행 미분할 + 선언높이
  신뢰) 회귀 핀으로만 사용하고, 한글 정합의 권위 검증은 아래 실문서로 수행한다.
- 검증: `rhwp dump-pages samples/task2097/none_table_declared_fits.hwpx` /
  `cargo test --test issue_2097_none_table_declared_fits`

## rowbreak_midpage_declared_fits.hwpx (합성 — 중간-쪽 RowBreak 선언-fit 핀)
- 출처: `samples/task2105/rowbreak_table_declared_fits.hwpx` 에 HEAD 문단을 추가해
  표를 중간-쪽(cur_h 21.3px)에 배치하고 선언을 68000HU 로 축소한 판. 실문서
  재현원은 `3080901_pii_ledger.hwp` (아래, rhwp 2쪽 → 1쪽 = 한글 1쪽).
- 형상: HEAD 문단 + 3행 RowBreak 표 — HEAD(21.3px)+선언 68000HU(906.7px) ≤ 본문
  933.6px, r1/r2 내용 실측 팽창으로 측정 합 944.9px 이 본문을 11.3px 초과.
- 결함(수정 전): RowBreak 선언-fit 게이트가 쪽 상단(current_height≤0.5) 한정이라
  중간-쪽에서 행 분할 → 마지막 행 sliver 2쪽 조각.
- 기대(rhwp): 중간-쪽에서도 실측 초과(overshoot)가 측정 노이즈 수준(≤16px)이면
  선언 신뢰로 통째 1쪽, AFTER TABLE 2쪽, 전체 **2쪽**.
- **oracle 주의**: none_table_declared_fits.hwpx 와 동일 — 수작업 저장값이라 한글
  재조판 PDF 는 정답지가 아니며, rhwp 시맨틱 회귀 핀으로만 사용한다. 한글 정합의
  권위 검증은 아래 실문서(3080901)로 수행한다.
- 검증: `rhwp dump-pages samples/task2097/rowbreak_midpage_declared_fits.hwpx` /
  `cargo test --test issue_2097_rowbreak_midpage_declared_fits`

## 1730000_selection_report.hwp (실문서 — 한글 정합 권위 검증)
- 출처: hwpdocs 코퍼스 `prism_downloads/새만금개발청/1730000-201800001_D0150013-1-000_
  정책연구과제_선정_결과보고서_새만금신교통특구추진방안연구.hwp` (PRISM 공개 정책연구
  서식, 원본 그대로 복사, 18.9KB).
- #2097 대표 실결함 문서 (PR #2101, COM 3자 오라클): 쪽나눔=None 18행 표 —
  한글 실측 행높이 합 910.6px = 저장 선언 68291HU(910.5px), rhwp 실측 954.1px
  (+43.5px, 내용 실측 팽창 + rowspan 사다리 퇴화 행) → 수정 전 분할 강제로 **2쪽**,
  수정 후(선언높이 신뢰) **1쪽 = 한글 1쪽**.
- 기준 PDF: `pdf/task2097/1730000_selection_report-2022.pdf`
  (한글 2022 COM, Print 액션 1-up 강제 출력 1쪽 = 편집기 PageCount 1 정합).
- 검증: `cargo test --test issue_2097_1730000_real_doc_pin`

## 3080901_pii_ledger.hwp (실문서 — 중간-쪽 RowBreak 한글 정합 권위 검증)
- 출처: hwpdocs 코퍼스 `admrul_downloads/지식재산처/3080901_[별지 2] 개인정보의
  목적 외 이용 및 제3자 제공 대장(지식재산처 개인정보보호 세부지침).hwp`
  (행정규칙 공개 별지서식, 원본 그대로 복사, 50KB).
- #2097 잔존 백로그(중간-쪽 RowBreak) 대표 실결함 문서: 문단 2개 뒤 cur_h
  49.6px 에 배치된 17×4 RowBreak 표 — 선언 61269HU(816.9px)는 잔여 827.3px 에
  fit 인데 rhwp 실측 829.9px(+13px 팽창)가 2.6px 초과 → 수정 전 분할 강제로
  rows 16..17 이 88.3px sliver, **2쪽**. 수정 후(중간-쪽 overshoot ≤16px 선언
  신뢰) **1쪽 = 한글 1쪽**.
- 기준 PDF: `pdf/task2097/3080901_pii_ledger-2022.pdf`
  (한글 2022 COM, Print 액션 1-up 강제 출력 1쪽 = 편집기 PageCount 1 정합).
- 검증: `cargo test --test issue_2097_3080901_real_doc_pin`
