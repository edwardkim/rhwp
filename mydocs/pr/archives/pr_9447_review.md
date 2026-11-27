# PR #9447 리뷰 — Studio 문단 들여쓰기와 여백의 단위 보존

## 최종 판정

**승인.** 코드 커밋에서 기능 검증과 GitHub 필수 검사가 통과했다. 검토 문서를 추가한 커밋에서도 필수 검사와 병합 가능 상태를 확인한 뒤 일반 병합한다.

## 검토 범위와 결과

메인테이너 `edwardkim`의 [PR #9447](https://github.com/edwardkim/rhwp/pull/9447)이 [이슈 #8860](https://github.com/edwardkim/rhwp/issues/8860)의 절대 길이 입력과 기존 상대 길이 보존을 해결하는지 검토했다. 기준 `devel`은 `c63b87b046a31b04e12c72dd144aae71497a031e`, 검증한 코드와 증거의 커밋은 `1eba66d135ab9107fb951ca8b04ff111eccc09df`다.

- **쓰기 경로.** 새 pt·화면 px 입력은 정수 HWPUNIT으로 반올림한 뒤 두 배하여 저장한다. 기존 문단의 설정 확인과 모양 복사는 원래의 단위 비트를 보존한다. 형식 변환에서 이 값이 보존되는지 본문·셀의 HWP/HWPX 저장과 두 번째 재열기로 확인했다. 일반 셀에서는 Shift+Tab이 이전 셀로 이동했다. 구버전 WASM과 새 Studio 조합도 확인했다. [실제 브라우저 회귀](../../../rhwp-studio/e2e/paragraph-length-units-issue8860.test.mjs)
- **조판 원칙.** 변경은 편집기의 입력 변환과 문단 속성 조회에 한정된다. 한글 문서의 기존 상대 단위가 바탕글 크기에 따라 달라지는 동작은 같은 입력으로 확인했다. 페이지 배치·렌더러를 변경하는 PR에 적용하는 보호 전쪽 검사는 이 범위에 해당하지 않는다. 20pt 문서에서 1.005pt를 직접 입력하니 한컴 인쇄에서 둘째 줄 이동량이 10.07pt에서 0.96pt로 바뀌었다. 수정 전후의 각 저장본은 해당 인쇄본과 Native·브라우저에서 모두 100.00% 일치했다. [PDF 줄 원점](../../tech/investigations/issue-8860-paragraph-units/hancom-line-origins.tsv)
- **입력 근거.** 검토 커밋에 편집 전 HWP와 두 저장본, 두 독립 한컴 2020 PDF가 들어 있다. 출력 제품은 11.0.0.9172이고 1쪽씩 인쇄했다. 입력·출력의 SHA-256을 작업 기록과 대조했다. 기존 상대 길이 대조군 5쪽은 두 렌더러 모두 100.00%, 단 정의 대조 5쪽은 최저 99.99%였다. [인쇄 내역](../../tech/investigations/issue-8860-paragraph-units/print-jobs.json) · [쪽별 측정](../../tech/investigations/issue-8860-paragraph-units/visual-pages.tsv)
- **회귀 검사.** 전체 Rust 14,219건, Native Skia 4,227건, Studio 2,840건과 필수 정적 검사를 통과했다. 인접 #7490 E2E의 단 정의 기대값은 원본 레코드에 맞춰 교정하고, 단 정의가 없는 원본의 저장도 따로 검사했다. [검증 보고서](../../report/task_m100_8860_report.md)

## GitHub Actions

코드 커밋에서 Rust 회귀, lint, Frontend 패키지, 코드 분석, 렌더 비교, 어댑터 비교와 무작위 왕복 검사가 성공했다. 최종 Build & Test와 CI Impact Policy도 성공했다. [CI](https://github.com/edwardkim/rhwp/actions/runs/45957797213) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/45957797086) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/45957797052) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/45957797195) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/45957797083) · [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/45961863238)

CodeQL은 변경 언어인 Rust와 JavaScript/TypeScript를 분석했다. Actions와 Python은 변경 범위 밖이어서 분석을 생략했다. GitHub의 별도 코드 스캐닝 요약에는 이 두 구성에 대해 중립 상태가 표시됐다. [언어별 작업](https://github.com/edwardkim/rhwp/actions/runs/45957797086)

## 병합 뒤 이슈와 PR에 남길 내용

일반 병합 커밋과 최종 검토 링크, 한컴 인쇄의 둘째 줄 이동량, 문단 모양·Shift+Tab·눈금자·모양 복사의 보완 범위를 #8860에 기록한다. 같은 20pt 문서의 수정 전후 화면과 인쇄본을 연결하고 이슈 종료 상태를 확인한다. [수정 전 화면](../../tech/investigations/issue-8860-paragraph-units/before.png) · [수정 후 화면](../../tech/investigations/issue-8860-paragraph-units/after.png)
