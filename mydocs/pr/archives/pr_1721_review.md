# PR #1721 리뷰 — #1720 개체 단위 시각/geometry 회귀 인프라

- PR: #1721 `Task #1720: 개체 단위 시각/geometry 회귀 인프라 (object_visual_regression.py)`
- 작성자: @planet6897
- 기준: `devel`
- 원 PR head branch: `planet6897/pr/devel-1720`
- 검토 대상 커밋: `ae76ea6f8b476609626009b75e36fe2dce6c5050`
- 규모: 4 files, +527/-0
- 관련 이슈: #1720
- 문서 작성 시점 상태: `MERGEABLE`, GitHub Actions 통과(원 PR 기준)
- 처리 방침: #1719와 함께 `upstream/devel` 기준 통합 cherry-pick PR 로 수용 후보
- 최종 처리: 통합 PR #1727 merge commit `150ca316ee557d6bf95928302166e037d7467b03` 으로 반영 완료, 원 PR #1721 supersede close 완료

## 변경 요약

`tools/object_visual_regression.py`를 추가해 개체 단위로 rhwp 렌더 결과와 한글(OLE) 기준 출력을
대조할 수 있게 한다. #1718에서 남은 42쪽 vs 48쪽 격차를 개체 배치 관점으로 조사하기 위한
보조 인프라다.

주요 기능:

- `export-render-tree`에서 depth 1 이상 중첩 `Table` 개체 추출
- 한글 COM -> PDF -> PyMuPDF 기반 기준 렌더/개체 bbox 추출
- `export-png` 기반 rhwp 페이지 PNG 생성 옵션
- 문자 3-gram Jaccard 기반 표 내용 매칭, 텍스트 없는 개체는 크기 기반 폴백
- `objects.tsv`, `gallery.html`, `baseline.json` 생성
- `--baseline --no-hwp` 조합으로 rhwp 버전 간 개체 geometry 회귀 검출

## 로컬 검증

통합 브랜치 `codex/pr1719-1721-bundle`에서 #1719 실제 변경 커밋 2건과 #1721 커밋 1건을
`upstream/devel` 위에 cherry-pick 한 뒤 검증했다.

- `python3 -m py_compile tools/object_visual_regression.py`: 통과
- `python3 tools/object_visual_regression.py --help`: 통과, argparse CLI 표시 확인
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo fmt --check`: 통과
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo clippy --all-targets -- -D warnings`: 통과, 38초

macOS 로컬 환경에는 Windows 한글 COM/pyhwpx 실행 조건이 없으므로 한글 대조 end-to-end는 수행하지 않았다.
대신 `--help`와 `py_compile`로 Python 진입 경로를 확인했고, rhwp 쪽은 #1719 샘플 `dump-pages` 및 cargo
검증으로 확인했다.

## GitHub Actions

원 PR #1721의 문서 작성 시점 참고값:

- CI preflight: success
- CodeQL preflight: success
- Render Diff preflight: success
- WASM Build: skipped
- Analyze (javascript-typescript): success
- Analyze (python): success
- Analyze (rust): success
- Canvas visual diff: success
- Build & Test: success
- CodeQL: success

최종 merge 판단은 통합 cherry-pick PR head 기준 GitHub Actions 통과를 조건으로 한다.

## 리뷰 결과

Blocking finding 없음.

도구는 기본적으로 실패 시 명확한 exit code를 반환하고, Windows/한글 의존 구간이 실패해도 rhwp-only
결과를 계속 낼 수 있게 구성되어 있다. 내용 기반 매칭을 우선하고 크기 기반은 텍스트 없는 개체에만
폴백으로 쓰는 점도 전폭 표 오매칭 리스크를 줄인다.

## 리스크 / 후속 확인

- 한글 대조 전체 파이프라인은 Windows + 한컴 + pyhwpx + PyMuPDF 환경에서만 완전 검증 가능하다.
- `export-render-tree`가 포착하지 못하는 프레임 없는 인라인 그림은 한글 이미지 bbox와 매칭 단위가 다를 수 있다.
- 통합 PR merge 후 #1720 auto-close 여부를 확인하고, 실패 시 수동 close 한다.
- 통합 PR merge 후 원 PR #1721에는 supersede/통합 반영 코멘트를 남기고 close 한다.

## 최종 판단

통합 cherry-pick PR 로 수용 가능.

## merge 후 확정 기록

- 통합 PR: #1727
- merge commit: `150ca316ee557d6bf95928302166e037d7467b03`
- 원 PR #1721 상태: supersede close 완료
- 관련 이슈 #1720 상태: 수동 close 완료
- #1720 인프라 사용 후속 이슈: #1728 `RowBreak 표 continuation 5~6쪽 PDF 기준 시각 차이 보정`
- 후속 샘플 보강:
  - `samples/table_giant_cell_overfill.hwpx`
  - `pdf/table_giant_cell_overfill-2024.pdf`
  - `samples/table_scattered_header_rowbreak.hwp`
  - `pdf/table_scattered_header_rowbreak-2024.pdf`

### 후속 확인

macOS 로컬에서는 Windows 한글 COM end-to-end를 실행하지 않았으므로 #1720 도구의 한글 대조 전체
파이프라인은 후속 환경에서 계속 확인한다. 이번 후속 PR에는 #1728 재현에 필요한 PDF 기준 파일과
HWP/HWPX 비교 입력을 함께 포함한다.
