# 계산 빈 호스트 표의 바깥 상자와 뒤 문단

제품 소스 `bc1e99abf036ef6febc551437962ef20fb726939`, 고정 정책 base
`9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`다.
base 생산 소스는 `60972534680a0c5e51e54ec189b92e252e81a867`이며 runtime·Cargo·font asset·
WASM wrapper 차이가 없음을 별도로 확인했다. 생산 SHA를 정책 SHA로 바꾸어 적지 않는다.

같은 공개 API의 빈 앵커·2×2 표 뒤 빈 줄 또는 본문, 바깥 여백 0/5,000HU의 네 입력이다.
한컴 2020 11.0.0.9136 PrintMethod0·1-up PDF를 새로 출력했고 전처리는 none이다.
원 helper와 생성 단계·job·입력/PDF SHA는 [원장](evidence.json)에 기록했다.
[독립 좌표](expected.json)는 PDF vector 선 중심과 마지막 본문 글줄의 실제 font bbox를
72pt에서 CSS96dpi로 환산했다. 픽셀 값을 엔진 출력에서 역으로 정하지 않았다.

네 입력은 Native210/fresh057과 고정 devel Native194/fresh052에서 각 한 쪽이며 모두 raw100%다.
여덟 base/후보 대응은 페이지별 하락 0%p다. 원 통합 후보 Native209/fresh056의 5,000HU 입력은
79~86%였으므로 이 보정은 devel의 정상 동작을 복원한다. 기존 관계 검사 기대값은 유지했다.

정상 대조군인 기존 계산 빈 호스트 14입력과 강조점 한 입력도 후보 양쪽 모두 raw100%다.
따라서 후보 총 38쪽, base 8쪽의 전체 TSV 원문 46개를 `candidate/`와 `base/`에 보존했다.
기존 14입력과 강조점의 원본·PDF는 이미 추적 중인 동일 바이트를 재사용한다.
19조건 통과를 원래 45개 PR 또는 전체 최종 검증 완료로 확장하지 않는다.

| 후보 경로 | review | standalone overlay |
| --- | --- | --- |
| Native | [review](candidate/native/review_001.png) | [overlay](candidate/native/overlay_001.png) |
| fresh WASM | [review](candidate/wasm/review_001.png) | [overlay](candidate/wasm/overlay_001.png) |

같은 페이지·배율로 직접 판독해 표의 원점 유지, 위로 역전됐던 두 문단의 아래 여백 뒤 배치,
본문 내용·순서 보존을 확인했다. 얇은 괘선의 raster 농도 차이는 남는다. PDF의 선 폭은
0.36pt(96dpi0.48px)이고 일부 외곽선을 중복 그린다. raw100%와 RGB 완전 일치는 구분한다.

새 이름의 `computed_table_outer_box_successor_contract`는 네 원본의 전체 쪽수, 표의 원래
문단 소유·행열, 뒤 빈 줄·본문의 소속·순서·누락/중복·바깥 아래 여백 비침범을 보호한다.
완료100%쪽의 표 네 변과 뒤 본문 위치는 독립 Print의 ±2px도 함께 검사하며,
공개 CLI JSON의 0.05px 반올림 오차를 그 허용폭 안에서 소비한다.
기존 검사를 지우거나 허용치를 늘려 통과시키지 않았다.
동일한 새 Rust 검사는 Native209 CLI에서 바깥 아래 여백을 침범한 뒤 문단으로 FAIL(3354),
Native210 CLI에서는 PASS(3355)였다. 외부 harness의 지원 library는 Native210이며, 실제
수정 전 실행 CLI가 Native209임을 구분했다. 관련 기존 대조군까지 22 PASS/0 FAIL이었다.
전체 Rust·lint·Studio와 원래 승인 범위의 최종 gate는 별도로 남아 있다.

고정한 네 쪽은 향후에도 양쪽 raw98% 이상을 유지해야 한다. 한쪽이라도98% 미만이면
회귀이며100% 개선은 허용한다. 페이지·DPI·좌표계·SHA·원본 TSV·PNG와 새 검사 소스는
`evidence.json`의 `px_regression_contract`에 연결했다.
