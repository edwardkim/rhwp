# 복합 테두리의 분해 출처와 실제 픽셀 커버리지

parent `1484048eca8a4f6465f78b77285ceec75843f5bc` + C433 구현 patch SHA `b4583d9b2abd5034b1eae5f7d2d81925258838b3f8bdceab961aba0fab210fce`.
Native frozen CLI SHA `14d94e098ca88e48fe6bf02222eb0dac5c2e43cc53f9a8a01099803d9a265469`.
fresh WASM SHA `61cb176d355b1733b0dfc0cb2baf2972e99f051738a6ebf32412c31e8db01ed8`.
Native/Studio를 포함한 runtime2198파일 ledger가 실제 source와 일치함을 확인했다.

| 원본 / 독립 한컴 Print | SHA-256 |
| --- | --- |
| samples/hwp3-sample16-hwp5-2022.hwp | fec3671c9a83cb8388cbdaab38754cb17a8da54d0314ca7e3f56860e871506fe |
| pdf/hwp3-sample16-hwp5-2022-hwp-2020.pdf | 6737b57af568a591eb073eeb27054e10aaf7587cbdfd6f18a06d1033b4025a19 |
| samples/issue5941/1480000-201900698-native-neartop-reset.hwp | 18765ac22270a2cb4ece6a1e6539c2dea6ca6ccde2573b42fb9acce7c34fa83a |
| pdf/issue5941-1480000-201900698-2020.pdf | fe01f1ddffc3153d015adab93178f0fa8a5e9624763ff529362d4410d50955dc |

compat2022, sample16은192DPI, 생물독205쪽은96DPI. canonical PDFium의 raw2px 실루엣과
별도 pdftoppm audit를 파일명으로 구분했다. 양쪽 수치를 혼합하지 않았다.
PNG exporter를 사용했으며 SVG raster를 실제 Native PNG라고 보고한 결과가 아니다.
전체 전후538쌍에서0.1%p 이상 하락0쪽, 최저 delta0.00000%p. PNG byte동일468쌍.
생물독의9/20/21쪽은 양쪽 backend에서 PNG byte가 달라도 raw 값은 모두 유지됐다.

| 범위 | Native before→after | fresh WASM before→after | 미달 |
| --- | --- | --- | --- |
| sample16 전64쪽 최저 | 94.14342→94.59411% | 94.60213→94.60213% | 0 / 0 |
| sample16 63쪽 | 97.65514→99.80596% | 99.80728→99.80728% | 0 / 0 |
| sample16 64쪽 | 98.22800→99.66659% | 99.67064→99.67064% | 0 / 0 |
| 생물독 전205쪽 최저 | 98.06390→98.06390% | 98.12078→98.12078% | 0 / 0 |
| 생물독 회색 실선143쪽 | 100→100% | 100→100% | 0 / 0 |

fixed4a07 Native63/64쪽99.80954/99.67493% 대비 잔여 delta는-0.00358/-0.00834%p다.
0.1%p 기준 이내이며 엄격한 RGB 일치가 아니다. 직접 판독한63/64쪽에는 아래 선의 작은 원점·
글자 농도 차이가 남는다. 40쪽의 본문 줄 간격/자간 잔여도94.59411%에 포함되며100% 해결로
표현하지 않는다. 32/143/205쪽 정상 대조군은 표 소유·후속 내용·그림을 대표 판독했다.

독립 Print63/64쪽의 아래 선은192DPI에서2127/2128행과2130/2131행의 두 띠다.
Native 수정 전은2130행·2133행의 한 행씩으로 줄어 있었다. 구성 획을 다시 픽셀 중심에 옮기는
분기를 건너뛰어 인접 픽셀 coverage가 복원된다. 검사는 원본 문단의 쪽 소유·전체 내용 보존과
분리된 두 띠/각2행 이상 coverage, 독립 중심1063.75/1065.25 CSSpx±2를 함께 보호한다.
63/64쪽 양쪽98% 이상·직접 판독 후 추가했다. 별도 private font 파일은 검사에 공급하지 않는다.

prototype은 실제2PASS/1FAIL→3PASS이며 실패 원인은 각 띠가 한 행인 결함이다.
정식 NativeSkia에서는 sample16세 검사·기존 실선143쪽·본문/종이 테두리 관계 두 검사:
**6PASS/0FAIL**,720skip(선택 필터),test2.072초·전체147.375초,exit0.
prepare 뒤 실제 target016/017/002를 재계산했다. CanvasKit 좌표 단위4PASS, fmt·NativeSkia lib Clippy(-D warnings,38.81초)·Studio TypeScript noEmit도 통과했다.

Root pkg/Studio public 및 새 Chrome155.0.8059.84 초기 JS/WASM 응답이 frozen60과 일치했다.
실제 Studio Print의 정상 한 쪽 이중선 대조군에서 Canvas2D/CanvasKit 모두8개 구성 획 출처를
받았고, software CanvasKit은 readiness PASS·fallback/미지원op/숨은 Canvas2D overlay0,
실제 페이지 화면도 캡처했다. 이는1쪽의 제품 소비 확인이며 actual36 전체 수용을 대신하지 않는다.
Mac WASM은 locked web/pkg/no-opt 로컬 대체 빌드이며 Docker 최적화 빌드가 아니다.

원 로그·producer·PNG 입력 SHA는 ignored output의 `repair58`~`repair65`에 있다.
전쪽 새 실행6건948쪽의 canonical/audit12개 silhouette_manifest는 모두 complete이고
실제 PNG1896쌍의 hash·누락/중복·쪽수도 확인했다. `run_manifest`의 export 초기 incomplete와
silhouette 전용 완료 manifest를 구분하며 원 manifest를 임의로 바꾸지 않았다.
첫 Studio 소비 시도는 inspector response cache 소진으로 실패해 보존했고, 버퍼 용량을
명시한 새 실행으로 초기 응답을 검증했다. 최초 unit 명령의 미설치tsx loader 오류도 보존했으며
현재 Node24의 정식 `node --test` 경로4PASS를 사용했다. 제품/회귀 실패로 집계하지 않았다.

전체 nextest20FAIL의 집중 수정 두 건과 이번 보호 추가는 아직 전체 재실행 결과가 아니다.
다른 필수 미해결·전쪽 검증·actual36이 남아 있으므로 PR 준비 완료가 아니며, PR은 새 승인 후 진행한다.
