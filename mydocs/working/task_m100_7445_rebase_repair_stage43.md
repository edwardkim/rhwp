# Stage 43: 표 앞의 공백·빈 줄과 표 소유 줄을 구분

- 시작 HEAD: `9440feae26c7420213502f9482e04f0a6eb6c641`.
- 선행 분석: 원 #7491 검사 `3f30bc78fbb`와 캡션 줄 소유 보정 `3aaecd070c753e22a651cee36bdd3e7a5e03e4dc`를 대조했다. 공백 두 칸·명시 빈 줄은 각각 독립 첫 줄을 소유하고 표는 다음 줄이다. 기존 실패2건은 리베이스 전에도 있었으며 기대값은 유지했다.
- 실제 원인: `owned_row::prepare_occupied`가 가로 pen과 개체 줄 높이 일치만으로 OwnedRow를 반환했다. 그 결과 `typeset_tac_table::before_text`는0으로 바뀌고, 측정/metadata의 표 원점이 앞줄 원점에 놓였다. 앞줄 자체는 별도 PartialParagraph로27.6266667px 전진했지만 표의 확정 원점은 이미 앞쪽이었다. 표시되는 글자가 있는 dot+개행은 해당 경로를 타지 않았다.
- 공통 보정: 첫 TAC 앞에 실제 일반 문단 줄이 있으면 표 한 줄 전용 소유 결과로 그 prefix까지 대체하지 않고 기존 문단 구성·전진 경로를 사용한다. 앞 TAC가 이미 소비한 개체 줄은 별도 소유 경로를 유지한다. 판정은 원시 제어 슬롯과 기존 `ordinary_tac_prefix_rows`를 사용하며 문서 ID/쪽/좌표 상수 분기는 없다.
- 소비: 실제 제어 줄 소유 → owned row의 범위 → before_text/가용 예산 → 공통 inline box 확정 원점 → PageItem/metadata → `layout_table_control_block`의 실제 표 원점. 표 위 바깥여백은 기존 경로에서 한 번 소비한다.
- Native 기존 검사: 두 FAIL → PASS. #7490 전 입력/저장 왕복, 캡션 번호·옆 캡션·저장 소유 줄, #7998 prefix 번호, #7679 뒤 표, #7330 단일 출력 대조군을 합쳐 실제29 PASS/0 FAIL(2.011초). 검사 수·기대값·기준 파일은 변경하지 않았다.
- 독립 Print: 현재 공개 편집 API로 공백·개행·dot+개행3개 HWP를 생성했다. `lastSavedWith=hancom-office-2020/11.0.0.2129`를 확인하고 engine2020/1800초로 Print했다. 3건 모두 한컴11.0.0.9136 성공,1쪽. 같은 실제 직렬화 바이트임을 live/reopened 각각 다시 확인했다. 첫 진단에서 PDF 벡터 선을 조회했지만 draw 경로0이어서 좌표 근거로 사용하지 않았다. 실제 PDF 래스터와 source 줄/개체 소유를 사용했다. 새 PDF는 ignored output의 진단 대조군이며 새 기준/회귀 입력으로 활성화하지 않았다.
- 같은 Native SDK·713개 글꼴·Print·96DPI·PDFium152.0.7947.0/smooth RGB·raw2px: 전6조건(편집 직후/재열기 ×3), before/after 합12쪽. 두 live 미달은85.64856→97.37766%, 표 y631.04→658.6666667px로 복구했다. 두 reopened는99.68821% 유지, dot-live97.37861%·dot-reopened99.68834% 유지. 후보 전조건90% 이상, 수정 전 대비0.1%p 하락0이다. live/reopened에 남는약2.31%p 차이를 동일 출력이라고 주장하지 않는다.
- 직접 판독: spaces live/reopened의 review/overlay를 열어 발표 논문 제목 뒤 문단과 마지막 표 두 행의 분리, 학력·경력의 순서·전체1쪽 소유를 확인했다. 글꼴과 미세 간격 잔여는 유지되며 완전 픽셀 일치나 새 px 기대값을 주장하지 않는다.
- [증적](../pr/assets/issue7445/completed-20261009/stage43-tac-prefix-20261022/): 전조건 TSV, exact input/PDF hash·engine, Native producer, before/after 수치, 대표 PNG.
- 직전 broad 재검사: Native v8는56건 중6 PASS/50 FAIL(47.071초). 50건 원장도 source를 고정해 보존했다. Stage43 이후 전체 실패 수로 재인용하지 않는다. 이번 후보에서는 그중 prefix2건을 집중 재검사로 복구했고 다른 실패 보정을 계속한다.
- 로그/임시 SDK는 ignored `rebase-7752-20261022/{repair-focused-v9,tac-prefix-native-v9,tac-prefix-source-controls,retry-remaining-native-v8}/`에 있다. 진단 SDK의 최초 compile은 private accessor 사용으로 실패했고 공개 source 필드로 정정해 실제 실행했다. 제품 회귀로 집계하지 않는다.
- fresh WASM·최종 전체 회귀·전쪽·실제 소비 경로는 아직 미완료다. PR/push/원격 게시를 하지 않았다.
