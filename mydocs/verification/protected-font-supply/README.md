# 정본 보호 문서의 글꼴 공급 자료

[manifest.json](manifest.json)은 Linux·Windows·macOS의 정본 보호 검증에서 사용하는
문서별 Native·WASM 글꼴 공급 조건이다. 경로는 저장소 루트를 기준으로 해석한다.

- `documents[].backends`에서 문서 key와 backend를 선택한다.
- `supply_mode`가 `explicit_files`이면 `font_set`의 파일명·SHA-256·배열 순서로
  `--font-path`를 지정한다. 자동 선택 조건에서는 `font_set`을 공급 환경 확인에 사용한다.
- `font_environment`가 있으면 해당 JSON을 `--font-environment`로 지정한다.
- 선택 우선순위는 manifest의 `selection_order`를 따른다.
- `accepted_*`와 `records/`는 수용 당시의 비교 근거이며, 현재 전쪽 재검증 상태는
  `reproduction_validation`에 기록한다.

`environments/`는 실행에 사용할 환경 JSON, `imports.json`은 이전 기록에서 이관한 파일의
원래 경로·SHA-256과 공식 사본의 경로·`official_sha256`이다. 글꼴 프로그램 바이트는 포함하지 않는다.

적용·판정 절차는 [Visual Sweep 가이드](../../manual/verification/visual_sweep_guide.md#정본-보호-문서의-글꼴-공급-재현)를 따른다.
