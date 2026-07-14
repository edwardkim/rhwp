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

## 3248363_upmu_bunjang.hwpx / 21217935_simsa_jipyo.hwp / 18095317_eogu_geumji.hwp (실문서 — 블록 밴드 필 핀)

- 출처: hwpdocs 코퍼스 `admrul_downloads/문화체육관광부/3248363_[별표 2] 부·전속단체별
  업무분장표(국립중앙극장 기본운영규정).hwpx` / `ordin_downloads/구례군/21217935_[별표 1]
  장기요양기관 지정 심사지표 및 기준(...).hwp` / `law_downloads/해양수산부/18095317_[별표 7]
  어업의 종류별 어구사용의 금지구역 및 금지기간(...).hwp` (원본 그대로 복사).
- #2097 잔존 계열(RowBreak rowspan 블록 통이월) 대표 실결함 문서: 블록이 쪽 하단
  잔여를 초과할 때 plain 블록 컷 walk 가 행 시작 y 를 무시해 fully_consumed 로
  오판 → 분할 기각 → 통이월로 쪽이 절반가량만 참. 수정(행 오프셋 컷 재시도 밴드
  필) 후 3248363 5→**4쪽**, 21217935 11→**8쪽**, 18095317 22→**21쪽** (모두 한글
  2022 COM PageCount 실측 일치, 3248363 은 쪽 2/3 경계 내용의 한글 PDF 글자 단위
  정합 확인).
- 검증: `cargo test --test issue_2097_band_fill`

## 75544_pii_bunseok.hwpx (실문서 — protected 블록 밴드 필 핀)

- 출처: hwpdocs 코퍼스 `opinion_downloads/개인정보보호위원회/75544_(규제영향분석서)
  개인정보 보호법 시행령 일부개정령(안).hwpx` (입법예고 공개문서, 원본 그대로 복사, 159KB).
- 동계열 확장 실결함 문서: RowBreak 표의 rowspan 블록이 내부 hard-break 도
  행합-초과도 없어 **protected** 로 분류(rbrb=false)되면, plain 컷이 진행분을
  내도(fully=false) `allow_block_split` 이 쪽-초과(page-larger) 블록만 허용해
  기각 → 통이월. 75544 rows 8..11: block_h 420.0px > 쪽 2 잔여 79.2px 통이월로
  하단 방치, 하류 만석 전파 끝에 마지막 행 조각 20.3px 가 쪽 4 를 단독 생성
  (67쪽). 한글 PDF/COM 실측은 쪽 2 하단에 rows 8..9 수용(밴드 필) 후 표를 쪽 3
  에서 종료, **66쪽**. 수정(기각 경계 전체로 오프셋 컷 재시도 확장) 후 67→**66쪽**,
  PI↔페이지 630문단 전수 한글 COM 일치 (커밋 사본 기준 COM PageCount 66 재확증).
- 검증: `cargo test --test issue_2097_band_fill`

## 3023771_wichokjang.hwpx (실문서 — 쪽나눔=None 표 fresh-쪽 초과 통째 배치 핀)

- 출처: hwpdocs 코퍼스 `admrul_downloads/소방청/3023771_[별지 1] 위촉장(위험물
  사고조사위원회 운영에 관한 규정).hwpx` (행정규칙 공개 별지서식, 원본 그대로 복사, 10KB).
- r13 스윕 최소 재현 (#2097 이슈 코멘트): 쪽나눔=None + 글자처럼(tac) 전면 표
  2건(4x2, 3x1) — 선언=실측 높이 1005px/987px 가 본문 933.5px 를 초과. rhwp 는
  None 임에도 각 3조각(헤더 64.8px / 본체 926.2px / 꼬리 sliver 14.5px)으로 행
  분할해 2→**6쪽**. 한글은 각 표를 1쪽 통째 + 본문 하단(꼬리말·하단 여백)
  오버플로로 렌더 (한글 PDF 실측: 표 하단 ≈1082px, 용지 1122px 이내).
- 수정: fresh 쪽 초과 None 표는 행 분할 대신 통째 배치 + 오버플로. 오버플로가
  본문 하단 아래 물리 슬랙(용지 경계)을 넘는 미관측 극단은 기존 분할 폴백 유지.
  수정 후 6→**2쪽** (커밋 사본 기준 한글 2022 COM PageCount 2 재확증), rhwp
  렌더 하단 좌표 1068~1079px 로 한글과 정합.
- 검증: `cargo test --test issue_2097_band_fill`

## 17809123_jawonbongsa.hwpx (실문서 — 나란히 TopAndBottom float union 예약 핀)

- 출처: hwpdocs 코퍼스 `ordin_downloads/강서구/17809123_[별표 2] 자원봉사증
  종류(제12조제2항 관련)(서울특별시 강서구 자원봉사활동 지원 조례 시행규칙).hwp`
  (자치법규 별표, 원본 그대로 복사, 420KB — 자원봉사증 예시 그림 4장 포함).
- OVER+ORPHAN_PAGE 계열 대표: 한 문단(pi=8)에 wrap=TopAndBottom vert=문단 그림
  2장이 좌우로 나란히(단 오프셋 82.9mm/3.8mm, 세로 오프셋 31/35px, 높이 359/335px,
  세로 band 겹침) 앵커된다. 페이지네이터 pushdown 이 두 그림 높이를 **합산** 예약해
  page1 used 를 1226px(본문 905.6px 초과)로 부풀려 trailing 빈 문단 pi=9 를 여분
  페이지로 밀었다(2쪽). 렌더는 두 그림을 정상적으로 나란히 배치(SVG bottom 958/979px,
  본문 이내) — 회계만 이중 예약. 한글 PDF/COM 은 전부 1쪽.
- 수정: 같은 문단의 TopAndBottom float pushdown 을 band `[off, off+extra]` 의
  union span 으로 예약. 겹치는 2번째+ float 은 증분만 가산(세로 스택=비겹침
  float 은 종전대로 합산, 단일 float 은 union=extra 라 동작 불변). 수정 후 2→**1쪽**
  (커밋 사본 COM PageCount 1 재확증), PI↔페이지 전수 일치.
- 검증: `cargo test --test issue_2097_band_fill`

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
