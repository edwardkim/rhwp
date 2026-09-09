# PR #6949/#6952 누적 체리픽 검토

## 최신 결론

- [#6949 review](pr_6949_review.md): **머지 보류**. 완성된 ROP 시퀀스/같은 목적 영역을 확인하기 전 DPA 삭제(P1), 1bpp만으로 마스크 판정(P2)을 정적 검토에서 발견했다. 원 PR 제공 3/7쪽 PNG는 열었지만 통합 candidate의 직접 visual sweep은 미실행이다.
- [#6952 review](pr_6952_review.md): **머지 보류**. 기존 #6940과의 직렬화 충돌을 보존 병합했지만 보정 후 build/회귀/Clippy/실물 왕복 검증은 미실행이다. 추가 6개 시험은 합성 IR 직렬화 범위이며 parser/미주/인라인 USER_CHAR 왕복의 직접 검증을 대체하지 않는다.
- 현재는 로컬 누적 체리픽과 정적 PR 검토 단계다. 원격 통합 PR 생성/push/merge/원 PR close/공개 review comment는 수행하지 않았다. 원 PR 두 건의 reviewer 지정만 완료했다.

## 기준과 적용 순서

2026-09-09 시작 작업 트리는 clean한 devel이었다. upstream/devel을 `92f6242af96f51ede912588fa6fe35f5447709bc`까지 fast-forward한 다음 `review/planet6897-6949-6952-20260909`를 만들었다. 열린 planet6897 PR은 두 건이었고 모두 원 head CI가 성공했다. 이미 수용한 #6938/#6940을 다시 적용하지 않았다.

| PR | 원 commit | 로컬 적용 commit | 결과 |
| --- | --- | --- | --- |
| #6949 | `8fbfd865b1190fdf95b5a98d80467a53c944ce59` | `32554ff8bf193e4da64e7855854dcb3b80bffa64` | 출처 보존 체리픽, 충돌 없음 |
| #6952 | `4529c2a0c0104a2783b44a5b798a7badce628042` | `d01f7989b25e5ecff7a6a42a04b1d901ea44c1d6` | 출처 보존 체리픽, serializer 충돌 2개 구간 해소 |

#6952의 ON_* 토큰은 최신 devel에 이미 있어서 유지했다. suffix fallback은 원본의 명시적 빈 값을 보존하는 `deco_chars_from_source`와 새 USER_CHAR 조건을 OR로 결합했다. 미설정 숫자 형식의 `)`와 실제 지정 문자를 유지한다. 원 PR 쪽 파일로 덮어써 #6940 계약을 잃는 해소는 하지 않았다.

## 검증과 산출물 정책

- 원 PR별 실제 CI run과 실행/skip은 개별 review에 기록했다. 이는 원 head의 성공이며 새 누적 candidate의 검증이 아니다.
- 이번 단계는 소스/시험/본문/코멘트 정적 검토 및 기존 contributor PNG 열람이다. 로컬 테스트, Clippy, build, 새로운 PDF/SVG 출력, visual sweep을 실행하지 않았다.
- 기존 #6949 report PNG 4개는 contributor commit의 원래 증적이다. 이를 maintainer 직접 검증 PNG로 분류하거나 중복 복제하지 않았다.
- 추가 PDF, PNG, 로그, generated suite/manifest를 만들거나 stage하지 않았다. 다음 검증에서도 기준 PDF가 적합하면 재사용하고 로그/중간 산출물은 output에만 둔다.
- #6949 소스 결함의 회귀 재현/보정과 #6952 충돌 해소본 검증은 다음 단계이며 완료 사실로 쓰지 않는다.

## 후속 단계와 승인 경계

1. #6949의 불완전/다른 영역/유색 1bpp 입력에서 삭제하지 않는 계약을 고정하고 제한된 메인터너 보정을 설계한다. 이번 검토에서는 발견한 코드 결함을 임의로 고치지 않았다.
2. 검증 승인 뒤 고정 target/pr-review에서 순차로 build/회귀/Clippy 묶음을 수행한다. #6952는 #6940/#2742와 인라인 USER_CHAR/미주/속성 파싱을 포함한다.
3. #6949는 원 문서 3/7쪽과 issue6469 최소 fixture, #6952는 기존 두 원본의 XML 및 한컴 기준 PDF를 직접 대조한다. 원 PR 증적만으로 승인하지 않는다.
4. 실제 수치/대표 PNG/미해결 범위 및 개별 contributor comment 계획을 보완한 뒤 통합 PR 생성 승인을 받는다.
5. 최종 통합 head CI와 merge 승인 뒤에만 post_merge.md의 archive/devel sync/issue/원 PR comment/cleanup을 수행한다. 이 문서는 원격 조치 승인을 대신하지 않는다.
