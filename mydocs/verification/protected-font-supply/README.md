# 정본 보호 문서의 글꼴 공급 자료

[manifest.json](manifest.json)은 Linux·Windows·macOS의 정본 보호 검증에서 사용하는
문서별 Native·WASM 글꼴 공급 조건이다. 경로는 저장소 루트를 기준으로 해석한다.

- `documents[].backends`에서 문서 key와 backend를 선택한다.
- `supply_mode`가 `explicit_files`이면 `font_set`의 파일명·SHA-256·배열 순서로
  `--font-path`를 지정한다. 자동 선택 조건에서는 `font_set`을 공급 환경 확인에 사용한다.
- Arial 4종, 바탕 TTC, HY신명조는 `accepted_sha256_pairs`에 파일별로 등록한 두 SHA-256을 모두 허용한다.
  공급 목록이 쌍의 한 해시를 지정하면 같은 쌍의 다른 해시도 그 순서와 face에 공급할 수 있다.
  실제 파일을 해시하여 선택한 SHA-256을 기록하고, 같은 공급 조건에서 기준과 후보의 쪽별 하한을 검증한다.
- 문서 항목의 `accepted_sha256_pairs`는 그 문서의 Native·WASM 공급 목록에서만 적용한다.
  화학물질 표시 기준(`chemical`)의 50번째 파일에는 정본 `dotumche.ttf` 또는
  명시된 해시의 `DotumChe-Regular.ttf`를 공급할 수 있다.
- `font_environment`가 있으면 해당 JSON을 `--font-environment`로 지정한다.
- 선택 우선순위는 manifest의 `selection_order`를 따른다.
- `accepted_font_set`, `accepted_font_environment`와 `records/`는 수용 당시의 비교 근거이며, 현재 전쪽 재검증 상태는
  `reproduction_validation`에 기록한다.

`environments/`는 실행에 사용할 환경 JSON, `imports.json`은 이전 기록에서 이관한 파일의
원래 경로·SHA-256과 공식 사본의 경로·`official_sha256`이다. 글꼴 프로그램 바이트는 포함하지 않는다.

저장소 루트에서 다음과 같이 문서가 지정한 SHA-256과 설치 파일을 대조할 수 있다.
검사는 허용된 파일의 실제 SHA-256을 출력한다. 아래는 시행령 문서의 HY신명조를 확인하는 예다.

```bash
python3 scripts/check_protected_font.py 49dd1ef0caa6989a7dd95b484fdd052cbc846f6514c0c08421ed03a6cb1669cc /mnt/c/Windows/Fonts/H2MJSM.TTF
```

문서 한정 허용 쌍을 확인할 때는 문서 key를 지정한다.

```bash
python3 scripts/check_protected_font.py --document chemical 097a83c15227aa0b01bba1c446d2cf67ac04694052e60d41fef557ed5a0a8a62 /path/to/DotumChe-Regular.ttf
```

적용·판정 절차는 [Visual Sweep 가이드](../../manual/verification/visual_sweep_guide.md#정본-보호-문서의-글꼴-공급-재현)를 따른다.
