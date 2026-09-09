# 구현계획서 — Task #1646

**제목**: README에 브라우저 확장 스토어 배지 추가
**마일스톤**: M100 · **이슈**: edwardkim/rhwp#1646 · **브랜치**: `local/task1646` (base: `upstream/devel`)
**확정 방향**: 기존 README 상단 배지 줄은 유지하고, 브라우저 확장 스토어 배지는 바로 아래 별도 가운데 정렬 문단으로 추가한다.

## 수정 대상

`README.md` 상단 배지 영역:

- 기존 `<p align="center">` 배지 문단은 수정하지 않는다.
- 기존 배지 문단 직후에 새 `<p align="center">` 문단을 추가한다.
- 새 문단에는 Chrome, Edge, Firefox 순서로 확장 스토어 배지를 배치한다.

`README_EN.md`는 이번 구현에서 수정하지 않는다. 구현 후 상단 구조만 재확인해 후속 적용 필요 여부를 보고서에 남긴다.

## Stage 1. README 배지 문단 추가

1. `README.md`의 현재 상단 배지 문단을 확인한다.
2. 기존 배지 문단의 닫는 `</p>` 다음에 새 가운데 정렬 문단을 추가한다.
3. 새 문단에 다음 배지를 삽입한다.
   - Chrome Web Store
     - 링크: `https://chromewebstore.google.com/detail/pgakpjflombjmehnebnbpnalhegaanag`
     - 이미지: `https://img.shields.io/chrome-web-store/v/pgakpjflombjmehnebnbpnalhegaanag?label=Chrome%20Web%20Store&logo=googlechrome&logoColor=white`
   - Edge Add-ons
     - 링크: `https://microsoftedge.microsoft.com/addons/detail/rhwp/nfkdfobhmanddlhdbclkpoanbccpigcn`
     - 이미지: `https://img.shields.io/badge/Edge%20Add--ons-Store-0078D7`
   - Firefox Add-ons
     - 링크: `https://addons.mozilla.org/firefox/addon/rhwp-free-hwp-editor/`
     - 이미지: `https://img.shields.io/amo/v/rhwp-free-hwp-editor?label=Firefox%20Add-ons&logo=firefoxbrowser&logoColor=white`

## Stage 2. 문서 검증

1. `git diff -- README.md`로 변경 범위가 README 상단 배지 문단 추가에 한정되는지 확인한다.
2. `git diff --check`로 공백 오류를 확인한다.
3. `rg`로 세 스토어 URL과 Shields.io 배지 URL이 모두 `README.md`에 존재하는지 확인한다.
4. `README_EN.md`의 상단 배지 구조를 다시 확인하고, 이번 변경에서 제외한 사실을 단계 보고서에 기록한다.
5. 문서 전용 변경이므로 cargo 빌드/테스트는 실행하지 않고 생략 사유를 단계 보고서에 기록한다.

## 산출물

| 종류 | 경로 |
|------|------|
| 소스 | `README.md` |
| 수행계획서 | `mydocs/plans/task_m100_1646.md` |
| 구현계획서 | `mydocs/plans/task_m100_1646_impl.md` |
| 단계 보고서 | `mydocs/working/task_m100_1646_stage1.md` |
| 최종 보고서 | `mydocs/report/task_m100_1646_report.md` |

## 리스크와 대응

- Shields.io 동적 배지는 외부 서비스 상태에 영향을 받는다. Chrome/Firefox만 공식 Shields.io 엔드포인트를 쓰고, Edge는 정적 배지로 README 안정성을 우선한다.
- 기존 배지 줄에 확장 배지를 섞으면 상단이 과밀해질 수 있다. 별도 문단으로 분리해 기존 배지 줄의 가독성을 유지한다.
- `README_EN.md`와 한국어 README의 상단 구조가 달라질 수 있다. 이번 이슈 범위가 루트 README이므로 변경은 보류하되 보고서에 후속 선택지를 남긴다.
