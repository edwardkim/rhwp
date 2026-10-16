# #7445 인쇄 비트맵 축소 보정 C85

Issue: #7445

최신 devel `fe5725dc38` 위 parent `1c8ef6cb60`에서 축소 비트맵의 얇은 선 손실을 보정했다.
원인·소비 경로·전쪽 TSV·직접 판독·실패 잔여는 [단계 증거](../pr/assets/issue7445/completed-20261009/regression-policy-20261015/bitmap-print-20261017/README.md)를 따른다.

## 현재 단계

- Native48/fresh WASM48조건 각각1745쪽 비교 완료. 최초 Chrome 종료1건은 동일 빌드 재실행으로 정상 완료했다.
- 로드맵302쪽은 양쪽 최저91.00093%, 생물독소205쪽은97.12915/97.13157%다.
- source/layout 좌표 → 최종 crop/fill paint → 공통 준비 비트맵/장치 paint 경계 → Native·WebCanvas·실제 CanvasKit의1:1 image draw를 추적했다.
- actual 그림 소유 영역의 Native/CanvasKit240370픽셀 차이0을 확인했다. 전체 문서/Actual36 통과는 아니다.
- source-side unit 추가는 기존 정책이 금지하므로 임시 모듈을 제거하고 기존 Native Skia CI의 차트 PNG 통합 검사에 등록했다. 새 정책·상한 변경은 없다.
- sample16 WMF18/35쪽의 Native/WASM raw 미달을 보존했다. 장치 패턴 좌표 진단은 점수가 낮아져 채택하지 않았으며 Windows GDI와 독립 Print PDF의 JPEG/마스크까지 대조 중이다.
- PR 생성은 미달 해소와 남은 전체 필수 검증 후 진행한다. 사용자 오늘할일·PR7871 초안·원본/PDF/공유 target은 보존했다.

## 정식 실행

- 정식 Native Skia `issue_2293_chart_png_text`: 2 PASS/0 FAIL/232 filtered,0.646초. 컴파일1분49초.
- 동일 원본 회귀 음성/양성 대조: 이전 renderer 실제 FAIL(exit101,1.242초), 새 renderer PASS(exit0,1.170초).
- suite manifest PASS:1525 sources/6604 attrs/48 targets. unit-tier PASS:4103 tests/296 modules, 기준선 상향0.
- TypeScript와 fresh Native/WASM 빌드·96조건 전쪽 비교는 위 단계 producer에 고정했다. 현재 전체 nextest·Rust lint/전체 Skia 등 최종 게이트는 아직 실행 전이며 통과로 쓰지 않는다.
