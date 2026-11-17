# PR #9152 리뷰 — 줄 밖 탭 정지의 줄 소속

## 최종 판정

**로컬 검토 충족 — 최신 통합 PR CI·후속 기록 확인 후 병합 가능.** 원 기여와 필요한 메인터너 보정을 구분해 검증했다.

## 변경과 검토 범위

작성자는 `hyunmyungJaneLee`, 원 head는 `acdb778b35dd9efe30ad63a8ba9d3f1ef6b88e4b`다.
관련 이슈는 [#9150](https://github.com/edwardkim/rhwp/issues/9150)다.
원 기여가 `가` / 탭 / `X`의 세 줄 소속을 복원했다. 탭만 있는 문단과 탭 없는
문단은 한 줄이며 워드 호환 문서에는 한/글 전용 규칙을 적용하지 않는다.
새 독립 PDF에서 `가`→`X`는 약 29.248px(96DPI), 시작 x는 같음을 확인했다.

## 공식 PDF와 실제 시각 결과

원 PR의 **웹한글기안기 PDF는 공식 비교 자료에서 제외**했다. 같은 원문을 MCP
`--engine 2024`로 새로 출력했다. 실제 제품은 데스크톱 한컴 2024 `13.0.0.3901`,
**Hancom PDF Print**이며 Microsoft Print to PDF·PDF 저장/내보내기가 아니다.
입력 전처리는 `none`, Print method는 `0`, 출력 모드는
`frame_print_to_pdf_ex_one_up`다. 입력의 저장 버전과 실제 출력 버전을 구분했다.
[공식 PDF](../../../pdf/pr-review-open-prs-20261117/tab_stop_beyond_line-2024.pdf)와
[MCP·해시 기록](../assets/open_prs_20261117/reference-pdfs.json)을 보존했다.

같은 원문 1쪽·PDF·96DPI·compat2024·2px raw 기준 Native 공식 표시값은
base **55.75%** → 원 기여 **100.00%** → 보정 후보 **100.00%**다.
쪽수 불일치는 없으며 review와 overlay를 직접 열어 탭 앞뒤 글의 줄 소속·같은 시작점·후속 문단 순서을 판독했다.
이는 허용 반경의 실루엣 보조값이며 RGB 픽셀 완전 동일성을 뜻하지 않는다.

- [base TSV](../assets/open_prs_20261117/cases/tab/native/base/silhouette.tsv) · [후보 TSV](../assets/open_prs_20261117/cases/tab/native/candidate/silhouette.tsv)
- [후보 review](../assets/open_prs_20261117/cases/tab/native/candidate/review-001.png) · [후보 overlay](../assets/open_prs_20261117/cases/tab/native/candidate/overlay-001.png)
- [Native 원문 PNG](../assets/open_prs_20261117/cases/tab/native/candidate/raw-001.png) · [한컴 PDF PNG](../assets/open_prs_20261117/cases/tab/native/candidate/pdf-001.png)

## 접수와 통합 신원

리뷰어는 `jangster77`이며 지정 결과를 API에서 확인했다. 검토 브랜치는
`review/integration-20261117-open-prs`, base는 `1e2488ebd432deb7c326be532f8a91945f260231`,
현재 보정 source는 `2c5e3fe38726d23bec1c970ba3f92a49e15cae34`다. #9144 → #9149 → #9152의 11개 커밋을
원 SHA를 기록하며 체리픽했고 개별·쌍별·누적 시뮬레이션 9건은 충돌 없이 종료했다.
원 fork에는 push하지 않았다. [선행 분석](../assets/open_prs_20261117/intake-analysis.md)을 따른다.

## 최종 로컬 검증과 후속 조건

[통합 검증 정본](../assets/open_prs_20261117/README.md)과 각 원본 TSV·PNG를 확인했다.
최소 글꼴 환경 focused 7건과 전체 nextest **13,708 PASS / 0 FAIL / 45 skip**,
fmt·Native/WASM/all-target Clippy·workspace build·manifest/unit-tier, Native Skia **2·2·36건**을 통과했다.
원본 글꼴 일반 환경의 정본 보호 18종·1,771쪽은 Native/fresh WASM 모두 하락·상승 0쪽이다.
명시적 편람 Native 383쪽(최저 99.97%)과 기여자 공통 보호 8건도 통과했다.
[회귀 집계](../assets/open_prs_20261117/rust-validation.json),
[보호 Native](../assets/open_prs_20261117/protected-native-validation.json)·
[보호 WASM](../assets/open_prs_20261117/protected-wasm-validation.json),
[실제 소비 경로](../assets/open_prs_20261117/consumer-validation.json)를 따른다.

최종 통합 PR의 code CI·오늘할일 trailing head CI·병합과 원 PR/실제 해결 이슈 close는 아직 확정하지 않는다.
원 fork를 수정하지 않으며, 통합 병합 뒤 검토 결과와 확정 SHA를 원 PR에 안내한다.

## 후속 코멘트 계획

통합 병합 뒤 원 기여의 성과와 메인터너의 PDF 재산출·필요한 보정 범위를 분리해 설명한다.
웹한글기안기 PDF를 판정에서 제외한 사실과 실제 데스크톱 제품·Print 경로를 코멘트에도 적는다.
해당 PR의 같은 쪽 raw·PDF·review·overlay를 merge SHA 고정 이미지로 직접 표시한다.

## 최종 실제 출력과 회귀 보강

원본 1쪽의 Native/fresh WASM과 실제 Studio Canvas2D·CanvasKit compat/default,
Native SVG·WASM SVG가 독립 Print 대비 각각 **100.00%**다.
같은 원문·쪽의 review·overlay를 직접 판독했으며 허용 반경 내 안티앨리어싱·획 차이는
RGB 완전 동일성을 뜻하지 않는다. 실제 Studio는 원본 함초롬바탕 TTF를 공식 host-font API로 공급했다.
CanvasKit software 실제 backend·typeface 준비·fallback 없음과 초기화 JS/WASM 응답 해시를 확인했다.
[소비 경로 TSV·raw/PDF/review/overlay](../assets/open_prs_20261117/consumer-validation.json)를 따른다.

독립 Print의 `가`/`X` 좌표 ±2px, 정상 문단 피치와 세 줄 소속·워드 호환 제외 관계를 함께 검사한다.
