# 영월 서식의 월별·누계 셀

시각 source: `6ce82238e64f7ad7c7306db33b6374692933ab43`.
이 증적 이후 변경은 테스트·기록만 포함하며 생산 코드는 같음을 확인한다.

입력: `samples/issue5699/20099369_yeongwol_forms.hwp`.
SHA-256: `6d723789148699479983fbcb6fa6368a252c1187fe0a7b3c922b583f6727ec09`.
독립 기준: `pdf/issue7642-failed-inputs-followup-20261011/20099369_yeongwol_forms-2020.pdf`.
SHA-256: `a04da624238d77da4832dd302017c7d2f559a01da02e4c1b4ae12cfd3f77b123`.
기존 PDF와 Print 생성 기록을 재사용한다. 전 4쪽 Native/fresh WASM 모두 최저 99.29000%,
90% 미만·누락 0쪽이며 전체 쪽수도 독립 Print와 일치한다.

대표는 세 번째 쪽이다. 각 모드의 compare·overlay·review와 같은 원점·배율의 확대를 보존한다.
`월별` 뒤 `누계`가 같은 셀 안의 두 줄이고 두 글줄이 겹치지 않는지 직접 확인했다.
글꼴 가장자리의 raster 차이와 실루엣 일치율을 셀 소유·내용 순서 검사와 구분한다.

전체 TSV·생성 명령·패키지/실행 파일 해시는 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`의
`stage396-source393-targeted-native-corrected.json`, `stage397-source393-targeted-wasm-corrected.json`,
각 `stage396/397-corrected-yeongwol-whole-*` 경로에 있다.
정식 검사와 실행 결과는 `mydocs/working/task_m100_7642_stage405.md`에 연결한다.
