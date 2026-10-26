# PR #8273 리뷰 — duration refresh CLI의 Archive E 입력 보정

## 최종 판정

**승인 — 실제 CLI 검증과 최종 head의 전체 CI를 확인하고 일반 merge했다.**
검토 head는 `fc77cd99add15a9b4aa8383c46433bc608ad3770`, merge는
`61ec35daa499b375b1dc6b66a4f5f6d6cc6ad0aa`다. 사용자 지정 옵션 2의 문서 PR #8274에
원 PR #8188 기록과 함께 보존한다. 병합 후 metadata 실제 발행도 아래와 같이 확인했다.

## 접수와 원인

- [PR #8273](https://github.com/edwardkim/rhwp/pull/8273), 작성자 `jangster77`, base `devel`,
  관련 [#8272](https://github.com/edwardkim/rhwp/issues/8272). 별도 reviewer 미지정, 최종 MERGEABLE/CLEAN 확인 후 MERGED.
- [실패 run 40290628059](https://github.com/edwardkim/rhwp/actions/runs/40290628059/job/121464579118)의
  실제 오류는 CLI main에 남은 B/C/D 전용 label 검사였다. 수집기·공통 함수는 E를 지원했지만
  CLI의 중복 검사가 네 입력을 거부했다. Python 검사가 그 옛 오류 문구를 요구하던 문제도 확인했다.
- 중복 검사를 제거하고 공통 함수의 정확한 B/C/D 또는 B/C/D/E 집합 검사에 위임했다.
  파싱·provenance·잘못된 입력 거부와 미발행 계약을 유지한다.

## 실제 검증

- Node 136개·Python 44개·전체 actionlint·공백·문서 링크 검사가 통과했다.
- 실제 프로세스에 schema 1/2의 세 개·네 개 JSON 파일을 전달했다. 정상 output의 네 archive
  provenance와 E target을 확인하고 누락·중복·알 수 없는 label·개수 오류에서는 실패와 미발행을 확인했다.
- 같은 CI run의 실제 B/C/D/E 실측을 사용한 음성 대조에서 기존 CLI exit 1, 보정 CLI exit 0이었다.
  46 target·1,730 source case·7,578 testcase를 보존했다. 검사 자체가 실제 CLI를 통과하므로
  순수 함수 검사만으로 CLI main을 우회하지 않는다.
- 최종 GitHub [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/40293166605)、[CI](https://github.com/edwardkim/rhwp/actions/runs/40293166557)、[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/40293166575)、[Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/40293166615)가 모두 성공했다. CI Impact Policy도 success다.
- A/B/C/D/E는 3,801/2,397/2,199/460/2,522개 전부 통과했다.
  집계 `archive_total=11379 shard_total=11379`를 실제 로그로 확인했다.
  Native 전용 #8160 이중 테두리 회귀도 실제 PASS였다.
- 기준 `2546c8b2a` 대비 제품 Rust·기존 테스트 fixture·sample·PDF·TTF·#8160 TSV의 Git 차이가 없다.
  CLI와 Node/Python 계약의 변경은 원래 문서 출력이나 원본 경로를 바꾸지 않는다.

## 공통 검토와 범위

| 항목 | 판정과 근거 |
| --- | --- |
| 선행 원인 분석 | 충족 — 실제 실패 job과 중복 CLI 검사·잘못된 Python 기대값 추적 |
| 동작 기반 회귀 | 충족 — 실제 Node CLI의 허용·거부·미발행 및 동일 실측 수정 전 실패/후 성공 |
| 조판 원칙·시각 검증 | 비해당 — metadata CLI만 변경. 제품 source·회귀 fixture·입력/PDF·TSV 동일성을 Git으로 확인 |
| HWP/HWPX/PDF 새 입력 | 비해당 — 이 보정은 CI가 생산한 duration JSON만 읽음. 기존 #8160 자료는 보존 |
| 권한·이벤트 경계 | 충족 — workflow/event/permission 변경 없음. metadata refresh만 자동 관찰 |
| 로컬/CI 구분 | 충족 — 로컬 136/44건과 정확한 최종 head의 GitHub 전체 결과를 구분 |

## 병합 후 metadata

[Metadata run 40298571437](https://github.com/edwardkim/rhwp/actions/runs/40298571437)은 성공했다.
수집기는 PR #8273의 정확한 head·run 40293166557·worker별 attempt 1에 결합한 B/C/D/E 네
실측을 수용했다. CLI가 `refreshed_targets=46`을 출력했고 실제
[metrics commit 222ae0a6](https://github.com/edwardkim/rhwp/commit/222ae0a6a578efb3ddad1fe8ea7186cbc060f713)으로
46 target·1,730 source case·7,578 testcase와 네 archive provenance를 발행했다.
발행된 JSON을 다시 읽어 source head/run·네 label을 확인했다. 병합 후 제품 검증 CI는 시작하지 않았다.

## 후속 comment와 정리

최종 code/merge SHA·실제 CLI 대조·최신 CI·metadata 결과와
[계획 및 검증 기록](../../working/task_m100_8272_duration_refresh_cli_plan.md)을 한국어 comment로 연결한다.
원 PR #8188의 이미지·raw100 증거는 [기존 검토](pr_8188_review.md)를 따른다.
이 문서 PR이 기록 완료 산출물이므로 추가 기록 PR을 만들지 않는다. 전용 branch/worktree와
output은 실제 자료 보존·사용 종료를 확인한 뒤 원 작업과 함께 정리한다.
