# 머리말 그림 토글의 독립 점유 검증

Issue: #7445, 관련 #3080 / 원PR#3081. 이 단계는 기존 검사 기대값의 독립 재검증이다. 제품 로직을 바꾸거나 기존 품질 하한을 낮추지 않았다. 전체 브랜치·actual36·PR 준비 완료가 아니다.

## 입력과 생성 출처

| 입력 | bytes | SHA-256 |
|---|---:|---|
| `hwp3-sample11.hwp` | 391507 | `51b743b2823a2df9b6fac243f56aebecedbbd02e2a8baad58ffc2e5a4e695f20` |
| `header-tac-on.hwp` | 316928 | `af97b438b67da6a1750f3aff3c33b0655131853473099149711741fc2c7bc4e3` |
| `header-tac-on-independent-2020.hwpx` | 526726 | `b7e4bf6f82c5df8d0faccb096bc855126886d49e4b1922abd9411f48284bec39` |
| `header-tac-on-2020.pdf` | 27739902 | `f963124d16e21a669589a921aaf7090c999d50b9af2a7b42f68e12e1e6184f61` |

원본은 기존 `samples/hwp3-sample11.hwp`(HWP3)다. 공개 `DocumentCore`에서 첫 머리말 그림(section0/para23/header0/innerPara0/control0)을 false→true로 토글하고 `export_hwp_with_adapter_snapshot`으로 만든 HWP5를971에 보존했다. 새 저장본의 SHA를 원본 HWP3의 SHA로 바꾸어 쓰지 않는다. 독립 HWPX는 이 동일 HWP를 한컴2020에서 재저장한 측정 자료로 로컬에만 보존한다.

PDF는 `pdf/header-tac-on-2020.pdf`다. job `4a86ea2c-54c1-4b56-ac9f-1eb0d65a1c8d`, Hancom11.0.0.9136/32bit/directDLLhost, Print method0/hancom2020_pdf_driver_one_up/preprocess none,196.568초,151쪽을 확인했다. 재저장 job `56143292-4969-477e-acc0-2ad9e9bc7281`도 같은 한컴 제품이며17.453초다. 재저장은 Print가 아니고 별도의 원문 메트릭 근거다.

재현은 기존 source를 읽은 `DocumentCore`에 다음 공개 호출을 순서대로 적용한다. 임시 저장본은 ignored output에 쓴다.

```rust
core.set_header_footer_picture_properties_native(0,23,0,0,0,r#"{"treatAsChar":false}"#)?;
core.set_header_footer_picture_properties_native(0,23,0,0,0,r#"{"treatAsChar":true}"#)?;
let edited = core.export_hwp_with_adapter_snapshot()?;
```

## 독립 결과와 정규 검사

한컴 재저장본은 본체2448HU·위/아래 여백284HU씩을 유지했다. 첫 줄 line_height/text_height는3016HU, baseline은2564HU다. 기존 검사의 본체2448HU 기대값은 전체 세로 점유를 빠뜨렸다. parser/source 속성과 독립 출력으로 기대값을 정했고 renderer helper를 기대값으로 호출하지 않았다.

기존 `header_picture_tac_on_updates_anchor_line_height` 함수 안에서 원래 false→true setter 경로를 유지했다. 전체 점유·text_height·85% 기준선과 그림 본체/여백 보존, 머리말 문단 텍스트, 본문 구역/문단 텍스트의 순서·보존을 검사한다. 단위 검사 수를 늘리지 않았다.980에서 기존 source는 실제1FAIL, 최종981은1PASS(0.307초)다. manifest prepare/자동 suite 실행, manifest/check·unit-tier/check(base2e3)·fmt/diff도 exit0이다. 초기980의1PASS와 최종981을 중복 합산하지 않는다.

## 전쪽 시각 검증

| 경로 | 쪽수 | 최저/쪽 |90% 미만·누락|
|---|---:|---|---|
|Native960|151/151|93.14319% /50|0/0|
|fresh WASM961|151/151|93.14843% /50|0/0|

976에서 전302개 쪽별 측정을 새로 산출했다. canonical PDFium smooth_rgb_v1/96DPI/Print/compat2022/raw2px90, 기존 공개 글꼴+사용자 Fonts,8쪽 묶음/capture총8/배타lock이다.2·50·151쪽 review/overlay를 직접 판독했다. 머리말 로고·윗선과 본문이 겹치지 않고 끝151쪽의 최종 편집 텍스트·꼬리말·번호도 보존된다.50쪽의 PC/라우터 아래 회색·갈색 면 등 독립 Print와 원본 표시 경로의 기존 차이, 미세 자형·잉크/가는 선 위상 차이는 남는다. 이 기대값 검토로 그 잔여를 새로 해결했다고 쓰지 않는다.2px 실루엣 통과는 엄격RGB 일치가 아니다.

[Native 전151쪽 TSV](native-header-tac-all-pages.tsv) · [fresh WASM 전151쪽 TSV](wasm-header-tac-all-pages.tsv)

- [native 2쪽 review](native-p002-review.png) · [overlay](native-p002-overlay.png)
- [native 50쪽 review](native-p050-review.png) · [overlay](native-p050-overlay.png)
- [native 151쪽 review](native-p151-review.png) · [overlay](native-p151-overlay.png)
- [wasm 2쪽 review](wasm-p002-review.png) · [overlay](wasm-p002-overlay.png)
- [wasm 50쪽 review](wasm-p050-review.png) · [overlay](wasm-p050-overlay.png)
- [wasm 151쪽 review](wasm-p151-review.png) · [overlay](wasm-p151-overlay.png)

## producer·범위

앱 코드는 Stage25의 Native960/fresh961과 같다. 그 뒤9ff65a9e07은 문서 원장 정리이며 이번 변경은 테스트뿐이다. source-side cfg(test) 재배치 전후 runtime prefix 동일성은968, 실제 Studio 초기 JS/WASM 응답은967에 있다. Native CLI SHA `6cf280200468c670b8e06aa821c14d97a56c1562d3e5c1fc705860bdba7c6f05`, JS SHA `63c587e4b072f0a65416e65a0e86bf00b2971cdfb0f46cfda7f150449dd30524`, WASM SHA `0bc3fc93aa2436da75e73a5ee2ff3a5edb9c6006104004a2a412b1620605e341`다. Mac locked web/pkg/no-opt 로컬 대체 빌드이며 Docker 최적화 통과가 아니다. Studio 문서 paint/actual36을 이 PNG/초기화 검증으로 대신하지 않는다.

원본 source와 편집 저장본, Native와 fresh WASM은 구분했다. 마지막 생성 SVG는 완료 후 SHA 원장을 남겨 정리했고 PNG/TSV/PDF/원본은 보존했다. 로그·중간 JSON·생성 HWP/HWPX·probe는 ignored `output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/`의971/975/976/979/980/981/982에 있다. 원본 HWP3 전체의 새 수용이나 다른 저장본의 검증을 이번151쪽 결과로 주장하지 않는다.

[독립 한컴2020 Print PDF](../../../../../../pdf/header-tac-on-2020.pdf)
