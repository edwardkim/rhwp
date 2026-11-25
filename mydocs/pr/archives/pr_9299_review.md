# PR #9299 한국어·영어 README의 감사 문구

## 최종 판정

**승인.** 2026년 11월 22일 메인테이너 `edwardkim`이 제출 커밋 `766115acb43fad68228c0a0df17289a68e233b72`을 셀프 리뷰했다. GitHub에서 두 언어의 감사 문구와 배지를 확인했고, 같은 커밋의 필수 검사도 통과했다. 이 검토 기록을 추가한 뒤에는 최종 커밋의 검사와 최신 `devel` 병합 결과를 확인한다. [PR #9299](https://github.com/edwardkim/rhwp/pull/9299)

## GitHub에서 확인한 표시

한국어와 영어 문장 모두 rhwp 기여자의 개발·유지보수 활동을 지원한 OpenAI에 감사를 표한다. 기존 `Notice` 바로 앞에 제목, 배지, 감사 문장이 차례로 한 번씩 표시됐다. 두 파일 모두 추가한 6줄을 제거하면 이전 본문과 바이트가 일치했다. [한국어 README](https://github.com/edwardkim/rhwp/blob/766115acb43fad68228c0a0df17289a68e233b72/README.md#감사의-말-acknowledgments) · [영어 README](https://github.com/edwardkim/rhwp/blob/766115acb43fad68228c0a0df17289a68e233b72/README_EN.md#acknowledgments)

Chrome 152에서 GitHub가 렌더링한 README를 열어 배지가 실제로 로드됐는지 확인하고, 해당 영역을 직접 판독했다. 배지의 대체 텍스트와 SVG 제목은 모두 `Contributor support: OpenAI`다. 로고 없는 170 × 20 이미지가 두 화면 너비에서 온전히 표시됐고, 모바일에서는 감사 문구가 열 너비에 맞춰 줄바꿈됐다.

| 브라우저 화면 | 한국어 | 영어 |
| --- | --- | --- |
| 데스크톱 1280 × 900 | [감사 문구와 Notice](../assets/pr9299/ko-1280.png) | [Acknowledgments와 Notice](../assets/pr9299/en-1280.png) |
| 모바일 너비 390 × 844 | [한국어 줄바꿈](../assets/pr9299/ko-390.png) | [영어 줄바꿈](../assets/pr9299/en-390.png) |

## 병합 검증

- 원래 기준은 `5235b4d131`이며, 최신 `devel` `1a27faacff48ea7329e76ce8eca15aa9ca8aa877`과 제출 커밋을 병합한 트리는 `3681f859c9153b1fe8f91abdf408025ece577e6f`였다. 이 트리와 최신 기준의 차이도 두 README에 추가된 12줄이었다. `git diff --check`는 통과했다. 변경 범위는 GitHub 문서이므로 HWP 조판과 문서 파일 입력 검증은 비해당이다.
- 원 head의 CI Impact Policy, Adapter, Proptest가 성공했다. 두 README는 CI의 자동 실행 제외 경로여서 최초에는 보호 브랜치가 요구하는 `Build & Test` 결과가 생성되지 않았다. [경로 필터](https://github.com/edwardkim/rhwp/blob/766115acb43fad68228c0a0df17289a68e233b72/.github/workflows/ci.yml#L7-L25) · [정책 검사](https://github.com/edwardkim/rhwp/actions/runs/44933977027)
- 같은 커밋에서 기존 CI를 기본값으로 한 번 수동 실행했다. 회귀 A–E, Native Skia, 포맷·Clippy, WASM 빌드, 프런트엔드 패키지 검사와 `Build & Test`가 통과했다. 실행 결과는 19개 성공, 조건에 따른 생략 3개였다. GitHub Actions가 이 커밋에 필수 검사 성공을 등록한 뒤 검토 기록을 작성했다. [전체 CI](https://github.com/edwardkim/rhwp/actions/runs/44968779862) · [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/44968779862/job/137050913246)
