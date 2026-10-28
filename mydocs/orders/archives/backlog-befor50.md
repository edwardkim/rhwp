# 백로그 (Backlog) — v0.5.0 이전 수립분 현행화

v0.5.0 이전(2026-02-08)에 수립한 백로그의 현행화 판이다. 당시 9개 분야
40여 항목 중 대부분이 v0.5~v0.7 기간에 구현 완료되어, **잔여 항목만 표로
유지**하고 완료분은 하단에 요약 보존한다.

> 신규 작업은 GitHub Issues 로 채번하므로 이 문서에 새 항목을 추가하지
> 않는다. 잔여 항목도 착수 시 이슈로 등록해 진행한다.

---

## 잔여 백로그 (2026-07-17 실측 기준)

| No | 작업 | 설명 | 현재 상태 |
|----|------|------|-----------|
| B-102 | HWP 내장 폰트 추출 | binData 임베디드 폰트 추출·적용 | 미구현 (export 측 서브셋 임베딩 `--embed-fonts` 는 별개 기능으로 존재) |
| B-204 | OLE 개체 | 외부 문서 임베딩 실행 | **부분** — RawSvg 미리보기 렌더링은 지원 (차트/OLE preview), 원본 앱 연동은 범위 외 성격 |
| B-205 | 글맵시 | 텍스트 효과 렌더링 | 파싱(tag)만 존재, 렌더러 미구현 |
| B-504 | HTML 내보내기 | 스타일 포함 HTML 생성 | 미구현 — `export-markdown`/`export-hml` 로 부분 대체 |
| B-603 | 인쇄 기능 | 브라우저 `window.print()` 연동 | 미구현 — 인쇄 등가 렌더링(print/high-quality profile) 은 export 측에 존재 (#2225/#2297) |
| B-604 | 전체화면 모드 | Fullscreen API 연동 | 미구현 — F11 은 컨트롤 선택 단축키로 전용 중이라 UX 재설계 필요 |
| B-703 | 웹 워커 분리 | 파싱/레이아웃 별도 스레드 | 미구현 (백그라운드 레이지 렌더링 구상 논의만 존재) |
| B-704 | 점진적 로딩 | 대용량 문서 스트리밍 로드 | 미구현 |
| B-803 | 문서 자동 생성 | rustdoc + mdBook 연동 | 미구현 |

---

## 완료분 요약 (실측 확인, 2026-07-17)

### 1. 렌더링 품질
- B-101 이미지 비동기 로딩 (타스크 15) / B-103 다단 레이아웃 (페이지네이션
  엔진, 단 경계 분할 포함) / B-104 머리말·꼬리말 / B-105 각주 (영역 분리·
  번호 연결·각주 모양) / B-106 형광펜 (shade_color 렌더링)

### 2. 컨트롤 지원
- B-201 수식 (equation layout, TAC 인라인) / B-202 차트 (OLE preview RawSvg,
  PNG/SVG/PDF 전 backend) / B-203 그리기 개체 (선·곡선·호·다각형) /
  B-206 필드 코드 (쪽번호·날짜·파일명 등)

### 3. WYSIWYG 편집
- B-301~304 선택·캐럿·입력·삭제 (타스크 17~21) / B-305 복사·붙여넣기 /
  B-306 Undo·Redo (undo 연작 포함) / B-307 서식 변경 (글자모양·문단모양
  대화상자, 서식 도구 모음) / B-308 문서 열기 시 캐럿 자동 배치
  (activateWithCaretPosition) / B-309 커서 이동 체계 (Home/End·단어·문단·
  페이지 단위, shortcut.hwp 단축키 체계)

### 4. 저장
- B-401~403 HWP 직렬화·CFB·압축 (타스크 23) / B-404 새 문서 생성
  (create_blank_document) — HWPX 직렬화·roundtrip 게이트는 백로그 수립
  이후 신규 축으로 별도 진행 (#1315 계열)

### 5. 내보내기
- B-501 PDF (`export-pdf`, svg2pdf vendored) / B-502 이미지 (`export-png`,
  skia) / B-503 텍스트 (`export-text`)

### 6. 브라우저 통합
- B-601 테스트 페이지 → **rhwp-studio 로 대체·초과 달성** (legacy /web 은
  #2313 에서 제거) / B-602 줌·스크롤 UI (상태 표시줄 줌, fitWidth/fitPage) /
  B-605 반응형 (responsive e2e)

### 7. 성능
- B-701 가상 스크롤 (virtualScroll.getVisiblePages) / B-702 렌더 캐싱
  (페이지 트리 캐시, layer JSON 캐시 #2222, 정적 overlay 재사용)

### 8. 인프라
- B-801 E2E (Puppeteer 기반 — Playwright 대신 채택, CDP 매뉴얼) /
  B-802 시각 회귀 (render-diff·Canvas visual diff CI, OVR) /
  B-804 CI/CD (GitHub Actions 다수) / B-805 npm 배포 (npm-publish +
  crates.io)

### 9. 표 편집
- B-901~902 객체·셀 선택 (타스크 25) / B-903 셀 탐색 (Tab 이동 + 마지막 셀
  행 추가) / B-904~905 행·열 추가·삭제, 병합·분할 (타스크 26) /
  B-906 표·셀 속성 편집 (table-cell-props / cell-border-bg 대화상자)

---

*마지막 업데이트: 2026-07-17 (전 항목 코드 실측 대조로 현행화)*
