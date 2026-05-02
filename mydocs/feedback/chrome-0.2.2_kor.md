rhwp는 브라우저에서 HWP/HWPX 문서를 바로 열고, 편집하고, 인쇄할 수 있는 무료 오픈소스 확장 프로그램입니다. 별도 프로그램 설치가 필요 없습니다.

주요 기능:

웹에서 HWP/HWPX 파일 다운로드 시 자동으로 뷰어에서 열기
문서 편집: 텍스트 입력/수정, 표 편집, 서식 변경
인쇄: Ctrl+P로 인쇄 미리보기, PDF 저장 또는 프린터 출력
편집한 문서를 HWP 파일로 저장
드래그 & 드롭으로 파일 열기
웹페이지의 HWP 링크를 자동 감지하여 아이콘(배지) 표시
마우스 호버 시 문서 정보 미리보기 카드 표시
우클릭 메뉴: "rhwp로 열기"

⚠️ 중요 안내:

HWPX 직접 저장은 현재 베타 단계로 비활성화되어 있습니다.
HWPX 파일을 열어 편집하더라도 저장이 불가능하며, 다음 업데이트에서
정식 지원될 예정입니다. (HWP 파일 저장은 정상 동작합니다)

중요한 HWPX 문서는 작업 전 반드시 백업해주세요.

개인정보 보호:

모든 처리는 브라우저 내에서 WebAssembly(WASM)로 수행됩니다
파일이 외부 서버로 전송되지 않습니다
광고 없음, 추적 없음, 회원가입 불필요
어떠한 개인정보도 수집하지 않습니다
웹 개발자 지원:

data-hwp-* 프로토콜로 HWP 링크 통합 지원
내장 개발자 도구(rhwpDev)로 디버깅 가능
개발자 가이드 제공
이런 분께 추천합니다:

정부/공공기관 문서를 열람하는 시민
가정통신문을 확인하는 학부모
계약서/보고서를 확인하는 직장인
한컴 오피스가 없는 macOS/Linux 사용자
HWP 파일을 열기 위해 별도 프로그램을 설치하고 싶지 않은 모든 분
MIT 라이선스 — 개인/기업 모두 무료.

[v0.2.2 변경 사항 / Changes — 2026-05-01]

▣ v0.2.2 (2026-05-01) 주요 변경

본 업데이트는 라이브러리 코어를 v0.7.3 → v0.7.9 로 갱신합니다 (4 사이클 누적). 외부 기여자 다수의 조판/렌더링 정정과 메인테이너 회귀 정정 통합.

