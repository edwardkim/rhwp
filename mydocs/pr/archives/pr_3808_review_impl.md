---
kind: review_plan
status: local-validation-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-04
---

# kevin9327 열린 PR 9건 누적 검토 기록

기준은 `upstream/devel`의 `f5a458e9c437c0f3d374ca3159a573290f19967c`다.
외부 기여자 PR 검토 절차에 따라 기본 작업트리에서
`review/kevin9327-20260804`를 만들고, Git Graph에서 `devel` 위 적용 상태가 보이도록
원본 head를 누적 체리픽했다. 원 PR 브랜치에는 push하지 않았다.

| 순서 | PR | 원 head | 검토 브랜치 commit | 충돌 |
| --- | --- | --- | --- | --- |
| 1 | #3808 | `e21240f08` | `24ea3d9f4`~`23860e951` | 없음 |
| 2 | #3903 | `0972b386e9` | `e9b6e997d`~`0418d579b` | 없음 |
| 3 | #3899 | `2162212987` | `5708e2e3c` | 없음 |
| 4 | #3897 | `0c057605f1` | `48cbd0458` | 없음 |
| 5 | #3887 | `c5cfba6cda` | `b49b626c1` | 없음 |
| 6 | #3886 | `de6aeb895e` | `31d7600aa`~`9e650d0c9` | 없음 |
| 7 | #3889 | `a8f9b2cb29` | `44546e472` | 없음 |
| 8 | #3898 | `4c99429397` | `0b1bad548` | 없음 |
| 9 | #3908 | `215fca1bb8` | `9d707d56f` | 없음 |

## 공통 검증

- `git diff --check upstream/devel...HEAD`: 통과.
- `cargo fmt --check`: 통과.
- `cargo clippy --all-targets -- -D warnings`: 통과.
- focused contract 96건: 통과.
- `cargo test --profile release-test --tests`: 통과. 장기 회귀
  `overflow_cell_baseline`은 294.20초, `security_corpus_regression`은 189.37초를 포함해
  마지막 `visual_roundtrip_baseline`까지 실패 없이 완료했다.
- 변경 Markdown 링크 검사: 498개 문서, 내부 상대 링크 이상 없음.
- 전체 문서 메타데이터 검사는 `upstream/devel`에도 있는 두 기존 문서의 front matter
  오류 3건으로 실패했다. 이번 9개 PR이 추가·수정한 문서는 원인이 아니며, 신규 문서의
  front matter와 canonical 경로는 개별 확인했다.

## 누적 후보 재작업 사항

1. #3887의 `tools/agent_preflight.py --bin <release-test/rhwp>`가 실패한다. `gen-pua`,
   `gen-table`, `measure-width`, `test-caption`, `test-field`, `test-shape`가 미지 플래그를
   exit 0으로 무시한다. 새 검사가 문제를 발견했지만 후보 자체가 green이 아니므로
   명령별 거부 계약 또는 명시적 제외 부류와 테스트가 필요하다.
2. #3889는 active 가이드에 현행 수치를 고정했지만, 누적 후보의 실제 자기서술은
   CLI 64개, JSON 계약 34개, `recordFields` 159개, MCP 선언 43개, `tools/list` 55개다.
   문서의 61·31·148·39·51과 다르며, #3903가 보완한 출처 표지 누락 6종도 현행 예외로
   적혀 있다.
3. #3908은 자체 운영 규칙과 달리 아직 열린 #3808과 #3898을 `[완료]`로 표시한다.
   활성 canonical 로드맵의 완료 표기는 merge 상태와 함께 갱신돼야 한다.

모든 원 PR은 검토 시점에 `BEHIND`였다. 이는 원격 병합 조정 상태일 뿐 로컬 검증의
유효성을 취소하지 않는다. 수용 판단은 위에서 완료한 로컬 검증으로 고정한다. #3887,
#3889, #3908만 명시된 실제 결함을 보정해 다시 로컬 검증한다.
