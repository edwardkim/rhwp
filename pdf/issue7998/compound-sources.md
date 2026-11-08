# P4 복합 경계 그림의 독립 원본과 검증

PDF 연산자를 직접 작성한 원본이다. 엔진 출력이나 추론 결과에서 기대 구조를 만들지 않았다.
별도 그림으로 내려온 dash 경계를 원본 데이터·phase·alpha·clip 그대로 유지하면서 실제 중첩
Table/Cell/Paragraph에 연결한다. 경계 보존용 Picture 조각과 원래 셀 내용 그림은 구분한다.

```bash
python3 pdf/issue7998/generate_composition_sources.py --output /tmp/issue7998-compound --include-compound
python3 pdf/issue7998/generate_compound_variations.py --output /tmp/issue7998-compound
```

첫 명령은 기존 실선/periodic 원본과 dash 7개를 함께 재생성한다.
`--include-compound-hold`는 과거 실행 명령 재현용 alias다. 두 번째 명령은 생성기 옆의
독립 `nested-dash-child.pdf` 원본 연산자로 phase/alpha·clip·곡선 배경/전경 4개를 만든다.
중간 JSON은 로컬 진단이며 저장소에는 원본 PDF·생성기·이 근거만 보존한다.

| 원본 | SHA-256 | 실제 표 수 계약 | Native 최저 % | fresh WASM 최저 % |
|---|---|---:|---:|---:|
| compound-backdrop.pdf | 0880464e4782a2cce30c2385e5dca92020e6800e27cf1a914880732c8042a74d | 2 | 99.98154 | 99.98134 |
| compound-clipped.pdf | 5990ce409b00b4c2bf266fba4d3db3d2b3f497b69ac37f79efb9e81d999b9a5f | 2 | 99.97270 | 99.97221 |
| compound-foreground.pdf | 83644a86fcd9deb1ed50e50673593d983c3184ef3b94afb47681a2049efdc5f7 | 0 | 99.98333 | 99.98285 |
| compound-phase-alpha.pdf | 93495dfde3ae20f16ff430337b2550a475e1b75f75f80c977d9817e22d0bc2ba | 2 | 99.48824 | 99.47968 |
| nested-dash-both.pdf | 479f434c663716daf9185186f856c93359c0202c8f71a4372056a06ec6171710 | 2 | 99.96892 | 99.96833 |
| nested-dash-child.pdf | 41e72d29cbc0892811863a5f341a9f196f1123a5751d5fa14d3d9aa53625bac5 | 2 | 99.97282 | 99.97237 |
| nested-dash-deep.pdf | 09d459e519f6543320e89e8fc62db42fa984094e6e55bebeff123e2c6328d774 | 3 | 99.96603 | 99.96546 |
| nested-dash-first.pdf | d8b832873edb9b72c717dceb95b9c408cab4bbf2315729e594ad77043620d1a4 | 2 | 99.98594 | 99.98572 |
| nested-dash-image.pdf | a3df3184a3b1c6fe1b612c7151e88b0d51cb2a94cf3ab9066ccef8975ca16160 | 2 | 99.97631 | 99.97598 |
| nested-dash-parent.pdf | 42f8f3cfd74b3f0f67fd47eff2383b061ac01e75c86981ed92670be9a8c734ac | 2 | 99.95631 | 99.95554 |
| nested-dash-siblings.pdf | c17761ba615701254438e5adc2e017710850323c93d1406b4766cacdee1333ff | 3 | 100.00000 | 100.00000 |

각 최저값은 같은 원본의 PDF 직접 입력·HWP 저장 재열기·HWPX 저장 재열기 전쪽을 포함한다.
모두 1쪽이며 각 backend에서 무편집 저장본의 PNG는 직접 입력과 동일하다.
자식 셀 삽입·삭제 뒤 양 형식 복원 20쪽의 최저는 Native 99.48796%, WASM 99.47924%다.

독립 셀 계약은 A/B/C/D, 형제 자식의 E/F/G/H, 깊이 2의 자식 소유다.
내용 그림 원본은 첫 셀의 2×2 픽셀 한 개와 B/C/D이며 그림 데이터의 범위를 보존한다.
곡선 전경이 글자보다 뒤에 그려지는 반례는 실제 Table로 강제하지 않고 원래 소유/순서를 유지한다.
원본 PNG IHDR의 전체 영역과 각 Picture의 선택 영역 합집합·비중복,
실제 RenderTree의 셀 내부 포함·문자 보존·유일 소유를 정식 검사에서 확인한다.
일치율을 작성 당시 객체 구조의 정답으로 취급하지 않는다.

검증 producer는 시작 checkpoint `f4842a060` 위 런타임 변경 2개이며, 지문은
`46993bd7c858b203bf24262875fec86c2e7d7517da1139106be6a52663510aeb`다.
Native: `ff29632219b8433ef294bcc9f0a2791cd036c19c2083205f08ebeadfe4e6f8a9`.
fresh Docker WASM: `3f8ba4e23d4a984def562745fa716a69511de1dc58d2fe9cd91078304d9797ed`.
[단계 보고](../../mydocs/working/task_m100_7998_stage4.md#p4-두-번째-절편-착수--compound-경계-그림)에
명령·전쪽 TSV·수정 전 보류/실패·실제 Studio 편집·직접 판정을 연결했다.

2026-10-18 메인테이너가 **시각·중첩 구조 통과**로 판정했다.
이전 positional 출력보다 `nested-dash-parent`가 Native 0.01656%p/WASM 0.01438%p,
`compound-phase-alpha`가 Native 0.48740%p/WASM 0.49549%p 낮다.
반투명 경계의 래스터 농도 차이를 숨기거나 기준값을 갱신하지 않는다.
이는 이번 절편 판정이며 P5의 최신 devel 비교·최종 PR gate 예외를 뜻하지 않는다.
테두리 스타일/행열 편집·독립 병합/지역 격자·긴 inline 그림 host(B14)와 P5는 남는다.
원본 PDF가 독립 시각 기준이며 최종 한컴 열기/인쇄는 P5에서 별도로 확인한다.
