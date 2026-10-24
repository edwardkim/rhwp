# #8160 sample16 전 페이지 Print 회귀 계약

## 독립 기대값

`hwpx.json`, `hwp2024.json`, `hwp2022.json`은 각 원본 64쪽과 새 MCP 독립 Print의
SHA-256, 실제 96 DPI 기준선, 원문 내용과 원본 주소를 보존한다. PDF 출력 경로·실제
한컴 engine·job ID는 [MCP 증거](../../../mydocs/pr/assets/issue_8160/mcp-reference-refresh.json),
회귀 추가 전의 384쪽 raw 100%·정수 미대응 0은
[최종 전쪽 증거](../../../mydocs/pr/assets/issue_8160/candidate33-all-backends.json)를 따른다.

PDF의 `get_text("dict")` 줄에서 공백·제어 문자만 제외한 문자열을 읽고, 첫 span의
`origin[1] * 96 / 72`를 기준선으로 측정했다. 같은 쪽에서 원본 내용과 단일하게 대응하는
문구의 앞·중간·뒤를 선정했다. 가시 첫 런의 앞에 공백이 있는 줄은 글자 원점과 전진
프레임을 혼동하지 않도록 선정 대상에서 제외했다. 모든 192쪽에 중요 내용이 있으며
합계 572곳이다. 기대 좌표는 현재 렌더 트리의 bbox에서 복사하지 않았다.

문단·셀·글상자 주소와 문자 시작은 고정한 원본의 주소다. 정식 Rust 검사는 매번 원본을
다시 파싱하고 주소가 가리키는 실제 문단의 문자열과 각 런의 문자 범위를 대조한다.
같은 문구라도 다른 셀·본문·문단으로 이동하면 실패한다. 독립 기준선 ±2px, 해당 쪽의
내용·중복·소유, 원문 순서와 전체 쪽수도 함께 검사한다.
`verified_sample16_all_pages.rs::verify_table_picture_rows`는 이전의 HWPX 18쪽 검사 보호를 세 문서로
확장해 표·그림·표의 원본 컨트롤과 셀 주소·비겹침을 확인한다. 중복 활성 case는 제거했다.

## 실제 전 페이지 시각 회귀

내용 관계 검사와 별도로 `scripts/verify_sample16_raw100.py`가 실제 Native/Canvas2D PNG
384개를 독립 Print PNG와 다시 비교한다. TSV의 반올림 값이나 이전 통과 판정을 쓰지 않는다.
threshold 232·흰색 padding·양방향 2px의 정수 미대응이 단 하나라도 있으면 exit 1이다.
쪽 누락·추가·잘못된 입력/PDF 해시·다른 profile/backend도 거부한다. 전체 가로·세로 paint는
이 비교가 보호하며 layout 전진 상자를 PDF 잉크 외곽선으로 잘못 고정하지 않는다.

기존 Visual Sweep을 같은 원본·독립 PDF·실제 원본 글꼴 프로그램으로 전쪽 실행한 뒤,
`runs.json`에 여섯 실행 폴더를 지정한다. 각 폴더는 `silhouette_manifest.json`,
`run_manifest.json`, `rhwp_png/`, `pdf_png/`를 포함한다. 형식은 다음과 같다.

```json
{"native":{"hwpx":"...","hwp2024":"...","hwp2022":"..."},
 "wasm":{"hwpx":"...","hwp2024":"...","hwp2022":"..."}}
```

```bash
venv/bin/python scripts/verify_sample16_raw100.py \
  --runs-json output/pr-review/<작업>/runs.json \
  --report output/pr-review/<작업>/raw100.json --workers 8
```

같은 검사로 수정 전 candidate31 WASM의 2022 18·35쪽을 실제 검출했다(382/384, exit 1).
candidate33 Native/표준 WASM은 384/384, exit 0이다. 실제 실패 페이지·정수 차이는
[회귀 검출 증거](../../../mydocs/pr/assets/issue_8160/all-page-regression-proof.json)에 있다.
음성 실행의 Native는 동일 대조군으로 유지했으며 이를 같은 source의 수정 전 전체 실행으로
표기하지 않는다. 라이선스가 있는 설치 글꼴은 fixture에 복제하지 않는다.


## 리베이스 후 Print 환경과 JPEG 경계

제품 source `5df67d3299befb467a49dc4eaafd12d6e1877e2d`에서 세 파일 각각 Native와
fresh 표준 최적화 WASM 64/64쪽 raw100·실제 양방향 정수 미대응 0/0을 재확인했다.
[단계 8](../../../mydocs/working/task_m100_8160_stage8.md)의 재측정·직접 판독 기록을 따른다.
좌표 검사는 이 독립 Print와 전쪽 PNG의 프로그램 선택 환경을 사용한다. 기존 572개
앵커·630개 영역의 기대값과 ±2px 공차는 유지한다. 글꼴 바이트를 fixture에 복제하지 않는다.

`hwp2022-header-logo.png`는 같은 새 MCP PDF의 1쪽 96 DPI raster에서
`[52, 64, 185, 102]`를 잘라 얻은 133×38px 독립 기대값이다. 원본/PDF 해시는
`hwp2022.json`의 기존 필드를, 영역·PNG 해시·원래 전쪽 PNG 해시·임계값 232·대칭 2px는
같은 파일의 `browser_raster_regions`를 따른다. 기존 64쪽의 좌표 행은 바꾸지 않았다.

[`print-jpeg-frame.browser.mjs`](../../../scripts/tests/print-jpeg-frame.browser.mjs)는 실제
WASM과 Canvas로 같은 표지를 그리고 원본 그림 두 개·JPEG 인쇄 프레임·64쪽·decode 성공을
확인한 뒤 이 독립 영역과 양방향 정수 미대응 0/0을 검사한다. 설치 글꼴은 필요하지 않다.
기존 `visual-sweep-canvas-fonts.browser.mjs`의 import로 수집하며 CI 개별 호출을 늘리지 않는다.
임시 추출·진단 도구는 정식 검사에 포함하지 않는다.


최신 리베이스 후에도 같은 독립 기대값으로 전쪽을 재검증했다.
[현재 6개 전쪽 TSV·실제 PNG 정수 0/0·정식 검사 결과](../../../mydocs/pr/assets/issue_8160/final-9418/README.md)를
따르며 위 candidate33은 이전 검증 이력이다. 현재 제품과 정식 테스트의 source를 구분해 보존한다.
