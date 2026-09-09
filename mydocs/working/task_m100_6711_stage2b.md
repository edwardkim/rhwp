---
kind: snapshot
status: active
canonical: mydocs/plans/task_m100_6711.md
issue: 6711
last_verified: 2026-09-04
---

# #6711 Stage 2-B — pr/report 월별 archive 이동

## 1. 기준선과 결과

PR #6713 merge 뒤 최신 `upstream/devel@009e30fe1f6812b046862589783c68f890b4d363`에서
continuation branch를 만들고 `2026-09-01T00:00:00+09:00` 이전에 Git에 처음 도입된 직접 하위
Markdown만 처리했다.

| 폴더 | 이동 전 root | cutoff 이전 후보 | 9월 유지 | 이동 후 root |
| --- | ---: | ---: | ---: | ---: |
| `pr` | 120 | 119 | 1 | 1 |
| `report` | 722 | 713 | 9 | 9 |
| 합계 | 842 | 832 | 10 | 10 |

처리 결과는 다음과 같다.

| 구분 | 수 |
| --- | ---: |
| 일반 archive 이동 | 828 |
| 서로 다른 충돌을 suffix 경로로 보존 | 2 |
| 동일 archive가 있어 root만 제거 | 2 |
| Git rename 합계 | 830 |
| root에 남은 cutoff 이전 문서 | 0 |

## 2. 충돌 원장

byte-identical 중복은 두 경로의 SHA-256이 같은 것을 다시 확인한 뒤 기존 archive를 유지하고 root만
제거했다.

| 제거한 root | 유지한 archive | SHA-256 |
| --- | --- | --- |
| `mydocs/pr/pr_2331_maintainer_review.md` | `mydocs/pr/archives/pr_2331_maintainer_review.md` | `085ea428cd97dc3646367e7df33e12ea3e5a21b3c7c6e673aab05337effcb8a2` |
| `mydocs/report/task_m100_1363_report.md` | `mydocs/report/archives/task_m100_1363_report.md` | `d2ed6bfa7e3ef29628ef0acb4599d352be578a3bf6870abfb40ea73a7d3fc2c9` |

서로 다른 기존 archive는 덮어쓰지 않고 root의 최초 Git 도입일과 content hash를 suffix에 사용했다.

| root | root SHA-256 | 기존 archive SHA-256 | 새 목적지 |
| --- | --- | --- | --- |
| `mydocs/pr/pr_1844_review.md` | `a72eb2d81a1239587e4940c03fa1e53614f96ceaaa00368cec7d64a3e9bddfde` | `0cf214213f10284b9514ed53694750a2d1cb5d4b0430ebdf425b0c234a102fe3` | `mydocs/pr/archives/pr_1844_review_archived_20260703_a72eb2d.md` |
| `mydocs/pr/pr_2370_review.md` | `100f4495f7e3cbb4fb72eee37480b71350beee1cc807b2909cfa5885b7cd4da3` | `da7c320294c3a0fb4660ccb04826d60cef6dfb69560e73e03106c6cf516766d2` | `mydocs/pr/archives/pr_2370_review_archived_20260725_100f449.md` |

## 3. 링크와 canonical 보존

이동 전에 832개 source·목적지·충돌을 모두 메모리에서 계산했다. 기존 source 위치에서 link target을
해석한 뒤 source와 target의 새 위치를 함께 반영했으며, 이동 대상이나 그 incoming link가 아닌
경로 표현은 바꾸지 않았다.

- Markdown link destination 재계산: 392개
- `canonical:` 이동 경로 갱신: 41개
- content가 바뀐 문서: 182개
- rename 이외의 수정 문서: 71개
- `mydocs` 밖 incoming link 수정: 4개
  - `pdf/issue5447/README.md`
  - `pdf/issue5652/README.md`
  - `samples/issue5447/README.md`
  - `samples/issue5652/README.md`

변환 초안에서 이동과 무관한 디렉터리 링크의 끝 `/`까지 정규화되는 과잉 변경을 표본 검사로
발견했다. 변환 조건을 source 이동 또는 target 이동으로 한정하고 원문에서 디렉터리를 나타내던 끝
`/`를 보존한 뒤 전체 변환을 기준선 원문에서 다시 계산했다.

## 4. 오류 집합 비교

`upstream/devel`의 Markdown tree를 checkout 없이 읽어 현재 tree와 논리 경로 기준으로 전수
비교했다. 두 동일본 root는 기존 archive가 같은 bytes를 이미 보유하므로 중복 source만 제외했다.

| 검사 | 이동 전 | 이동 후 | 신규 |
| --- | ---: | ---: | ---: |
| 추적 Markdown 문서 | 13,170 | 13,168 | -2 동일본 |
| 유효 내부 링크 | 6,623 | 6,623 | 손실 0 |
| historical broken link | 2,544 | 2,544 | 0 |

정규화된 broken-link multiset은 이동 전후 완전히 같으며, 새로 깨지거나 우연히 사라진 항목도 0개다.
canonical 기본 링크 검사는 609개 문서에서 오류 0개다. metadata 검사는 `mydocs/tech`의 기존 오류
16개만 재현했으며 이번 배치의 신규 오류는 없다.

`--changed-from upstream/devel --forbid-redirect-references` 검사는 이동한 historical 문서까지 읽어
기존 오류 5건을 표시한다. #1456의 잘못된 Studio 상대경로 2건과 `p[0](...)`, `record[0](...)`,
`array[3](...)` 형태의 설명 문장을 링크로 해석한 3건이다. 다섯 target은 기준선에서도 존재하지
않았으며 정규화 비교에서 같은 source·line·target으로 유지된다. #3684 보고서의
`canonical: mydocs/plans/task_m100_3684.md`도 기준선부터 대응 계획서가 없던 1건이다. 이번 이동에서
임의 정본을 만들지 않았고 canonical missing 집합의 신규 항목은 0개다.

## 5. PR 크기와 범위

보고서와 오늘 기록을 넣기 전 Git 실측은 rename 830개, 수정 71개, 삭제 2개로 903개다. GitHub가
rename을 전혀 인식하지 않는 보수적 계산은 `830 × 2 + 71 + 2 = 1,733`개다. 이 보고서 추가와
오늘 기록 수정 뒤에도 1,735개로 PR file API 3,000개 한도보다 충분히 작다.

Rust source·test, Cargo, WASM, workflow는 변경하지 않았다. `mydocs` 밖 네 README 변경은 모두
이동된 보고서로 들어오는 링크의 목적지만 갱신한 것이다.

## 6. 다음 절차

1. 보고서를 포함한 변경 문서 링크·metadata·diff 검사를 다시 실행한다.
2. 최종 rename-aware·보수적 변경 경로 수와 cutoff 이전 root 잔여 0건을 재확인한다.
3. Stage 2-B 결과 승인 뒤 commit을 만든다.
4. 별도 승인 뒤 원격 push와 PR 생성을 수행한다.
5. PR merge와 최신 `devel` 동기화 뒤 Stage 3 `working` batch를 시작한다.
