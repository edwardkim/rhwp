# PR #9196 리뷰 — 머리말·꼬리말 표 캡션 위치 조회

## 최종 판정

**승인.** 통합 코드 후보에서 머리말과 꼬리말의 표 캡션을 가리키는 클릭 좌표가 해당 표의 캡션 경로를 반환했다. 병합 전 통합 PR의 정확한 게시 head에서 필수 CI, 병합 가능 상태와 작업지시자 승인을 확인한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자·대상 | [#9196](https://github.com/edwardkim/rhwp/pull/9196) · `semanticist21` · devel |
| 원 head | `0fce33627d4efd7d0380720e354a05dcf6ebd297` |
| 체리픽 순서와 적용 | #9193 뒤 적용 · `43f66344347d53cc5967f9e9e1790419f946adc8` |
| 검토 담당 | `jangster77` |
| 통합 코드 후보·devel | `43f66344347d53cc5967f9e9e1790419f946adc8` · `9440d05e7d4738151822d02a8392d6938796e474` |

최신 devel과 개별·누적 병합 시뮬레이션에서 충돌이 없었다. 선행 의존 PR과 메인터너 보정은 없었다. 로컬 실행의 소스는 `a040d84daa9dcd60b762125ea9beb898da60fe72`다. 리베이스에서 패치와 제품·테스트·fixture·빌드 입력이 동일했다.

## 변경과 검토 범위

`hitTestInHeaderFooterTarget`이 렌더 트리에서 클릭한 캡션 줄의 셀 문맥을 읽고, 문단·표·셀 번호로 글자 런을 고른다. 경로 끝의 셀 65534는 해당 표의 캡션이다. 판정 범위는 엔진의 위치 조회다. 새 코드는 기존 렌더 트리를 조회하므로 조판·그리기 및 보호문서 sweep은 비해당이다.

## 검증 입력과 결과

- macOS arm64 최소 글꼴 환경에서 `header_footer_table_paths` 7건과 일반·중첩 표 비교 `table_in_cell_non_inline` 4건이 통과했다. 원 테스트는 빈 문서에 만든 머리말 1×2 표에 아래 캡션을 옮긴 후, 그 캡션 줄 안의 좌표에서 경로의 마지막 셀과 문단을 검사한다.
- 머리말과 꼬리말에 같은 1×2 표와 아래 캡션을 둔 합성 HWPX 두 개를 준비했다. 같은 fresh WASM을 Node와 Chrome에서 불러 `hitTestInHeaderFooterTarget`에 각 캡션의 실제 좌표를 넘겼다. 두 경우 모두 올바른 본문 문단·표 컨트롤·캡션 셀 65534가 반환됐다. 실제 브라우저 검증은 공개 WASM API 호출까지다.
- 전체 Nextest 13,824건 통과, 실패 0건, 건너뜀 45건. 입력 변형은 해당 위치 조회를 검증하기 위해 사용했다.

## 통합 PR #9200의 코드 CI

[통합 PR #9200](https://github.com/edwardkim/rhwp/pull/9200)의 코드 CI head `8a8917958e38a30e556d2f08960268216ce59fee`. 필수 Build & Test가 통과했고 GitHub 검사 34건 성공, 5건 조건부 건너뜀, 실패 0건이다. 첫 Native Skia 잡은 APT 갱신의 180초 제한으로 종료됐고, 같은 head의 [잡 재실행](https://github.com/edwardkim/rhwp/actions/runs/44530849923/attempts/2)이 통과했다. [APT 자동 재시도는 #9203](https://github.com/edwardkim/rhwp/issues/9203)에서 다룬다. 후행 문서 커밋의 필수 검사를 확인한 뒤 통합 PR을 병합한다.

[CI](https://github.com/edwardkim/rhwp/actions/runs/44530849923) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44530849934) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44530849915) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44530849971) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44530850072) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44530849537)

## Merge 후 contributor PR comment 계획

통합 PR과 검증한 최종 head의 CI·병합 SHA를 확인 후 원 PR에 연결한다.

> 머리말과 꼬리말의 표 캡션 위치가 올바른 표를 가리키도록 개선해 주셔서 감사합니다. Native 회귀와 fresh WASM 공개 API에서 캡션 좌표가 두 경우 모두 해당 표의 셀 65534 경로를 반환함을 확인했습니다.
