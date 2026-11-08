# 저장 각주 앞 빈 줄의 물리 점유 — C444 중간 검증

Issue: #7445. 부모 `bd361ceac032830ab251d95ca8c088bcaf719686`에 runtime4파일을 적용한
frozen247 Native/250 fresh WASM 결과다. 코드/빌드 원장은 ignored
`rebase-4a07d3ae-20261019/repair247-resolved-empty-tail-native/`와
`repair250-resolved-empty-tail-fresh-wasm/`, 실행은 `repair251-roadmap-resolved-tail-whole/`다.
전체 PR 완료 또는 로드맵 시각 통과가 아니다.

- 입력 `samples/issue5941/1490000-201600081_roadmap_research.hwp` SHA256
  `a06f46ec3f175c7cfa84eb3178b8b3fbdf78e94f71b31d7d87f3417a2617dae9`.
- 독립 한컴2020 Print `pdf/issue7382-regression-review/1490000-201600081_roadmap_research-2020.pdf`
  SHA256 `133f79cad7b756ebef777d253befb14f9407faaf7fe04eeea66036f3e049bbe7`.
- compat2022/Print/96DPI, 현재 canonical PDFium, raw2px/90% 문턱.
- 302/302쪽 양쪽 전쪽을 새로 산출했다. Native 최저83.61607%, WASM83.58341%,
  각각 미달은153쪽 한 곳이다. 나머지301쪽의 정량 통과로 미달을 상쇄하지 않는다.
- 108/129쪽 review와 overlay를 양쪽 직접 판독했다. 본문/표/각주와 쪽번호가 같은 쪽에
  남고 빈 줄이 각주 영역 위에 실제 점유한다. 글자 AA와 미세한 색 테두리는 남는다.
-153쪽도 양쪽 직접 판독했다. 그림/제목/쪽번호 위치는 가까우나 옅은 격자선의 명암과
  재표본화 차이가 남는다. 새 px 핀/90% 예외/기준 PDF 변경은 하지 않았다.
- 기존22검사: frozen217 18PASS/4FAIL → frozen247 22PASS/0FAIL.
  전8657개 본문/셀/각주 소속과 텍스트 순서 보존. 정식 전체 회귀는 별도 미완료다.
- WASM은 Mac locked `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker
  최적화 결과가 아니다. root pkg/Studio public/실제7736 초기화 JS·WASM 응답 hash 일치.

TSV 두 개는251의 전쪽 원본을 바이트 그대로 보존했다. 그림은 페이지별 review/overlay 쌍이다.
로그·중간 JSON·원본 비트맵·글꼴 바이너리는 이 폴더에 포함하지 않는다.
