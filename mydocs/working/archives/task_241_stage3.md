# Task 241 - 3단계 완료 보고서: 기타 컨트롤 CTRL_DATA 현황 조사 및 문서화

## 완료 항목

### 기술 문서 작성
- `mydocs/tech/hwp_ctrl_data.md` 신규 작성
  - ParameterSet 바이너리 구조 상세 정리
  - ParameterType 13종 값 목록
  - hwplib 기준 7종 컨트롤별 CTRL_DATA 사용 현황
  - 우리 구현 상태 매트릭스
  - 향후 고도화 대상 정리

### 분석 결론
- Bookmark/Field 외 5종(SectionDef, Table, Picture, Rectangle, GSO)은 **raw bytes round-trip 보존으로 현재 충분**
- 구조적 파싱이 필요한 시점: 해당 컨트롤의 속성을 UI에서 수정할 때
- 현재 가장 시급한 고도화: 표 셀 내 커서 이동 (별도 타스크)
