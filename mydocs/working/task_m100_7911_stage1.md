# Task #7911 1단계 보고: PDF2IR 제품 통합 설계

- 일자: 2026-10-15 (Asia/Seoul)
- 상태: 조사·설계 완료, 구현계획 승인 대기
- [수행계획](../plans/task_m100_7911.md), [구현계획](../plans/task_m100_7911_impl.md)
- 메인테이너 승인: “수행계획 승인, 통합 설계 진행”. 제품 코드는 아직 변경하지 않았다.

## 1. 고정 상태

| 항목 | 확인 결과 |
| --- | --- |
| 조사 head | `task_m100_7911` / `92df2f3cbdc7012f5e91635512ceda68debecfac` |
| 로컬/추적 devel | `a5e88ad682c805e1d845502ddc9521b94ea8eccd`로 일치 |
| 원격 | `origin = git@github.com:edwardkim/rhwp.git`; upstream alias 없음 |
| POC head | `6bc3847e0ff3e1302736914859903f3c9c51bb69` |
| 공통 조상 | `07e7e813d06d3f8031e86eb7c27233b829964907` |
| POC 고유 변경 | 9 commit, 31파일, 9,337줄 추가; 모두 `tools/pdf-ir-poc/` |
| 이슈/PR | #7911 OPEN, assignee edwardkim; 현재 branch의 PR 없음 |

`git status`, `git log`, `git diff --stat`, `git show`, `rg`/`sed`와 `gh` 조회로 조사했다.
POC branch를 checkout/merge하지 않고 고정 commit의 파일을 읽었다.
issue/PR 상태는 조회 당시 값이며 다음 원격 작업 전에 다시 확인한다.

## 2. 실제 호출 경로

| 생산/소비 경계 | 조사 위치 | 판단 |
| --- | --- | --- |
| format detect/parse | `src/parser/mod.rs:44`, `:205`, `:1611`, `:1652` | PDF case 없음; metadata 분리 선례는 HML |
| 공통 로드 | `src/document_core/commands/document.rs:158`, `:174` | parse 후 styles/fixup/compose/paginate, 독립 constructor 복제 불필요 |
| CLI loader | `src/cli/document_io.rs:77`, `:86` | 공통 bytes 경로를 연결하면 기존 export가 소비 |
| 실제 CLI dispatch | `src/main.rs:127`, conversion 모듈 | `export-hwpx`, `convert`가 실제 명령; 별도 POC CLI는 불충분 |
| WASM constructor/source | `src/wasm_api.rs:441`, `:564`, `:590` | PDF 이름 누락 시 현재 fallback은 hwp; 신규 분류 필요 |
| provenance/profile | `src/model/provenance.rs:13`, `src/model/document.rs:386` | PDF를 기존 출처로 가장하지 않고 일반 IR 계약 생성 |
| 글상자 저장 | `src/serializer/hwpx/shape.rs:521`, `:588` | 네 여백 0이면 textMargin 생략; 명시적 0/부재 분리 필요 |

POC는 `extract -> recognition/table topology/hierarchy -> lowering -> Document` 다음
HWPX와 XML ZIP 여백 보정을 만든다. 제품은 Document 직후 공통 Core로 연결해야 한다.
POC는 쪽마다 독립 style/resource pool을 생성하므로 여러 Document를 합치는 것으로 다중 쪽을 구현하면
참조 충돌 위험이 있다. 하나의 builder와 전체/선택 쪽 계약을 구현계획에 포함했다.

문단 여백의 R3 물리 단위, 실제 탭 폭과 자식 표 호스트 점유는 producer 책임으로 보존한다.
관측 줄 단위 Paragraph는 임시 정책이다. 원본 논리 문단이나 편집 재조판 완성을 주장하지 않는다.

## 3. 의존성·안전성 조사

로컬 Cargo registry의 pinned source를 읽었다. 설치·업데이트·실행 공격 실험은 하지 않았다.

- `pdf_oxide 0.3.78`은 `office_oxide ^0.1.9`를 default feature 없이 의존한다.
  두 crate의 로컬 manifest는 `MIT OR Apache-2.0`을 선언한다. 전체 전이 의존성·NOTICE 감사 완료는 아니다.
- `pdf_oxide/src/document.rs:1615`의 `open_with_config`는 `_config`를 받는 stub이다.
  이 함수로 브라우저 bytes 입력의 제한을 구성했다고 주장할 수 없다.
- `decoders/mod.rs:212`는 `decoder.decode` 후 크기/비율을 검사한다.
  Flate의 개별 출력 상한이 있어도 문서 누적/다른 decoder의 선행 할당 상한은 별도다.
- `decoders/brotli.rs:12`의 `read_to_end` 경로를 확인했다. 추출 이후 spans/images/paths 카운터는
  이 할당의 선행 제한이 아니다. 즉시 외부 비신뢰 입력에 안전하다는 결론을 내리지 않는다.
- 초기 opt-in, 사전/누적 budget·오류 복구·Worker 취소를 설계했다. 공개 dependency API로 필요한
  선행 제한을 달성하지 못하면 배포 준비를 보류하고 최소 patch 범위를 별도 승인받는다.

## 4. 검증 범위와 남은 게이트

이번 단계는 source·문서 조사다. 제품 build/test, POC 71개 재실행, fresh WASM, 전쪽 TSV,
한컴 MCP Print는 **미실행**이다. 이전 선택 7/8쪽의 수치는 이전 POC에만 유효하다.
12쪽 전체 진단은 전체 12쪽 변환/시각 통과가 아니고 3쪽 Deferred도 남아 있다.

본인 PR 경로와 local validation 4.3, visual fixture evidence, rework 절차를 확인했다.
전쪽 90%·쪽수/직접 판독 외에 기존 포맷의 최신 pinned devel 대비 페이지별 0.01%p 하락도 재검토한다.
새 실물 회귀는 선행 시각 검증 뒤 추가한다. 현재 제품 코드가 없어 PR 준비 완료로 판정하지 않는다.

- 문서 검증: `python3 scripts/check_markdown_links.py`에 이 단계의 세 문서를 지정해 실행했고
  3개 문서의 내부 상대 링크에 이상이 없었다. `git diff --check`도 통과했다.
- 변경 범위는 수행계획·구현계획·1단계 보고서 3개뿐이다. 제품 파일과 오늘할일은 수정하지 않았다.

문서-only 단계 commit으로 남긴다.
구현계획 승인 후 2단계 POC 이력 통합과 직접 IR frontend 구현을 시작한다.
이번 단계에서 devel 변경, push/PR 생성/merge/issue close는 하지 않는다.
