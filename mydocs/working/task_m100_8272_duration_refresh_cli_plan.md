# #8272 duration refresh CLI의 Archive E 입력 거부 보정

- 이슈: [#8272](https://github.com/edwardkim/rhwp/issues/8272)
- 작업 브랜치: `task/8272-duration-e-cli`
- 기준: `2546c8b2ab7041399fb15fe82e0f50c1ade21724` (#8188 merge)
- 사용자 지시: 이 오류를 별도 이슈로 등록한 뒤 개선한다. 원 PR의 review·오늘할일은 옵션 2로 분리한다.

## 원인과 선행 근거

[Metadata run 40290628059](https://github.com/edwardkim/rhwp/actions/runs/40290628059/job/121464579118)는
검증된 B/C/D/E 실측을 수집한 뒤 CLI main의 옛 B/C/D 전용 label 검사에서 실패했다.
공통 갱신 함수와 인자 개수는 이미 네 archive를 지원했으나 CLI 최종 검사가 남았다.
순수 함수·수집/발행 검증만으로 실제 CLI 네 입력 경로를 입증하지 못한 검증 누락이다.

원 코드의 A~E 기본 11,379개·Native·CodeQL·Render Diff·CI Impact Policy는 모두 성공했다.
제품 Rust·source/PDF·전쪽 100% TSV는 바꾸지 않는다.

## 구현과 검증

1. CLI의 중복된 옛 layout 검사를 제거한다. 공통 함수가 B/C/D 또는 B/C/D/E의 정확한
   단일 집합을 검사하게 하며, 실패를 숨기거나 미완성 policy를 발행하지 않는다.
2. 실제 Node 프로세스로 schema 1/2 각각 세 개·네 개 JSON 파일을 읽고 output policy와
   E의 provenance·target 보존을 검사한다. 누락·중복·알 수 없는 label·개수 오류는 실패와 미발행을 검사한다.
3. 관련 Node/Python·actionlint를 실행하고 원 CLI에서 네 입력이 실패하는 음성 대조도 남긴다.
   원 PR source·test·입력/PDF·TSV 동일성을 Git으로 확인한다.
4. 새 이슈의 전용 PR에서 최신 head CI를 확인한다. 병합 후에는 metadata refresh의 실제
   결과를 확인하고 제품 검증 CI를 다시 실행하지 않는다.

## 기존 작업 보존

#8188 검토·오늘할일 후보는 `task/8160-sample16-review-records`의 `d7509e78c`에 보존한다.
이번 코드 보정을 그 문서 PR에 섞지 않는다. 공유 `target/pr-review`, 원본·PDF·개인 글꼴,
`/home/tsjang/rhwp-pr-evidence/pr-8188/`의 최종 증적을 보존한다.

## 로컬 검증 결과

- 관련 Node 136개·Python 44개 및 전체 actionlint 통과.
- CLI 검사는 schema 1/2 각각 B/C/D와 B/C/D/E 출력의 provenance·E target을 확인하고,
  누락·중복·알 수 없는 label·개수 오류에서 실패와 output 미발행을 확인한다.
- 같은 CI run 40286124631의 실제 B/C/D/E report 네 개를 그대로 내려받아 대조했다.
  원 merge commit CLI는 exit 1로 E 입력을 거부했고, 보정 CLI는 exit 0이었다.
  결과는 46 target·1,730 source case·7,578 testcase를 보존했다. 이는 integration 실측이며
  lib archive A의 3,801개를 더한 root default 실행 수 11,379개와 구분한다.
- Python 구조 검사가 옛 B/C/D 전용 오류 문구를 요구해 잘못된 중복 검사를 고정하고 있었다.
  공통 함수 위임 연결로 정정하고 실제 허용/거부 동작은 CLI 회귀로 보호했다.
- 임시 report·실행 로그·음성 대조 스크립트는 ignored output에 두며 PR에 포함하지 않는다.

## 최종 CI와 병합 후 결과

- 최종 head `fc77cd99add15a9b4aa8383c46433bc608ad3770`의 [PR #8273](https://github.com/edwardkim/rhwp/pull/8273)은 CI·CodeQL·CI Impact Policy가 통과했고 기본 archive 11,379개 실행/통과가 예상 수와 일치했다.
- `61ec35daa499b375b1dc6b66a4f5f6d6cc6ad0aa`로 일반 merge했다. Metadata run 40298571437도 성공했고, B/C/D/E 실측은 `222ae0a6a578efb3ddad1fe8ea7186cbc060f713`으로 실제 발행됐다.
- 발행 JSON의 네 label·원 head/run 및 46 target·1,730 source case·7,578 testcase를 다시 확인했다. 검토·오늘할일은 사용자 지정 옵션 2의 별도 문서 PR #8274에 반영한다.
