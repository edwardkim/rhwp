# PR #8040 메인테이너 보정 시각 증거

2026-10-29, code source `f8f31d0166`, Print/compat2022/96DPI. Native Skia와 새 release WASM의 실제 Chrome Canvas를 사용했다. WASM 빌드는 `wasm-opt`까지 완료했다. 정확한 입력·기준 PDF·소스·binary/package SHA-256은 [provenance.json](provenance.json)에 있다.

## 같은 편집의 독립 기준

- 원본: `samples/exam_social.hwp`, 독립 기준 `pdf/issue7642-exam-print/exam-08-exam_social-hwp-print-2020.pdf`.
- 삽입: 구역 1, 문단 0, 문자 offset 1에 `가` 삽입. `samples/pr8040/exam-social-insert.hwp`와 `pdf/pr8040/exam-social-insert-2020.pdf`.
- 삽입·삭제: 같은 글자 한 자를 삭제. `samples/pr8040/exam-social-insert-delete.hwp`와 `pdf/pr8040/exam-social-insert-delete-2020.pdf`.
- 두 편집 저장본은 한컴 2020 Print로 새로 인쇄했다. 각 4쪽이며 최종 소스가 내보낸 저장본 SHA-256이 인쇄 요청 입력과 같다. live 편집 출력은 저장본 재열기 출력으로 대체하지 않았다.
- 정상 대조군: `samples/hwpx/exam_social.hwpx` / `pdf/issue7642-exam-print/exam-13-exam_social-hwpx-print-2020.pdf`, 4쪽.

## 전쪽 raw2px 실루엣 TSV

색상 경계 보정 전 `silhouette_raw_match_percent`를 사용한다. 각 TSV는 canonical `scripts/visual_sweep.py --silhouette-only` 산출 원문이다. 평균으로 미달 쪽을 상쇄하지 않았다.

| 상태 | Native | fresh WASM | 쪽수 |
| --- | --- | --- | --- |
| 편집 전 원본 | [전쪽](native-original.tsv) | [전쪽](wasm-original.tsv) | 각 4/4 |
| 삽입 직후 | [전쪽](native-insert.tsv) | [전쪽](wasm-insert.tsv) | 각 4/4 |
| 삽입·삭제 직후 | [전쪽](native-insert-delete.tsv) | [전쪽](wasm-insert-delete.tsv) | 각 4/4 |
| HWPX 정상 대조군 | [전쪽](native-hwpx-control.tsv) | [전쪽](wasm-hwpx-control.tsv) | 각 4/4 |
| 머리말만 보정한 1a7c5ad250 — 실패 증거 | [전쪽](header-only-native-insert-delete.tsv) | [전쪽](header-only-wasm-insert-delete.tsv) | 각 4/4 |

삽입·삭제의 후보 전쪽 값은 Native **98.37/98.49/98.58/98.49%**, WASM **99.83/98.74/99.09/99.07%**다. 삽입만 한 경우 WASM 2쪽은 98.75%이고 나머지 표시 값은 같다. 후보의 미달·누락은 0이다. 머리말만 보정한 비교군은 동일 명시 글꼴 환경에서 Native 최저 53.21%, WASM 최저 53.14%다. 이 실패 자료를 완료 증거로 사용하지 않는다.

Native 원본은 머리말 단독 후보와 본문 보정 후 네 쪽 모두 픽셀이 같으며, 최종 소스에서 삽입·삭제 뒤에도 네 쪽 모두 편집 전과 같다. 삽입만 했을 때 바뀌는 영역은 2쪽의 해당 문단이다. HWPX 정상 대조군도 고정 devel과 후보 네 쪽 모두 픽셀이 같다.

## 직접 판독

