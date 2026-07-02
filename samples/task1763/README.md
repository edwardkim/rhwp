# Task #1763 재현 샘플

## cell_trailing_ls_expand.hwp
- 출처: 국가법령정보센터 `[별지 4의2] 명의개서자료처리부(증권거래세사무처리규정).hwp`
  (공개 서식, HWP5/OLE, 가로 A4 1쪽)
- 구조: 12×16 TAC 표. row0 셀(rs=1×cs=16, 선언 h=10668HU=142.24px, valign=Center)에
  5개 문단(빈 줄 + 25.3px 대형 폰트 제목 + 텍스트 3줄, 콘텐츠 스팬 10016HU).
- 결함(수정 전): 다문단 셀의 마지막 줄 trailing line_spacing(600HU=8px)이 콘텐츠 높이에
  포함되어 required(149.1px) > 선언(142.2px) → 행 확장 +7px (한글 find_tables 는 142.1px).
- 기대(한글 정합): trailing 제외 콘텐츠+pad 가 선언높이 안 → row0 = 선언 142.2px.
- 검증: `rhwp export-render-tree samples/task1763/cell_trailing_ls_expand.hwp -p 0`
  → Table pi=1 row0 셀 bbox h.
