# Task M100-1251 Stage 8 완료 보고

- **이슈**: [#1251](https://github.com/edwardkim/rhwp/issues/1251)
- **브랜치**: `task-1251-ole-chart`
- **작성일**: 2026-06-03
- **단계**: PR 초안 작성 및 공유 자료 정리

## 1. 수행 내용

PR 초안에 포함할 공식 문서, 결정 배경, 구현 결정, 검증 결과, known visual gap을 하나의 보고서로 정리했다.

신규 문서:

- `mydocs/report/task_m100_1251_pr_draft.md`

## 2. 참고 문서 정리

PR 초안에는 다음 문서를 명시했다.

- 한컴 HWP/OWPML 형식 다운로드 센터
- 한컴 HWP 5.0 문서 형식 revision 1.3
- 한컴 차트 문서 형식 revision 1.2
- Microsoft MS-CFB
- Microsoft MS-OLEDS
- `yuankunzhang/charming`

## 3. 결정 배경 정리

PR 초안에 다음 판단을 포함했다.

1. `charming`은 HWP OLE `/Contents` parser가 아니므로 parser는 rhwp 내부에 둔다.
2. 기본 렌더링 경로는 Rust SVG `RawSvg`로 유지한다.
3. `charming`은 `charming-renderer` feature 뒤 native SSR optional adapter로 유지한다.
4. browser/WASM `WasmRenderer` 기본 경로는 DOM id와 ECharts global runtime이 필요하므로 채택하지 않는다.
5. 정답 PDF 대비 visual gap은 후속 object graph parser 확장으로 다룬다.

## 4. PR 생성 전 주의 사항

현재 브랜치는 fetch된 `upstream/devel` 기준 뒤처져 있다. PR 생성 전 최신 `upstream/devel`로 rebase 또는 merge가 필요하다.

작업 트리에는 #1251 외에 `task_m100_1142`, `task_m100_1143`, `task_m100_1144` 문서가 untracked 상태로 존재한다. PR에 포함할 파일을 staging할 때 #1251 범위만 명시적으로 선택해야 한다.

