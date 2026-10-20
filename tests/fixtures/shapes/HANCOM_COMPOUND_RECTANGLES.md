# 한컴 Print로 확인한 닫힌 복합선 대조군

Issue: #7642

## 입력과 독립 기준

| 파일 | 바이트 출처와 SHA-256 |
| --- | --- |
| `hancom-2020-compound-rectangles.hwp` | 정상 한컴 저장본, `e7de32917945045a818d8ac7e32254d6f89b8891f609d2475632aaa2842eca39` |
| `hancom-2020-compound-rectangles.pdf` | 같은 원본의 한컴 Print, `699ac94f9f70f012a8a583a8339e4a3da8203037d6d88695d8f263d32089a717` |

유효한 HWPX 패키지와 실제 원본 사각형 구조를 바탕으로 선 종류·폭·채우기의 대조군을
작성했다. 이 수동 입력 자체를 한컴이 저장한 문서로 간주하지 않았다. Windows의 한컴
2020 COM에서 A3 용지를 실제 PageSetup으로 지정하고 속성을 다시 읽은 뒤 HWP로
정상 저장했다. MCP 2020의 `hancom2020_pdf_driver_one_up`, `pdf_print_method=0`으로
동일 HWP를 Print했다. 실제 제품 빌드는 `11.0.0.9136`이며 전체 1쪽이다.

현재 로컬 증거의 공통 경로는
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`이다.
작성은 `make-stage1913-compound-fill-controls.py`, 실제 저장은
`stage1914-hancom-paper-authoring/receipt.json`, Print는
`stage1915-compound-fill-print-mcp.json`에 기록했다. fixture는 출력 바이트를 변경하지 않고
복사했으며 글꼴을 포함하거나 새로 공급하지 않는다.

26개 원본은 단선 3폭, 복합선 4종의 3폭, 둥근 모서리 4종, 넓은 삼중선 3폭,
파랑 채우기 이중선 2개, 선 없음·파선이다. 수동 회전 속성과 행렬이 일치하지 않았던
별도 진단 입력은 이 fixture에 포함하지 않았다. 회전 출력 검증으로 주장하지 않는다.

## 검증과 검사

Native producer는 `stage1904-current-native.json`, fresh WASM producer는
`stage1905-fresh-wasm-build.json`이다. 둘 다 `7c082b0a3f` 위의 같은 수정 source를
해시로 확인했다. 공통 구현 파일 해시는
`7bd0dfe52129eff44d5f5239edbce3c547a74b88b8f37a52d5f00a7b10805dfb`이다.

| 출력 경로 | 전체 쪽수 | 페이지별 2px 관용 실루엣 | 직접 증거 |
| --- | --- | --- | --- |
| Native Skia | 1/1 일치 | 100.00000% | `stage1916-compound-fill-native/` |
| fresh WASM | 1/1 일치 | 100.00000% | `stage1919-compound-fill-wasm/` |

전쪽 TSV는 같은 실행의 PNG 두 묶음을 `scripts/visual_sweep.py --silhouette-only
--png-pair <rhwp_png> <pdf_png>`로 재비교한
`stage1920-compound-fixture-tsv/{native,wasm}/silhouette.tsv`에 있다. 새로 렌더한 결과와
기존 PNG의 재계산을 구분한다. 대표 review·standalone overlay는
[단계 기록](../../../mydocs/working/task_m100_7642_stage1900.md)에 연결했다.

띠의 개수·안팎 진행 방향·간격·두께 순서·둥근 모서리·파랑 채우기 보존을 직접 확인했다.
100%는 2px 관용 실루엣 값이며 모든 픽셀의 색상·농도가 동일하다는 뜻은 아니다.
단선의 래스터 농도 차이는 남아 있으므로 완료 페이지의 절대 좌표를 고정하지 않는다.

정식 검사는 `tests/cases/verified_hancom_compound_rectangle_paths.rs`에 새 이름으로
추가한다. 원본 개체 소유·누락·중복, 반대 winding인 구멍, 실제 Print raster의 분리된
띠와 두께 순서, 빈 간격으로 드러나는 파랑 채우기를 관계로 검사한다.
같은 검사 source의 수정 전 3 FAIL/정상 대조군 1 PASS와 수정 후 4 PASS는
`stage1926-compound-regression-proof/`에 있다. 빌드 실패를 결함 재현으로 세지 않았다.

이는 복합 사각형 선의 검증 기록이다. 기존 원본 문서의 미해결 단선·글꼴 차이,
현재 devel 대비 무회귀, 전체 Rust gate의 완료를 대신하지 않는다.
