# PR #2451 리뷰 - 중첩 HWPX field parameters XML 균형 보존

## 메타데이터

| 항목 | 값 |
|---|---|
| 원 PR | [#2451](https://github.com/edwardkim/rhwp/pull/2451) |
| 작성자 | `kevin9327` |
| base | `devel` |
| 리뷰 경로 | collaborator-mediated 외부 PR, G3 체리픽 통합 |
| 적용 커밋 | `47dc2ee9675b91146f97ed6614514ab805be15d4` |
| 적용 순서 | G3 3/3, 최신 `upstream/devel` 위 체리픽, 충돌 없음 |
| 작성 시점 참고값 | 원 PR은 `BEHIND`, 기존 CI 통과. 통합 PR의 최신 head CI를 merge 전 다시 확인한다. |

## 변경 검토

`parse_field_parameters`는 무손실 저장을 위해 `hp:parameters` 자식을 원문 형태로 다시
구성한다. 기존 단일 `open_param` 상태는 중첩 `listParam` 안의 `stringParam`을 만나면
안쪽 태그로 덮여 바깥 닫는 태그를 잃었다.

End 이벤트가 제공하는 자신의 qualified tag 이름으로 닫도록 바꿔 임의 깊이의 중첩에서도
균형 잡힌 XML을 구성한다. 비중첩 parameter 출력은 변하지 않는다.

## 검증

| 게이트 | 결과 |
|---|---|
| `parse_field_parameters_reassembles_nested_params_balanced` | PASS |
| `cargo fmt --check` | PASS |
| `git diff --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `cargo test --profile release-test --tests` | PASS |

## 시각 검증 판단

중첩 field parameter의 raw XML 저장 경로만 변경한다. renderer와 레이아웃을 수정하지 않으므로
별도 visual sweep 또는 MCP PDF 검증 대상이 아니다.

## 최종 의견

G3 통합 PR에 원 커밋을 보존해 수용한다. merge 전 통합 PR 최신 head의 GitHub Actions와
mergeable 상태를 재확인한다.
