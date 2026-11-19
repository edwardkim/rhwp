# PR #9193 리뷰 — 번호 정의를 제자리에서 고치는 API

## 최종 판정

**승인.** 통합 코드 후보의 로컬 검증에서 원 PR의 번호 정의 편집·저장 계약을 확인했다. 병합 전 통합 PR의 정확한 게시 head에서 필수 CI, 병합 가능 상태와 작업지시자 승인을 확인한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자·대상 | [#9193](https://github.com/edwardkim/rhwp/pull/9193) · `semanticist21` · devel |
| 원 head | `a2d940a38cbf2334ba276b1736b9e5c70296ab65` |
| 체리픽 순서와 적용 | 기능 `688225ad80e0` → `501dbceb48`, 저장 회귀 `a2d940a38cbf` → `9d8a863654` |
| 검토 담당 | `jangster77` |
| 통합 코드 후보·devel | `43f66344347d53cc5967f9e9e1790419f946adc8` · `9440d05e7d4738151822d02a8392d6938796e474` |

각 커밋은 충돌 없이 적용했다. 선행 의존 PR과 메인터너 보정은 없었다. 로컬 실행의 소스는 `a040d84daa9dcd60b762125ea9beb898da60fe72`다. 리베이스 유입분은 #9198 CI와 #9199 문서이고, 통합 패치 네 개와 제품 소스·테스트·fixture·빌드 입력이 동일했다.

## 변경과 검토 범위

`set_numbering_native`와 WASM `setNumbering`은 ID 0으로 새 정의를 만들고 기존 ID로 해당 정의의 값을 바꾼다. 기존 정의를 참조하는 문단이나 구역은 같은 ID로 새 번호 형식을 쓴다. 명시적 호출 시 참조 중인 번호 머리를 다시 계산하고, 저장은 모델의 수준별 값을 기록한다. 기존 문서의 열기와 그리기에서 사용하는 조판 규칙은 그대로이므로 보호문서 sweep은 비해당이다.

## 검증 입력과 결과

- macOS arm64 최소 글꼴 환경의 `numbering_define` 5건이 통과했다. API로 만든 개요 문단을 사용해 10수준 생성, 7수준의 빈 수준 보완, 글자 모양, 잘못된 입력의 무변경, HWP·HWPX 재저장 뒤 개수·참조·출력 머리를 확인했다.
- 잠금 wrapper의 Mac `--no-opt` fresh WASM을 Node와 Chrome에서 실행했다. 정의 ID 1·정의 1개를 유지하며 `제1장 → 1편 → 제1장`을 세 차례 적용했고 2수준의 `1절`도 확인했다. 각 편집에서 HWP→HWPX와 HWPX→HWP 저장·열기 순서가 통과했다.
- 전체 Nextest 13,824건 통과, 실패 0건, 건너뜀 45건. Chrome이 읽은 WASM SHA-256은 `3db65de8e86dce77f5a12e98ac9bac3e1286c534aee1a2d44fbba91a4fcfe1eb`이다.
- 기여자의 일반 열기 비교에서는 18,270쪽 SVG가 동일했다. 기여자 실행에서 비교 대상 한 문서는 제한 시간에 도달했다. 이 수치는 원 PR의 보고값이다. [원 PR 검증 내역](https://github.com/edwardkim/rhwp/pull/9193)

## 통합 PR #9200의 코드 CI

[통합 PR #9200](https://github.com/edwardkim/rhwp/pull/9200)의 코드 CI head `8a8917958e38a30e556d2f08960268216ce59fee`. 필수 Build & Test가 통과했고 GitHub 검사 34건 성공, 5건 조건부 건너뜀, 실패 0건이다. 첫 Native Skia 잡은 APT 갱신의 180초 제한으로 종료됐고, 같은 head의 [잡 재실행](https://github.com/edwardkim/rhwp/actions/runs/44530849923/attempts/2)이 통과했다. [APT 자동 재시도는 #9203](https://github.com/edwardkim/rhwp/issues/9203)에서 다룬다. 후행 문서 커밋의 필수 검사를 확인한 뒤 통합 PR을 병합한다.

[CI](https://github.com/edwardkim/rhwp/actions/runs/44530849923) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44530849934) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44530849915) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44530849971) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44530850072) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44530849537)

## Merge 후 contributor PR comment 계획

통합 PR과 검증한 최종 head의 CI·병합 SHA를 확인 후 원 PR에 연결한다.

> 번호 정의를 같은 ID로 편집하고 HWP와 HWPX에 보존하도록 개선해 주셔서 감사합니다. Native 회귀와 fresh WASM의 두 교차 저장 순서에서 번호 정의 수, 문단 참조와 개요 머리 표시를 확인했습니다.
