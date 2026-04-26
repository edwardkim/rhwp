# Changelog

## [0.7.6] - 2026-04-26

라이브러리 버전 동기화. 외부 기여자 다수 + 조판 정밀화 사이클 (#268, #279, #324, #338, #340, #342). 상세는 저장소 루트 CHANGELOG.md 참고.

## [0.7.3] - 2026-04-13

### 수정

- 디버그 오버레이 / SVG 내보내기 표 안 글자 배치 오류 수정 (#136)
  - extension host WASM 직접 렌더링 → webview 위임 방식으로 전환
  - `measureTextWidth` 스텁(`text.length * 8`) 제거, Canvas `measureText()` 실제 호출

### 변경

- 인쇄 커맨드 제거 (WSL 환경 미지원)
- 뷰어 내부 우클릭 메뉴: 기본 메뉴 억제 → SVG로 내보내기 항목 추가

---

## [0.7.2] - 2026-04-13

### 컨텍스트 메뉴 추가 (#132)

탐색기/에디터 탭 우클릭 메뉴에 HWP 전용 커맨드 4개 추가:

- **HWP: 인쇄** — 뷰어 webview에서 `window.print()` 호출
- **HWP: SVG로 내보내기** — 페이지별 SVG 파일 저장 + 진행률 표시
- **HWP: 디버그 오버레이 보기** — 문단/표 경계 시각화 HTML을 VS Code 탭에서 확인 (개발자용)
- **HWP: 문단 덤프** — QuickPick으로 섹션/문단 선택 → ParaShape·LINE_SEG를 Output 채널에 출력 (개발자용)

---

## [0.7.1] - 2026-04-11

### 양식 컨트롤 지원

- HWPX 양식 컨트롤 파싱 구현 — checkBtn/btn/radioBtn/comboBox/edit (#110)
- 양식 컨트롤 셀 커서 진입 가능 (#111)
- 체크박스 클릭 토글 인터랙션 (#112)

### 보안 수정

- CodeQL XSS 경고 제거 — img.src URL 파싱 강화 (#128)

---

## [0.7.0] - 2026-04-11

### 조판 개선

- 표 캡션 current_height 보정 — LAYOUT_OVERFLOW 43→4건 수정 (#101)
- PartialParagraph 페이지 경계 클리핑 방지 — 렌더링 단계 줄 y 클램핑 (#102)

### 브라우저 확장 / 썸네일

- HWP/HWPX 썸네일 자동 추출 CLI + Chrome 확장 연동 (#86)
- Safari 확장 재작성 + 보안 수정 macOS 완성 (#83, #84, #88)

---

## [0.6.0] - 2026-04-07

### 조판 개선

- TAC 표 trailing ls 경계 조건 순환 오류 해결 (#40)
- 같은 문단 TAC+블록 표 y_offset 역행 수정 (#41)
- 머리말/꼬리말 내 Picture 렌더링 + 꼬리말 인라인 배치 (#42)
- 그림 자르기(crop) 렌더링 + 이미지 테두리선 (#43)
- 분할 표 셀 세로 정렬 — 중첩 표 높이 반영 + 분할 행 Top 강제 (#44)
- 누름틀 안내문 높이 제외 + TAC 표 fix_overlay 이중 적용 수정 (#61)
- 같은 문단 [선][선][표][표] 레이아웃 + 글앞으로/글뒤로 Shape vpos (#62)

### 폰트

- 오픈소스 폰트 폴백 전략 1~3단계 도입 — Pretendard, Noto Sans/Serif KR (#67)
- SVG export 폰트 서브셋 임베딩 (#68)
- 오픈소스 대체 폰트 메트릭 추가 (#69)

### 코드 품질

- Clippy 경고 0건 + CI 엄격 모드 전환 (#47)
- 저작권 폰트 완전 제거 + THIRD_PARTY_LICENSES.md 작성
- README 상표권 면책 조항(Trademark disclaimer) 추가

## [0.5.4] - 2026-04-03

### 수정

- Bold↔Normal 폰트 전환 시 글자 겹침 (#38)
  - Bold 폴백 폭 보정 제거, Justify 공백 최소 폭 보장
  - 반각 구두점(''""…·‧) 폭 수정 + Canvas scale 렌더링
  - 전각 통화 기호(₩€£¥) 폴백 처리
- 통화 기호(₩) 렌더링 — 폰트 폴백 (#39)
  - Canvas: 맑은고딕 폴백 폰트 전환
  - SVG: 폰트 체인에 시스템 한글 폰트 추가
- 다중 TAC 표 페이지네이션 (#35)
  - 캡션 이중 계산 제거, common.height 클램프
  - trailing line_spacing 제거, 중간 표 gap 이중 적용 제거
- 인라인 TAC 표 텍스트 reflow (#34)
  - LINE_SEG text_start 기반 줄 나눔

## [0.5.3] - 2026-04-03

### 수정

- 머리말/꼬리말 표 셀 안 이미지 미렌더링 (#36)
- 특수문자 포함 폰트 이름으로 문서 로드 실패 (#37)

## [0.5.2] - 2026-04-03

### 수정

- 문단 삽입/삭제 후 페이지 수 과도 증가 (#30)
  - measure_section 캐시 인덱스 조정, 증분 측정 최적화
- 인라인 TAC 표 텍스트 흐름 배치 (#31)
  - 표 하단 = 베이스라인 + outer_margin_bottom 세로 정렬
- 인라인 TAC 표 텍스트 reflow 개행 시점 (#34)
  - LINE_SEG text_start 기반 줄 나눔
- 다중 TAC 표 페이지네이션 간격 과대 (#35)
  - 캡션 이중 계산 제거, common.height 클램프, trailing ls 제거
- TAC 표 pre-flush/fit 체크 0.5px 톨러런스 적용

### 추가

- createTableEx API — 인라인 TAC 표 생성 (#32)
- 논리적 오프셋 체계 (insertTextLogical, getLogicalLength)
- getPageRenderTree API — 렌더 트리 JSON 직렬화
- E2E 조판 자동 검증 체계 (scenario-runner) (#33)

## [0.5.1] - 2026-04-02

### 수정

- 확대 시 캔버스 왼쪽 스크롤 불가 (#29)
  - scroll-container에 overflow:auto + scroll-content 래퍼 도입

## [0.5.0] - 2026-04-02

### 추가

- HWPX TabDef 파싱: hp:switch 구조, 2× 스케일, fill_type 매핑 (#13)
- 탭 리더 채울 모양 12종 SVG/Canvas 렌더링 (#13)
- 밑줄/취소선 13종 렌더링: 물결선, 이중물결선 포함 (#16)
- HWPX 글상자(사각형) 파싱: curSz/fillBrush/lineShape (#15)
- IR 비교 도구: `rhwp ir-diff` CLI 명령 (#18)
- CSS 디자인 토큰: 색상/타이포그래피/간격 30개 변수 (#22)
- 모바일 반응형 레이아웃: 태블릿/모바일/터치/인쇄 대응 (#22)
- iOS IME 한글 조합: contentEditable div + afterEdit 디바운스
- 글상자 내부 표/그림 렌더링 (#24, #25)

### 수정

- 페이지네이션 부동소수점 누적 오차 0.5px 톨러런스 (#14)
- HWPX 탭 문자 UTF-16 8 code unit 매핑 (#17)
- ParaShape lineSpacing Fixed/SpaceOnly/Minimum 2× 스케일 (#18)
- shape 배경 pattern_type 판정: >=0 → >0 (#16)
- 글상자 기본 treat_as_char=true (#2)
- iOS Canvas 최대 크기 64MP 제한 DPR 자동 조절
- 폭 맞춤/쪽 맞춤 줌: pageInfo 이중 변환 제거
- 글상자 내부 표 너비 비례 축소 (#24)
- 개체묶음 자식 도형 크기 regression 수정 (#27)
- 바탕쪽 탭 리더 렌더링 억제 — 고스트 라인 제거 (#28)
- Canvas 이중선 종류 반전 + 간격 과도 수정 (#28)

## [0.4.0] - 2026-03-31

### 수정

- Fixed 줄간격 TAC 표의 pagination overflow 수정 (#9)
- 비-TAC 어울림 그림의 pagination 높이 반영 (#10)
- TAC 표 line_end 보정에서 ls 이중 추가 제거 (#10)
- HWPX 하이퍼링크 필드의 char_shape 매핑 수정 (#11)
- HWPX 표 속성 UI 바인딩: table.common 필드 직접 사용
- 문단 간격 UI 바인딩: 원본 HWPUNIT 값 사용 (/2.0 제거)
- TAC 표 혼합 문단의 pagination 높이 이중 계산 수정 (#19)
- 강제 줄넘김(Shift+Enter) 후 TAC 표의 ComposedLine 분리 (#20)
- composer: LINE_SEG lh에 표 높이가 포함된 텍스트 줄을 th로 보정

## [0.3.0] - 2026-03-30

### 수정

- HWPX switch/case 네임스페이스 분기 처리 (문단 간격/줄간격 정확도 개선)
- 고정값 줄간격에서 TAC 표와 문단의 병행 배치 지원

## [0.2.0] - 2026-03-30

### 수정

- 셀 내 TAC 이미지가 수평으로 나열되던 문제 수정 (LINE_SEG 기반 수직 배치)
- 비-TAC 그림(어울림 배치) 높이가 후속 요소에 미반영되던 문제 수정

### 추가

- cellzoneList 셀 영역 배경 지원 (이미지/단색/그라데이션, HWP+HWPX)
- imgBrush mode="TOTAL" 파싱 지원

## [0.1.0] - 2026-03-29

### 추가

- HWP/HWPX 파일 읽기 전용 뷰어 (CustomReadonlyEditorProvider)
- Canvas 2D 기반 문서 렌더링 (WASM)
- 가상 스크롤 (on-demand 페이지 렌더링/해제)
- Ctrl+마우스 휠 줌 (0.25x ~ 3.0x, 커서 앵커 기준)
- 상태 표시줄 UI (페이지 네비게이션 + 줌 컨트롤)
- 문서 내 이미지 지연 재렌더링
