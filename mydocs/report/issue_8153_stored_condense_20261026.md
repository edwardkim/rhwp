# #8153 저장 줄의 공백 압축 구현과 검증 기록

브랜치: `task/8153-handbook-raw100`. [구현 계획](../plans/issue_8153_stored_condense_20261026.md)의
공통 원인과 독립 Print 근거를 따른다. 두 원본과 기존 기준 PDF를 그대로 사용한다.

## 구현

비분배 줄이 원본 공백 최소값의 예산으로 수용되면 내부 ASCII 공백에만 압축을 배정한다.
`composer::condense_space_savings_px`는 기존 HU 단위 줄 수용 예산을 재사용한다.
`CondensePlan`은 선두 공백·말미 구분 공백·nbSpace·원자적 필드를 제외하고 run별 원본
공백 폭에 비례해 같은 예산을 소비한다. 공백 최소값이 없거나 예산이 부족한 줄과
Justify/Split의 기존 계약은 이 새 배정의 적용 대상이 아니다.

공통 결과는 `TextStyle.space_compression`으로 전달한다. 문자 폭의 최종 결정은 원 글자·
자간 측정 뒤에 해당 ASCII 공백의 전진만 줄인다. exact/unrounded 폭과 glyph positions가
같은 결정을 소비한다. 인라인 경계의 `for_text_fragment`도 원문 scalar 범위를 이어받는다.
실제 run의 positions를 게시하므로 Native/Canvas가 별도 글꼴 측정으로 압축을 잃지 않는다.
정렬 기준 폭은 같은 계획의 실제 소비량을 사용한다. 문서 IR·글꼴·저장 줄 소속은 바꾸지 않는다.

## 현재 검증 상태

- `rustfmt`와 `git diff --check`를 확인했다. 새 구현은 아직 컴파일·출력 검증 전이다.
- 기존 producer `5bcb654516968c64d5c3910aeb556fe5e3eebe72`에서 두 원본의
  긴 줄 `관` 전진은 10.759259259259252px, 같은 글자모양의 짧은 줄은 11.04px였다.
  이 결함을 관계형 검사로 재현했다. 선두 run 분할을 허용하도록 검사 선택자를 정리했으므로
  최종 테스트 원문으로 수정 전 FAIL을 다시 확인한 뒤 수정 후 PASS와 연결한다.
- 기존 fresh WASM 표준 최적화가 사용하는 고정 checkout과 공유 target을 보존한다.
  최적화 종료·산출물 동결 뒤에만 새 Native 컴파일을 시작한다.
- 두 원본 200쪽과 정상 대조군의 직접 출력·기존 #7418/#7702 검사, 전쪽 TSV 및 최종
  필수 검증은 계속 수행한다. 현재 기록은 시각 개선 완료나 PR 준비 완료 판정이 아니다.

실행 로그·중간 JSON·이미지는 ignored
`output/pr-review/issue8153-handbook-raw100-20261022/`에 기록한다.
`upstream/devel` 동기화·원격 게시·PR 생성은 하지 않는다.