[조판 / 렌더링 정정]
• 다단 섹션 페이지 누적 공식 정정 — 단단/다단 분기로 trailing
  line_spacing 인플레이션 차단 (외부 기여 by @planet6897, PR #391)
• 다단 우측 단 단행 문단 줄간격 누락 정정 (외부 기여 by @planet6897,
  PR #424)
• 표 페이지 분할 시 큰 rowspan 셀 행 단위 분할 허용 — 한컴 호환
  정합 (외부 기여 by @planet6897, PR #401)
• 같은 문단 안 인라인 그림 페이지네이션 정정 — 두 번째 그림이
  같은 y 좌표에 그려지던 결함 (외부 기여 by @planet6897, PR #406)
• heading-orphan vpos 기반 5 조건 가드 — 헤딩이 후속 표와 함께
  배치되도록 정정 (외부 기여 by @planet6897, PR #408)
• 그림 자동 크롭 (FitToSize+crop) 공식 정정 + 셀 안 그림 클램프
  (외부 기여 by @planet6897 + 메인테이너 회귀 정정)
• 비정상 셀 padding (cell.height 초과) 케이스의 한컴 방어 로직
  모방 가드 추가 — 셀 텍스트가 누락되거나 셀 진입이 안 되던 결함
  정정 (mel-001 인원현황 표 회귀)

[수식 렌더링]
• 인라인 수식 높이를 HWP 권위값 기준으로 설정 + X/Y 스케일링 동시
  적용 (외부 기여 by @oksure, PR #396)
• 수식 ATOP / OVER 의미 분리 + AST 파싱 (외부 기여 by @cskwork,
  PR #397 — 본 저장소 첫 외부 컨트리뷰터)
• 수식 토크나이저 폰트 스타일 키워드 prefix 분리 + 렌더러 italic
  파라미터 적용 (외부 기여 by @planet6897)
• 빈 텍스트 + TAC 수식 셀의 alignment 적용 — 좌측 고정되던
  결함 정정 (외부 기여 by @planet6897)
• HWPX 수식 직렬화 보존 (외부 기여 by @cskwork, PR #400)

[그림 / 도형]
• Picture+Square wrap 호스트 paragraph 텍스트가 그림 영역을
  침범하지 않도록 LINE_SEG cs/sw 적용 (외부 기여 by @planet6897)
• 그룹 내 그림(Picture) 직렬화 구현 — 그룹 그림 포함 HWP 저장 시
  그림 데이터 유실 정정 (외부 기여 by @oksure, PR #428)
• HWP 그림 밝기/대비 효과 SVG 반영 (외부 기여 by @oksure, PR #395)
• wrap=Square 표 paragraph margin x 좌표 반영 (외부 기여 by
  @planet6897)

[각주 / 페이지네이션]
• 각주 multi-paragraph 처리 line_spacing 정합 (외부 기여 by
  @planet6897)
• PartialParagraph 인라인 Shape 페이지 라우팅 정정 (외부 기여 by
  @planet6897)
• TypesetEngine 페이지네이션 fit 누적 drift 정정 (메인테이너 회귀
  정정)

[API · 도구]
• Paragraph 의 char_idx 변환 메서드 외부 노출 — Python/Node binding
  지원 (외부 기여 by @DanMeon, PR #494)
• PageLayerTree 생성 API 도입 — 다양한 렌더러 backend 지원의 공통
  기반 (외부 기여 by @seo-rii, PR #419)
• Canvas visual diff 회귀 검증 인프라 — legacy Canvas ↔ PageLayerTree
  replay 픽셀 diff 자동 검증 (외부 기여 by @seo-rii, PR #498)
• rhwp-studio PWA 지원 — 오프라인 사용 가능 (외부 기여 by
  @dyjung150605, PR #413)
• editor.exportHwp() API 추가 (외부 기여 by @ggoban, PR #411)

[웹 페이지 통합]
• 일부 관공서 사이트의 다운로드 URL 처리 정합 — 뷰어가 직접
  파일을 받지 못하던 케이스 정정

[알려진 한계]
• HWPX 직접 저장은 현재 베타 단계로 비활성화 (HWPX→HWP 완전 변환기
  완성 시까지)
• 인쇄 미리보기 창 크기가 비정상적으로 크면 Ctrl+0 으로 리셋

[기여해주신 분들 — 감사합니다]
@cskwork — 본 저장소 첫 외부 컨트리뷰터 (PR #397, #400)
@DanMeon (PR #405, #494)
@dyjung150605 — 신규 컨트리뷰터 (PR #413)
@ggoban — 신규 컨트리뷰터 (PR #411)
@oksure (PR #395, #396, #427, #428)
@planet6897 (PR #391, #401, #406, #408, #410, #415, #424, #434, #478)
@seo-rii (PR #419, #498)


▣ v0.2.1 (이전)

라이브러리 v0.7.3 — 외부 기여자 다수 + 메인테이너 회귀 정정 통합.
v0.2.0 사이클의 dev-tools-inject.js / content-script.js 버전 동기화
누락 hotfix.


[전체 변경 이력]
https://github.com/edwardkim/rhwp/releases

[소스 코드]
https://github.com/edwardkim/rhwp
