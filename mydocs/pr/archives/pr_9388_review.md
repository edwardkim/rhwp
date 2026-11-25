# PR #9388 식량안보·제주 문서의 저장 조판과 인쇄 차트 복원

## 최종 판정

**승인.** 제품 제출 head의 로컬 검증과 GitHub CI를 확인했다. 이 검토 기록을 담은 최종 head의 필수 검사와 최신 devel 병합 가능 상태를 확인하고 병합한다.

## 접수와 조판 원칙

[PR #9388](https://github.com/edwardkim/rhwp/pull/9388)은 jangster77의 devel 대상 PR이다. 제품 코드 `24ec51a6f9186d509d3f3285faa6f9f6e067af5e`의 검증 자료를 포함한 제출 head는 `755cf2b9253aacde950fe898025584a1c400871f`다. 식량안보는 저장 경계에 맞춰 표와 글줄의 쪽 소속을 복원하고, 제주는 그림·캡션 및 구형 차트 외곽 괘선을 한컴 인쇄에 맞췄다. 입력과 독립 Print의 커밋 포함·SHA, 저장 프레임과 실제 소비 경계, 원 PR 대비 하락 여부를 대조했다. [원 PR #9327 후속 검토](pr_9327_followup_review.md) · [원본·인쇄 신원](../assets/issue_9326/20261124/print-provenance.json)

## 로컬 검증

- 전체 Nextest 14,074건 통과, 실패 0건, 45건 건너뜀. Native Skia 단위 검사 4,032건과 집중 검사 39건, Native·WASM Clippy, 전체 target Clippy, workspace 빌드를 통과했다. macOS fresh WASM은 잠금 wrapper의 `--no-opt` 빌드를 사용했다.
- 식량안보 141쪽과 제주 313쪽을 Native와 fresh WASM으로 각 전쪽 비교했다. 총 908건 모두 98% 이상이며 원 PR 대비 하락 0쪽과 devel 대비 2%p 초과 하락 0쪽을 확인했다. 기준은 Windows 데스크톱 한컴 2020·2024의 Hancom PDF 인쇄다. [전쪽 TSV와 대표 review·overlay](pr_9327_followup_review.md)
- 중첩 표 Native는 17쪽 모두 100.00%이며, 2·7·17쪽을 상향했다. 정본 보호 46조건·4,046쪽 하락 0쪽, 기여자 공통 실물 8건과 계약 검사 33건 통과. [정본 보호 비교](../assets/issue_9326/20261124/protected/README.md) · [하한 갱신](../assets/visual-protected-baseline/updates/issue9326-20261124.json)
- Studio Canvas2D 주요 쪽 14건과 CanvasKit 차트 대조군 2건을 실제로 실행했다. 두 원본은 devel과 후보 모두 CanvasKit 200,000 작업 한도에서 Canvas2D로 전환된다. 후보의 전환 출력 14건은 Canvas2D 직접 선택과 같은 PNG였다.

## 제출 head의 GitHub CI

`755cf2b9253aacde950fe898025584a1c400871f`에서 성공 34건, 건너뜀 5건, 중립 0건, 실패·대기 0건을 확인했다.

- [CI](https://github.com/edwardkim/rhwp/actions/runs/45362720088) 성공
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/45362719536) 성공
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/45362720178) 성공
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/45362720078) 성공
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/45362719858) 성공
- [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/45362719742) 성공

## 병합 후 범위

[이슈 #9326](https://github.com/edwardkim/rhwp/issues/9326)의 지정 미달 쪽과 [이슈 #8997](https://github.com/edwardkim/rhwp/issues/8997)의 표 2-5 분할 경계를 확인했다. 최종 병합 SHA에서 두 이슈의 닫힘과 후속 증적을 확인하고 [원 PR #9327](https://github.com/edwardkim/rhwp/pull/9327)의 1·2·96·97·98쪽 증적을 후속 검토에 따라 연결한다.
