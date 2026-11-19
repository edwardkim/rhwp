# PR #9187 리뷰 — fix(field): 각주 지우기·쪽 감추기가 뒤 누름틀의 컨트롤 번호를 함께 옮긴다

## 최종 판정

**통합 승인.** [통합 PR #9195](https://github.com/edwardkim/rhwp/pull/9195)의 코드 head `bd20cae04547db2053a541d259748eff6dc898e0`에서 필수 CI와 개별 주장 검증을 통과했다.

변경 요청에서 확인한 활성 주소 문제는 [활성 누름틀 후속 보정](#활성-누름틀-후속-보정)에 기록했다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#9187](https://github.com/edwardkim/rhwp/pull/9187) · `semanticist21` |
| 원 head | `42d14f0f930a64996c1cc472d8c7b1dfff9e42d4` |
| 적용 커밋 | `7ddcbe2ec6b9` |
| 시각 검증 base·측정 head | `af53659e19f70c18d0d3591a719c238235007a05` · `a84c52f4dcade0a068050226851fe61ba550a813` |
| 원 PR 검토 담당 | `jangster77` |

## 변경과 검토 범위

앞 각주 제거와 쪽 감추기 삽입·해제 뒤 누름틀의 컨트롤 참조, 이름과 안내문을 확인했다.

## 검증 입력과 결과

검증한 통합 head는 `a84c52f4dcade0a068050226851fe61ba550a813`이며 고정 devel은 `af53659e19f70c18d0d3591a719c238235007a05`다. 통합 최소 글꼴 전체 nextest는 13,816건 통과, 실패 0건, 건너뜀 45건이다.

Rust 누름틀 관계 회귀 2건과 fresh WASM HWP/HWPX 네 교차 저장 순서를 두 편집 경계에서 각각 확인했다.

## 시각 증적과 검증 기록

[검증 소스와 리베이스 확인](../assets/open9_integration_20261119/validation.json) · [원 PR 커밋 대응표](../assets/open9_integration_20261119/source-commits.json)

[주장 범위 회귀 실행](../assets/open9_integration_20261119/9187/regression.json)

[각 HWP/HWPX 반복 저장의 확인값](../assets/open9_integration_20261119/9187/repeated-wasm-save.json)

## 통합 CI

검증 head `bd20cae04547db2053a541d259748eff6dc898e0`. [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44509238552) · [CI](https://github.com/edwardkim/rhwp/actions/runs/44509238545) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44509238349) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44509238197) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44509238311) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44509238321)

## Merge 후 contributor PR comment 계획

원 PR의 head와 통합 PR #9195의 병합 SHA·코드 CI를 확인하고 아래 문안을 사용한다. 병합 SHA의 asset 바이트를 대조한 뒤 `--body-file`로 게시하고 API의 본문과 렌더링된 이미지를 확인한다.

> 누름틀의 컨트롤 번호가 앞 각주와 쪽 감추기 편집을 따라가도록 수정해 주셔서 감사합니다. 앞 각주 삭제와 쪽 감추기 전환 후 HWP/HWPX를 교차 저장하고 다시 열어 누름틀 이름·안내문·후속 본문을 확인했습니다.

[해당 기능의 회귀 실행](../assets/open9_integration_20261119/9187/regression.json)과 위 편집·저장 검증을 코멘트에 연결한다.

## 활성 누름틀 후속 보정

**메인터너 보정 후 수용 가능 — 쪽 감추기 전환 뒤 누름틀 끝 입력의 로컬 검증 완료.** 아래 결과는 #9195 병합 이후의 후속 통합 후보에 해당한다. 병합 전 후속 통합 PR의 정확한 게시 head에서 필수 CI, 병합 가능 상태와 작업지시자 승인을 확인한다.

[postmelee의 변경 요청](https://github.com/edwardkim/rhwp/pull/9187#pullrequestreview-5970239515)은 쪽 감추기 전환 뒤 현재 활성 누름틀의 주소도 옮기도록 요구했다. 원 기여는 목록과 저장에 쓰는 `field_ranges[].control_idx`를 옮겼다. `active_field`에는 이전 루트 컨트롤 번호가 남아 누름틀 끝의 다음 입력이 본문으로 분류됐다.

- 최신 devel 위의 보정 커밋은 `39de79630b`다. 쪽 감추기 컨트롤을 넣거나 뺄 때 기존 주소 보정 함수를 호출한다. 활성 위치 보정은 동일한 구역·문단에 적용되며 기존 컨트롤의 플래그 갱신에는 주소 이동이 없다. 두 후속 원 PR을 적용하기 전에 보정했고 충돌은 없었다.
- 기준 `eb9311d9d67ae2ae2b45e28c5a911d9f0e16127d`에서 회귀를 먼저 실행했다. 쪽 감추기를 켜거나 끈 뒤 누름틀을 다시 활성화하지 않고 끝에 `C`를 입력하자 신규 2건에서 값이 `AB`에 머물렀다. 기존 목록·저장 검사 2건은 통과했다.
- 보정 후 최소 글꼴 Native의 `control_edit_keeps_field_ranges` 4건이 통과했다. 켜기·끄기의 누름틀 값은 `ABC`였고 앞뒤 문자는 HWP와 HWPX 재열기에서도 보존됐다.
- Node와 Chrome의 fresh WASM에서 실제 공개 API로 쪽 번호 감추기 켜기·끄기를 실행했다. HWP→HWPX 및 HWPX→HWP로 재저장 후 열어도 누름틀 값과 전환한 상태가 유지됐다. 전체 Nextest는 13,824건 통과, 실패 0건, 건너뜀 45건이다.
- 로컬 실행 소스 `a040d84daa9dcd60b762125ea9beb898da60fe72`, 통합 코드 후보 `43f66344347d53cc5967f9e9e1790419f946adc8`, devel `9440d05e7d4738151822d02a8392d6938796e474`. 리베이스의 패치와 제품·테스트·fixture·빌드 입력이 동일했다. 변경은 편집 중 활성 위치의 소유관계에 해당하고 조판·그리기 및 보호문서 sweep은 비해당이다.

[통합 PR #9200](https://github.com/edwardkim/rhwp/pull/9200)의 코드 CI head `8a8917958e38a30e556d2f08960268216ce59fee`. 필수 Build & Test가 통과했고 GitHub 검사 34건 성공, 5건 조건부 건너뜀, 실패 0건이다. 첫 Native Skia 잡은 APT 갱신의 180초 제한으로 종료됐고, 같은 head의 [잡 재실행](https://github.com/edwardkim/rhwp/actions/runs/44530849923/attempts/2)이 통과했다. [APT 자동 재시도는 #9203](https://github.com/edwardkim/rhwp/issues/9203)에서 다룬다. 후행 문서 커밋의 필수 검사를 확인한 뒤 통합 PR을 병합한다. [CI](https://github.com/edwardkim/rhwp/actions/runs/44530849923) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44530849934) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44530849915) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44530849971) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44530850072) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44530849537)

병합 후 기존 #9195 기록과 후속 통합 PR을 함께 연결하고, 변경 요청에 대응한 두 수정 전 실패·수정 후 통과와 교차 저장 결과를 남긴다.
