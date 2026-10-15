# 분할 저장 셀의 프레임 회귀 입력

- 원본: `samples/issue1949_giant_cell_nested_tables_perf.hwp`.
- 이 파일은 과거 native 공개 1×2 셀 분할→HWP 저장 경로에서 생성한 저장본이다.
  현재 생성본으로 대체하지 않는다. 서로 본문/컨트롤은 같아도 저장 줄 캐시는 다르다.
- SHA-256: `d9bc6b13d54f5179c2cf57e6a2310a28efce4f8f6d6e74137a5828c9878a24b6`.
- 독립 한컴2024 Print: `pdf/issue4138-tab-period-split-20261015-2024.pdf`,197쪽,
  SHA-256 `687f1404e56a764258c93fb0e59d9fb1dd870b506d36451fbfa833fd207f8294`.
- 해당 입력의 Native/fresh WASM 전197쪽 최저90.24017%, 미달0쪽이다.
  새 검사는25/60/89/156쪽의 저장 텍스트 컷과 개체 앵커·내용 소속을 보호한다.
  모두97% 안팎이므로 절대 px를 고정하지 않는다.
- source/producer·원본/현재 생성본 대조·전쪽 TSV·review:
  `mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/reparsed-split-cache-20261016/validation.json`.
- 마지막197쪽의 기존99.11469% px 회귀는 원래 공개 생성·저장 검사를 유지한다.
  전체 PR의 devel 대비 시각 하락·수식 기호 미세 차이 등의 잔여는 stage5 C43에 별도 기록한다.
