# 기존 #7778 문서의 검토 범위 제외

## 사용자 지정과 기존 이관 범위

2026-10-25 사용자가 [기존 #7778](https://github.com/edwardkim/rhwp/issues/7778)을
지정해 `samples/issue1921/59043_regulatory_analysis.hwp`를 이번 PR의 검토·보정·
시각 검증 범위에서 제외했다. 새 이슈를 만들거나 기존 이슈를 종료하지 않는다.

#7778 본문은 전체 피델리티를 추적하면서 정상 쪽수·8/11/12쪽 그림 소속 4개 검사를
보존한다고 명시한다. 따라서 그 정상 검사와 원본·기존 기준 PDF는 유지한다.
현재 실패한 off-canvas 전수 수집만 공통 축별 이관 원장에 등록하고 해당 baseline
행을 제거한다. 다른 문서의 검사·허용치·렌더러 구현은 변경하지 않는다.

## 관측과 제외의 구분

기준 `upstream/devel`은 `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`다.
기존 원본 SHA-256은 `da9f9c9862475d1710f6ecf93e3ce134726054fe0be0d9523b9d60d1a90c82ef`다.
새 MCP는 한컴 2020 `11.0.0.9136`, PrintMethod0·one-up으로 37쪽을 출력했다.
서비스의 `input_preprocess`는 `scripts_disabled`이며 전처리 없음으로 보고하지 않는다.
PDF SHA-256은 `132745636d61b96179ad97197d0cd24f4e89db2e030682fd3f4f26b7290d94dd`다.

제외 지시 전에 실행한 전37쪽 진단의 최저 raw는 base Native35.37945%, 후보
Native35.38634%, fresh WASM35.33520%다. 후보의14·15쪽에는 base 대비 하락도
있다. 이 결과는 통과나 해결이 아니며 제외를 출력 개선으로 보고하지 않는다.
사용자가 지정한 기존 이관 범위에서 후속 검토할 자료로 보존한다.

새 PDF는 이번 PR의 수용 기준으로 쓰지 않으므로 임시 증적 루트
`output/pr-review/semanticist21-20261019/mcp-offcanvas-3817/reference/`에 보존했다.
기존 커밋 PDF는 이동하지 않았다. MCP job·입력/출력 hash와 실제 이동 전 경로는
같은 디렉터리의 `complete.json`에 있다. 전쪽 TSV와 로그는
`offcanvas-sweep-3818-result.json` 및 `logs/offcanvas-sweep-3818-driver.log`에 있다.
삭제한 코드 사본은 만들지 않으며 제거한 baseline의 이전 내용은 `98890fe33f`에서
확인할 수 있다. 남은 off-canvas 전수 검사 결과는 종료 후 기록한다.


## 제외 후 실제 검증

`logs/issue7778-exclusion-validation-3822.log`에서 정식 nextest **20개 실행,
19개 PASS·1개 FAIL**, 종료 코드100을 확인했다. #1921의 정상 검사4개와 기존 실패가
있던 off-canvas partition9는 통과했다. 전체16개 partition을 실행했으며 실패한
partition6의 원인은 **다른 입력**인
`samples/issue6133/156483831_poster_title_above_offset_float.hwp`의 신규1건이다.
이 입력을 #7778에 포함하거나 baseline을 완화하지 않고 남은 PR 준비에서 판정한다.
전체 통과나 PR 제출 준비 완료로 보고하지 않는다.
