# PR #3457 검토 기록 — AIFT 병합표 CLI 편집 사례

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3457](https://github.com/edwardkim/rhwp/pull/3457) — 정부지원사업 사업계획서 병합표 CLI 편집 사례 |
| 작성자·검토자 | `@kevin9327` · `@jangster77` |
| source head / 통합 commit | `6bded579e614e2722d1a9214bcbcee90fccb3292` / `1050903f8cfc619ff5ea5dfa67b5b3724a1647db` |
| 범위 | 문서·PNG 2개만 추가, Rust·renderer·fixture 변경 없음 |

`samples/aift.hwp`의 74쪽·90표·35×27 병합표를 대상으로, 식별 가능한 실제 제출값이 아닌 가상값 32개를
`edit set-cell`로 채우는 절차를 기록한다. 문서는 빈 셀과 병합 span을 먼저 확인하고 넓은 셀만 고르는
제출 전 안전 절차, `export-tables` 재독, 모호한 서명란은 임의 편집하지 않는 범위를 명시한다.

원본 branch의 devel merge commit은 적용하지 않고 contributor 문서 commit만 `-x` 체리픽했다. 이 통합
후보의 #3482 `overflow` 경고와 #3541 HWPX 산출 형식 보존을 포함한 CLI 계약은 별도 검증한다. 이 PR의
이미지는 사례 설명용이며 renderer 전후 개선 증적이라고 판정하지 않는다.

## 검증과 판정

- source head의 GitHub Actions 성공 상태를 intake에서 확인했다. 오래된 base이므로 원 PR 직접 merge는
  하지 않는다.
- `pdf/aift-2022.pdf`와 `samples/aift.hwp`가 모두 74쪽임을 확인했다. renderer가 바뀐 통합 후보의
  동등 fixture 시각 대조는 #3540 기록에 별도로 보존한다.
- 문서만의 독립 코드 변경은 없으며, 통합 후보의 focused CLI 회귀와 전체 validation gate를 적용한다.

가상 데이터·제출하지 않음·모호 필드 미편집이 문서에 명확하므로 **기술적 수용 가능**이다. 외부 제출의
최종 시각 수용과 실제 서식 내용 책임은 owner가 가진다.
