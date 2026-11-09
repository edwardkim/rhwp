# PR #8604 누적 검토 구현 계획

## 대상과 단계

[원 PR #8604](https://github.com/edwardkim/rhwp/pull/8604)의 원 head는 `013459bff4ff5313af53ece5646e79c48647a3aa`다.
[공통 계획](../assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `032dcb80e3de44b3ed4a7ca09549a197b6a18067` | `2d0320a7a5a5bbcf5468e8100d37db187682588b` | fix(typeset): #8596 앞 TAC 표 host 줄 안에 저장된 자리차지 표 프레임은 다음 쪽에서 시작한다 |
| `e74368c345437ca63591ea064c4b972ade999ee7` | `db364293b67e530082eecf90e2e7cf1ee32da096` | docs(#8596): 필수 편람 Native 383쪽 TSV — devel 7fdd5441ff 와 바이트 동일 |
| `013459bff4ff5313af53ece5646e79c48647a3aa` | `018675337754757d7da875e0e7d4ae7a9ba3439a` | docs(#8596): 최신 devel rebase head 재측정 — base·head 전쪽 TSV, 대표 PNG, 편람 gate |

## R42 리베이스 후 적용 상태

새 base `1e16a19aede5ccb2657d35974940eb596286cdf1`, 리베이스 직후 head
`896708a4a02c4138064ab3d426a487bbce52fc02` 기준으로 위 실제 적용 SHA를 갱신했다.
중복 patch 행의 원 기여 commit 참조는 유지한다. [고유 56개 적용표](../assets/planet6897_20261106/contributor-commit-map-r42.tsv)와
[리베이스·충돌 처리 보고서](../assets/planet6897_20261106/rebase-r42-analysis.md)를 따른다.
원 PR head와 입력은 보존됐으며, 이력 갱신을 최종 검증 통과나 승인으로 표시하지 않는다.

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.

## Native 표시 family 변경 시 체크 기호의 원본 폭

R36의 원본 전 14쪽은 Native 최저 96.52%, fresh WASM 최저 98.44%다. 그러나 실제
11쪽 연구 방식·예산 항목의 U+221A `√`를 확대하면 Native의 기호와 뒤 항목이 좁아진다.
같은 쪽 raw는 Native 97.59%, WASM 99.96%이며, 점수로 이 차이를 면제하지 않는다.

선행 Git의 `6c039b2006`·`b277a89737`은 HFT의 고유 전진과 꾸밈을 분리했고,
`6097253468`(PR #8191)은 이 계약을 유지했다. 현재 sparse HFT 표에는 ASCII만 있어
U+221A의 폭이 빠져 있다. Native의 실제 공급 환경이 표시 family를
`한양신명조`에서 `HY Sinmyeongjo`로 바꾸면 원본 표의 미수집 글자를 표시 family에서
다시 찾다가 반각 추정 550 HU로 내려간다. 표시 이름이 유지된 WASM과 환경 없는 진단은
기존 전각 1100 HU를 사용한다. 두 경로의 실제 글꼴 환경과 공개 font decision trace로
원인을 구분했다. 저장 줄·표 원점·문서 자체가 달라진 문제는 아니다.

독립 `pdf/issue8596_policy_committee_agenda-2020.pdf` 11쪽의 T6는 ToUnicode 04를
U+221A, CharProc HFT285에 연결한다. 해당 프로그램의 `d1` 전진은 1000/1000 em이다.
원본의 1100 HU 글자 크기와 합치며 현재 실제 공급 CFF의 hmtx도 같은 값을 준다.
글꼴 윤곽이나 프로그램 파일을 코드·저장소에 복사하지 않는다.

보정은 공통 HFT 메트릭 표에 이 독립적으로 확인한 전진을 추가하는 것이다. 문서별 좌표나
표 위치 보정은 추가하지 않는다. 원본 source 메트릭 → 글줄 측정 → TextRun 전진 → Native
glyph fitting과 뒤 항목의 원점이 같은 값을 소비하게 한다. 기존 ASCII·다른 family·수식
레이아웃의 별도 radical 프로그램은 적용 대상이 아니다. 실제 14쪽 재대조와 최소 글꼴
회귀를 끝내기 전에는 완료·승인을 선언하지 않는다.
