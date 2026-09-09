# 하이퍼링크 편집과 PDF 보존 구현 계획

- Issue: [#6963](https://github.com/edwardkim/rhwp/issues/6963)
- 착수 승인: 2026-09-10 사용자 요청(이슈 등록 및 작업 시작)
- 담당: postmelee
- 기준: `upstream/devel` `a3cd825c23d550e0c5f46b4e2eb3735a537f23f2`
- 브랜치: `codex/issue-6963-hyperlinks`
- 상태: 단계 2 완료(코어 링크 편집·HWP/HWPX 저장 왕복), PDF 영역·출력 및 Studio 연결 대기

## 목표와 범위

Studio에서 HTTP/HTTPS 텍스트 링크를 삽입·수정·제거하고 HWP/HWPX로 저장·재열기했을 때
표시 문자열, 주소와 필드 범위가 보존되어야 한다. Studio의 PDF 저장과 CLI PDF 생성에서
클릭 가능한 Link/URI annotation과 실제 글자에 대응하는 페이지별·줄별 영역을 보존한다.

메일·파일·문서 내부 책갈피의 신규 편집 UI는 후속 확장이다. 기존 문서를 여는 과정에서
지원하지 않는 링크를 HTTP 링크로 재해석하거나 삭제하지 않는다. 편집 컨텍스트의 지원 범위를
실제로 확인해, 아직 지원하지 않는 컨텍스트를 본문에 잘못 적용하는 대신 명시적으로 차단한다.

## 현행 경로와 설계 경계

1. `rhwp-studio/index.html`의 버튼 연결, `src/command/commands/insert.ts`의 stub,
   `src/engine/input-handler-keyboard.ts`의 chordMapK를 함께 구현한다.
2. HWP5/HWPX 신규 텍스트 링크는 기존 `Control::Field(FieldType::Hyperlink)`와 `FieldRange`를
   사용한다. HWP3 전용 `Control::Hyperlink`를 신규 저장 모델로 사용하지 않는다.
3. URL과 한컴 Command 문자열의 escape/unescape를 하나의 공통 코어 규칙으로 정리한다.
   확인된 원본은 `https\://...;1;0;0;`, URL fragment의 `\#`·`\:`를 포함한다.
   필드 명령과 URL은 서로 다른 값이므로 전체 Command를 PDF URI로 전달하지 않는다.
4. 편집 mutation은 코어 API와 WASM bridge로 노출한다. Studio에서는 기존 operation router로
   snapshot 또는 도메인 command를 기록해 history·dirty·refresh를 함께 처리한다.
   실패·취소·동일 값 수정은 문서와 history를 불필요하게 바꾸지 않는다.
5. PDF용 링크 영역은 글자수에 비례해 임의 추정하지 않고 레이아웃이 계산한 문자 경계와
   문서 컨텍스트를 사용한다. 기존 `cursor_nav`의 selection rect 질의와 render tree의
   `TextRunNode.char_start`, `cell_context`, `layout_positions`를 대조해 공통 추출 경로를 정한다.
   줄바꿈, 페이지 분할, 셀/글상자, clipping 및 display text의 다른 좌표계를 검증한다.
6. Studio는 `command/print-pages.ts`의 인쇄 DOM과 브라우저 PDF 결과를 검증한다.
   SVG anchor의 존재만으로 PDF 보존을 완료 처리하지 않는다.
7. CLI는 `DocumentCore::render_pages_pdf_native_with_options` 및 profile 변형을 통해
   `renderer/pdf.rs`의 SVG→PDF 생성에 주석을 넣는다. 선택한 페이지 순서 및 변환 실패로
   건너뛴 페이지가 있을 때 주석이 다른 페이지에 붙지 않아야 한다.
8. 선택형 DirectLayer/Skia PDF 경로는 별도로 지원 여부를 판정하고 누락을 보고한다.
   SVG PDF 통과를 direct backend의 통과로 확대하지 않는다.

## 단계와 검증

| 단계 | 결과물 | 직접 검증 |
| --- | --- | --- |
| 1 | 한컴 기준 샘플·SHA-256·URI·PDF 페이지/영역 계약 | 원본 HWPX XML과 PDF annotation, 생성 메타데이터, 대표 raster |
| 2 | 공통 URL/Command codec와 필드 조회·삽입·수정·제거 | 한글·query·fragment·escape, 범위 경계, HWP/HWPX 저장 왕복 |
| 3 | 페이지별 링크 영역 질의와 PDF 출력 | 여러 줄·여러 페이지·부분 출력·혼합 폰트·기존 링크 샘플, PDF annotation 파싱 |
| 4 | Studio dialog·명령·shortcut·라우터 | 선택/무선택, 수정/제거/취소, undo/redo, focus·dirty, 지원 컨텍스트 |
| 5 | Studio→저장→재열기→PDF 전체 여정 | 실제 브라우저, PDF 뷰어 클릭, 한컴 기준 URI·geometry 비교 |

단계 1·2 기록: [기준 샘플 계약](../working/task_m100_6963_stage1.md),
[코어 편집·저장 왕복](../working/task_m100_6963_stage2.md).

단계가 끝나면 해당 단계 변경과 검증 결과를 커밋하고 다음 단계 기록을 시작한다.
전체 feature 완료는 이슈의 모든 1차 범위 완료 조건 충족 후에만 보고한다.

## 한컴 기준 샘플

- `samples/basic/Textmail.hwp` / `pdf/basic/Textmail-2022.pdf`: p1, URI 1개.
- `samples/hwpx_sample2.hwpx` / `pdf/hwpx_sample2-hwpx-2020.pdf`: p2·p8·p11의 URI,
  p8 URL fragment escape. p29에는 실행 URI가 없어 정상 링크의 성공 기준에서 제외한다.
- `samples/hwp-img-001.hwp` / `pdf/hwp-img-001-2022.pdf`: mailto 보존의 확장 후보.
- `samples/hwpctl_Action_Table__v1.1.hwp` / `pdf/hwpctl_Action_Table__v1.1-2022.pdf`:
  p4→p14의 GoTo, 내부 이동 확장 후보.

파일명의 `-2020` engine bucket과 PDF Creator의 Hwp 2022를 구분한다.
`Hyper(hwp2010)`의 PDF destination 경고는 기존 PR #6466 기록에도 있어 정상 성공 기준에서 제외한다.
`k-water-rfp.hwpx`의 첫 URL에는 닫는 괄호·한글 접미사가 붙어 있어 주소가 깨끗한 p8·p13과 구분한다.
기존 사용자 변경 `samples/exam_eng.pdf`는 이 작업의 입력·수정 대상이 아니다.

## 검증 및 원격 작업 절차

현재 작업은 collaborator의 신규 내부 구현이다. PR 채번 이전이므로 오늘할일과 PR review 문서는
생성하지 않는다. 관련 절차는 `manual/pr_review/collaborator_self_merge.md`를 따른다.

- 개발 중: 해당 기능 focused Rust test, TypeScript/관련 Node test, WASM 및 실제 브라우저 확인.
- Rust test source는 `tests/cases/`에 작성하고 파생 suite는 review worktree에서만 준비한다.
- PR 준비: `manual/pr_review/local_validation.md` 4.3의 Rust lint 전체 묶음,
  변경 범위에 해당하는 release-test/Native Skia/WASM/Studio 회귀 및 시각 증적.
- 사용자 승인 범위는 이슈 등록·착수·로컬 구현이다. remote push, PR 생성, merge는 별도 승인 단계다.
- 현재 단계가 완료되어도 전체 제품 기능이나 PDF 보존이 완성됐다고 표시하지 않는다.

## 참고

- [한글 2024 하이퍼링크](https://help.hancom.com/hoffice130/ko-KR/Hwp/insert/hyperlink/hyperlink.htm)
- [HWP/OWPML 공개 규격](https://www.hancom.com/support/downloadCenter/hwpOwpml)
- [한컴 OWPML 모델](https://github.com/hancom-io/hwpx-owpml-model)
- [InsertHyperlink 액션 예제](https://forum.developer.hancom.com/t/topic/2291/2)
- [PDF 32000-1:2008](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf)
  §12.5.6.5, §12.6.4.7.
