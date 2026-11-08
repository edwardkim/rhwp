# #7998 병합·나누기의 소유 보존과 행 격자 왕복

`generate_edits.rs`는 기존 공개 PDF를 정상 DocumentCore API로 연 뒤 병합·나누기·
행 삽입·두 저장 형식을 생성한다. `edited/` 16개와 `preserved/` 4개를 별도 폴더에서
바이트까지 재현했다. 수동 저장 LineSeg나 가짜 셀은 추가하지 않는다.
`provenance.json`은 입력·독립 Print의 SHA-256, 실제 인쇄 원본 형식, 전체 쪽별 결과와
제품 소스/런타임의 해시를 연결한다. 실행 로그·중간 JSON은 ignored 증거에 둔다.

일반 표는 병합한 두 문단을 순서대로 배치한다. HWPX의 셀 시작이 없는 중복 논리 행은
셀 시작/끝·영역 경계를 보존한 표준 격자로 투영한다. 셀 내용·명시적 크기와 live IR은
변경하지 않는다. `META-INF/rhwp-table-row-grids.json`은 rhwp의 잠재 나누기 격자를
왕복시킨다. 이를 제거한 `rowspan-merge-standard.hwpx`도 정상 표준 표이며 한컴은
이 격자를 편집한다. 한컴 재저장으로 private 정보가 없어지면 원래 잠재 격자까지
보존한다고 주장하지 않는다. 물리 빈 영역·중복 소유·의미 있는 영역 경계를 지워야만
기록할 수 있는 경우에는 HWPX 내보내기 전체를 명시적으로 거절한다.

원본 경계를 retained paint로 보존하는 표의 병합·나누기는 아직 지원하지 않는다.
명령은 문단 이동·재조판·이벤트·저장 변경 전에 거절하며, 단순 내용 그림이나 다른 표/
자식 표의 경계 소유를 대상 표의 제약으로 오인하지 않는다. 일반 표와 지원 중인 행열
편집은 정상 대조군이다. 이 절편을 retained paint 재배분 전체의 완료로 해석하지 않는다.

## 재현

저장소 루트에서 `native-skia`·기본 `pdf-import`를 포함한 lib를 공용 review target에
빌드한 뒤 아래 생성기를 컴파일한다. 같은 제품 빌드가 있으면 그대로 사용한다.
`RHWP_ZIP_RLIB`와 `RHWP_SKIA_DIR`는 그 빌드가 사용한 zip rlib와 Skia 출력 경로다.

```bash
RHWP_REVIEW_TARGET=/home/edward/mygithub/rhwp/target/pr-review
rustc --edition=2021 pdf/issue7998/cell-regions/topology/generate_edits.rs \
  -L "dependency=$RHWP_REVIEW_TARGET/release-test/deps" \
  --extern "rhwp=$RHWP_REVIEW_TARGET/release-test/deps/librhwp.rlib" \
  --extern "zip=$RHWP_ZIP_RLIB" -L "native=$RHWP_SKIA_DIR" \
  -o /tmp/rhwp-7998-generate-topology
/tmp/rhwp-7998-generate-topology /tmp/rhwp-7998-topology-new
```

`rowspan-merge-from-hwp.hwpx`는 CLI `export-hwpx`와 같은 `from_bytes`·내보내기 API를
사용한다. 출력 폴더가 이미 있으면 생성기는 중단하므로 기존 증거를 덮어쓰지 않는다.
생성 후 파일별 SHA-256을 `provenance.json`의 `generatedInputs`와 대조한다.

## 독립 기준과 검증 범위

각 backend 24입력·24쪽의 최저 raw 실루엣은 Native **99.93299%**, fresh Docker WASM
**98.27024%**다. 각 입력의 전체 쪽수는 모두 1 / 1이며 누락은 없다. 2026-10-19
메인테이너가 실제 Studio에서 **시각·편집 구조 통과**로 판정했다.
점수와 별도로 문단 순서/비겹침·셀 소속·자식 표·저장 후 재편집을 확인했다.
기존 가는 선 농도와 Helvetica 대체 글꼴 차이를 없어졌다고 보고하지 않는다.

`print/` 12개 PDF 중 11개는 원본 399×360pt 용지의 한컴 2020 자동 Print다
(11.0.0.9136, PrintMethod 0, `hancom2020_pdf_driver_one_up`). 각 PDF의 실제 인쇄
입력은 manifest에 명시한다. 같은 편집의 HWPX Print를 HWP와 함께 대조한 경우를
별도 HWP 직접 인쇄로 세지 않는다. 나머지 `rowspan-merge-hwp-manual-2020.pdf`는
메인테이너가 **HWP**에서 인쇄한 정상 출력이다(11.0.0.9166, A4).
원본 용지와 다른 여백은 있으나 내용 원점/크기는 같고, PDF나 이미지를 사후 정합·
확대하지 않았다. 추가 MCP 출력과의 차이는 아래 승인된 예외다.

[추가 호환성 차이](compatibility/README.md)는 정상 사용자 PDF 기준으로 수용하고
백로그에 남기도록 별도 승인받았다. 진단 PDF를 정상 출력으로 숨기거나 정상 기준에
섞지 않는다. 상세 전후 결과·전체 TSV·실제 명령은
[stage27](../../../../mydocs/working/task_m100_7998_stage27.md)에 연결한다.
R1의 일반화, R2 수평 메트릭, P5 최신 devel 통합과 전체 검증은 남아 있다.