- 2·4쪽: Even 머리말이 복원되고 홀수 머리말·쪽번호가 유지된다. 양 단의 문항 순서, 표 외곽, 뒤 문단, 바닥 안내 상자와 쪽 소속을 확인했다.
- 4쪽 16번: 중첩 3행 표 첫 행의 상하 안 여백, 뒤 두 행의 테두리·텍스트 원점, 이탤릭 본문과 `(가)` 상자, 뒤 안내 문단을 같은 배율로 대조했다. 편집 후에만 첫 행이 3.76px 줄던 차이가 해소됐다.
- 4쪽 20번: 분수의 분자/분모·괘선·곱셈 기호, 조건 상자의 시작/끝, 후속 보기 간격을 확대 대조했다. 글자 누락·중복·새 겹침은 관찰하지 않았다.
- 글리프 굵기·윤곽과 일부 자간은 독립 Print와 남은 차이가 있다. 특히 Native의 가는 획과 숫자, 머리말 글리프 모양은 편집 전에도 존재한다. 이 PR의 배치 복원과 구별하며 완전 일치 또는 새 절대 px 완료 기준으로 주장하지 않는다. 글꼴 예외를 사용하지 않았다.

| 대표 쪽 | Native | fresh WASM |
| --- | --- | --- |
| 2쪽 | [review](native-p002-review.png) · [overlay](native-p002-overlay.png) | [review](wasm-p002-review.png) · [overlay](wasm-p002-overlay.png) |
| 4쪽 | [review](native-p004-review.png) · [overlay](native-p004-overlay.png) | [review](wasm-p004-review.png) · [overlay](wasm-p004-overlay.png) |

[머리말 확대](header-zoom.png) · [중첩 셀 확대](nested-zoom.png) · [수식·간격 확대](formula-zoom.png)

## 글꼴 공급

메인테이너의 로컬 변환 OTF, Windows 글꼴, 한컴 2020 설치 글꼴을 사용했다. 변환 OTF는 같은 family의 문자권별 프로그램이 나뉘므로 기존 FontEnvironment의 문서 language slot/font ID별로 실제 PostScript 이름과 프로그램 해시를 명시했다. Native의 공급 프로그램/PostScript 조회와 Canvas FontFace 등록 경로를 사용하며 폰트 이름 테이블이나 글리프 바이트를 바꾸지 않았다.

[환경](font-environment.json) · [언어 슬롯별 실제 파일 이름·SHA-256](font-programs.json). 폰트 파일은 로컬 사용 한정이며 이 증거에 포함하지 않는다. 로컬 루트는 `/home/edward/mygithub/ttfs/hft`, `/mnt/c/Windows/Fonts`, `/mnt/c/Program Files (x86)/Hnc/Office 2020/HOffice110/Shared/Fonts`다. 세 경로가 실제 존재함을 확인했다. HWPX 대조군은 원본 family 공급을 사용하며 HWP의 font ID 선택표를 재사용하지 않았다.

## 재현 경로와 검사

Native는 `NativePngExporter`의 Print context에 같은 FontEnvironment를 설정하고 실제 `insert_text_native(1,0,1,"가")`, `delete_text_native(1,0,1,1)` 전후를 직접 출력했다. WASM은 canonical `export-wasm-png-for-sweep.mjs`의 글꼴·그림 준비 후 같은 API `insertText`/`deleteText`를 실행하고 실제 Canvas를 캡처했다. 편집 후 쪽수와 페이지 텍스트 receipt도 확인했다. 각 편집 상태의 전체 PNG와 같은 상태의 PDF PNG를 `--png-pair`로 비교했다.

공통 배치의 별도 SVG 소비도 4쪽에서 확인했다. 동일 폰트 환경의 편집 전/삽입·삭제 후 SVG가 바이트 단위로 같았고, 로컬 폰트를 공급한 Chrome raster에서 머리말·중첩 셀·뒤 본문을 직접 읽었다. SVG raster를 Native Skia 또는 WASM Canvas 증거로 대신하지 않는다. 실행 로그·SVG 및 폰트가 포함된 임시 산출물은 ignored `output/pr-review/pr8040/`의 정리 대상이다.

신규 정식 `tests/cases/issue_8040_later_section_edit.rs`는 설치 글꼴 없이 문서 쪽·본문/표/셀/머리말 소유와 순서, 중립 편집의 배치 불변 관계를 검사한다. 독립 PDF의 절대 좌표를 새로 고정하지 않는다. 기존 `verified_social_print_host_frames`의 독립 근거가 있는 1쪽 좌표 검사는 변경하지 않았으며 HWP/HWPX·분리본 4건이 통과했다. 전체 suite/lint 결과는 [검토 기록](../../archives/pr_8040_review.md)을 따른다.
