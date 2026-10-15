# PR #7673 메인터너 보정 — 새 물리 쪽 단 배치와 구분선 끝

## 독립 근거

원 head `aeb6399f42b9cb8029d3e3cf640a5a66c5447998`는 같은 단 수/종류라도 새 쪽의 방향·선·너비를
적용하도록 진입 조건을 확장한다. 실제 두 쪽 입력의 Print와 비교하면 zone 정의만 바뀌고
`PageContent.layout` 소비가 오래된 정의를 유지하는 경우가 남았다.
맞쪽 단은 새 물리 쪽의 홀짝을 사용해야 하며 이전 쪽 parity로 계산하면 안 된다.

HWPX8000/768/24000 너비 정의의 합을 본문 폭에 맞춰 강제 보정한 가설은 폐기했다.
독립 한컴의 HWP→HWPX 재저장본도 같은 비례 값을 보존하는 것이 실제 관측이다.
따라서 parser가 `ColumnDef.proportional_widths` 계약을 전달하도록 보정했다.
수동 XML 입력과 한컴 정상 재저장 HWP/HWPX를 모두 검증했다.

## 실제 호출 경로

`typeset/section/entry.rs`의 쪽 나눔 뒤 단 정의 선택
→ 새 물리 쪽 parity로 레이아웃 생성
→ `typeset/state/commands.rs::install_page_start_layout`
→ 조판 state와 `PageContent.layout`에 동일 레이아웃 게시
→ 최종 단 원점·내용·구분선 배치가 새 결과를 소비한다.

`layout.rs`의 구분선 끝은 흐름 누적 높이와 실제 수용 자식의 물리 끝을 구분한다.
기존 후행 줄간격은 다음 내용의 흐름 예산에 유지하고 `prev_zone_sep_y_end`는 실제 자식 끝으로
갱신한다. 선을 특정 문서 좌표로 clamp하거나 자식 출력을 숨기지 않는다.

이번 변경은 쪽 시작 단 정의·구분선이며 표 rowspan/continuation의 컷·소유 유닛·예약 예산을
바꾸지 않는다. 그 분할 경로는 비해당이다. 단 수 변경뿐 아니라 방향만·선만·너비만 바뀌는 실제
진입 경계를 실행했다.

## 실행 증거

첫 독립 px 실행15건 중14PASS/1FAIL: 구분선 아래끝 Print145.6013px, 출력153.6px였다.
수정 뒤15/15PASS, 최종 문서별 px29/29PASS. 이미 통과 중이던 단 내용 순서·문단 소속·반대 방향
관계 검사도 유지했다. 고정 좌표는 독립 PDF의 문자 원점·벡터 선에서96dpi로 환산했다.

해당6문서 전12쪽은 Native/fresh WASM 모두100%이며 직접 확대 판독을 마쳤다.
[공통 시각 증거](../assets/semanticist21_20261015/README.md)에 원본/PDF SHA·base/head 전쪽 값·PNG를
보존한다. 실제 Studio Canvas2D·CanvasKit·SVG에서도 두 쪽과 선 끝을 대조했다.

통합 code 후보는 `20d93fe1d4226acab640b66875964d09c7d6d59b`이다.
전체 게이트와 원격 상태는 [개별 review](pr_7673_review.md)에 연결한다.
