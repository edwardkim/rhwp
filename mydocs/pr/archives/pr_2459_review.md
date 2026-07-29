# PR #2459 리뷰 - HWPX 3D 테두리 선 종류 파싱 보존

## 메타데이터

| 항목 | 값 |
|---|---|
| 원 PR | [#2459](https://github.com/edwardkim/rhwp/pull/2459) |
| 작성자 | `kevin9327` |
| base | `devel` |
| 리뷰 경로 | collaborator-mediated 외부 PR, G5 체리픽 통합 |
| 원 커밋 / 통합 적용 커밋 | `71c4c5086706acee5c64e264af94cfbfdf2ee809` / `39ef2206f` |
| 적용 순서 | G5 4/8, #2457 뒤 체리픽, 충돌 없음 |
| 작성 시점 참고값 | 원 PR은 `MERGEABLE` / `BEHIND`, 원 head CI는 성공이다. 통합 PR의 최신 head CI와 mergeable 상태를 merge 전 다시 확인한다. |

## 변경 검토

`THREE_D`, `THREE_D_REV`, `THREE_D_LIGHT`가 border line parser에서 기본값으로 떨어져 HWPX
재로드 후 선 종류 정보가 사라졌다. PR은 세 값을 기존 `BorderLineType` 표현으로 매핑하고, 3D 계열을
한 번에 확인하는 회귀를 추가한다.

## 검증

| 게이트 | 결과 |
|---|---|
| `parse_border_line_type_accepts_3d_styles` | PASS |
| G5 공통 full regression / fmt / clippy / diff / WASM | PASS |

## 시각 검증 판단

테두리 model 파싱 보존만 바꾸며 paint/layout 코드는 변경하지 않는다. 실제 3D 선의 최종 표현 fidelity를
주장하는 PR도 아니므로 구조 회귀로 충분하며 visual sweep은 적용하지 않는다.

## 최종 의견

G5 통합 PR에 기여자 원 커밋을 보존해 수용한다. 공통 검증과 최종 조건은
[G5 통합 실행 계획](pr_2455_review_impl.md)을 따른다.
