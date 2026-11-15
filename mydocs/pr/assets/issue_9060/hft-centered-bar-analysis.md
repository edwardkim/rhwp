# 원본 HFT 막대 칸과 대체 glyph의 중심

전진폭을 원본대로 1em으로 고쳐 16쪽 본문 정렬을 복구한 뒤에도, 확대 overlay에는
짧은 대체 dash가 왼쪽에 남았다. 독립 Print T6/HFT376의 칸은 1000/1000em이며,
막대는 x345~666에 놓인다. 반면 실제 fallback인 Haansoft Batang의 자연 전진은
반각보다 좁다. 이 자연 glyph를 원점에 그대로 두면 뒤 본문만 올바르고 막대는 어긋난다.

원본·독립 PDF와 인쇄 연산자의 출처는 [전진폭 분석](hft-en-dash-analysis.md)을 따른다.
관측한 glyph 경계는 독립 근거이며 그 좌표·문서 ID·쪽 번호를 제품에 복사하지 않는다.

공통 glyph-fit에서 원본 HFT의 확인된 중심 막대 칸과 실제 표시 glyph의 자연 전진을
구분한다. 원 칸은 측정에 사용하는 같은 `METRICS` 항목의 `get_width`와 `em_size`,
실제 그리기 크기·장평에서 구한다. 별도의 전진 상수를 추가하지 않는다. 좁은
표시 glyph를 늘리지 않고 양쪽 남는 칸을 똑같이 배분한다. 원 프로그램의 1em glyph가
공급되면 이동은 0이다. TrueType 원본·다른 원본 프로그램·확인되지 않은 기호에는
이 계약을 적용하지 않는다. 자간·줄맞춤은 뒤 원점만 옮기고 막대 자체의 중심을 바꾸지 않는다.

Native와 Canvas는 같은 `canvas_cluster_fit`의 결과를 기존 char 위치 뒤에 소비한다.
기존 다른 ASCII·음수 자간·잉크 보존 분기는 옮기기만 하고 유지한다. 공통 fit 구현을
기존 5천 줄 이상의 renderer/mod.rs에서 300줄 이하의 glyph_fit.rs로 이동하고 root의
동일 private API로 재노출한다. 큰 진입점에는 새 판정을 더하지 않는다.

회귀는 MIT 합성 CFF의 실제 hmtx 900/1000과 1em 원본 칸 사이의 중심 관계를 최종
Native raster에서 비교한다. 표시 glyph의 기준 출력은 같은 공개 프로그램을 독립적인
중심 위치에 놓아 얻는다. 설치 글꼴·비공개 글꼴·실물 문서의 절대 px·PNG golden은
요구하지 않는다. 보정 전 실제 최소 글꼴 반례가 실패한 뒤 제품 보정을 검증한다.
보정 전 실제 최소 글꼴 검사 한 건이 raster 112바이트 차이로 실패했다. 미확인 원본을
그대로 두는 대조와 `original_glyph_metric_family` 경로도 정식 회귀에서 확인한다.
현재 Native 16쪽의 99.96%나 90% 통과를 남은 glyph 결함의 면제로 사용하지 않는다.

첫 정식 suite 빌드에서는 새 검사에 필요한 `symbol_font_program` 소비자 등록 누락으로
`E0432`가 발생했다. 기존 suite 생성기의 공용 helper 소비자 목록에 새 검사만 추가하고,
함께 또는 따로 배정할 때 helper를 정확히 한 번 공급하는 기존 Node 검사에도 포함했다.
생성된 harness를 직접 수정하거나 새 개별 CI 호출을 추가하지 않는다. 관련 Node 26건은
실제 실행해 통과했으며, 제품 코드와 Rust assertion을 바꾸지 않고 정식 suite를 다시 빌드한다.

보호 기준은 수용한 최종 전쪽 Native/fresh WASM 결과에서 하락·누락이 없는지 확인한 뒤
상승한 공식 표시값으로 함께 올린다. 선택 페이지의 진단 점수는 새 보호 기준으로 쓰지 않는다.

Native 단계에서 새 공개 CFF 검사 2건과 관련 집중 검사 총 93건, Native lib·Clippy를 통과했다.
원 PR 문서의 Native 14쪽은 모두 100.00%이며, 중첩 표 17쪽은 하락 없이 16쪽이 99.99%로
올랐다. 실제 review·overlay 확대와 남은 검증 범위는 [단계 기록](hft-centered-bar-stage/README.md)에 있다.
