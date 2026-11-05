# CI checkout·검증 시간 예산 보정안

로컬 검증 시각: `2026-11-04T07:34:41+09:00`. 적용 전 head는 `542474f36c436d30959c30b71fd13f1402e2790e`, 확인한 base는 `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4`다. 작업지시자가 시간 예산 보정·commit·push를 승인했다. 이 보정의 새 head와 최신 devel 통합 뒤에도 모든 필수 검증 성공이 필요하다. 아래 실측과 252건 검사는 승인 전 로컬 보정의 근거다.

## 실측 원인

기존 job timeout은 checkout·설치·계약 검사·컴파일·실제 테스트·업로드를 합한 전체 시간이다.

| 실제 job | checkout | 이후 확인한 결과 | 종료 |
| --- | ---: | --- | --- |
| Lint attempt 3 | 8분 36초 | Native Clippy 2분 37초·WASM Clippy 2분 28초 성공, workspace 3분 21초 진행 | 전체 20분 상한 취소, 컴파일 진단 없음 |
| Proptest attempt 2 | 17분 58초 | toolchain·cache·runner 계약·suite 준비 성공, property 명령(컴파일 포함) 1분 53초 진행 | 전체 20분 상한 취소, assertion 실패 없음 |
| Frontend attempt 2 | 23분 49초 | WASM build 3분 31초·바인딩·호환성·Studio 2,716건 통과 | Canvas 글꼴 검사 중 전체 30분 상한 취소 |
| Archive B attempt 1 | 약 16분 | rhwp 컴파일 진행 | 전체 30분 상한 취소, 컴파일 오류 없음 |
| Archive D/E attempt 1 | 약 30분 | 후속 빌드 시간 부족 | 전체 30분 상한 취소 |
| Frontend attempt 3 | 30분 | checkout 미완료 | 전체 30분 상한 취소 |
| Archive B attempt 3 | 18분 19초 | archive build 11분 36초 진행 | 전체 30분 상한 취소, 컴파일 진단 없음 |

기준 devel의 추적 blob 합은 8.316 GiB, 후보는 8.324 GiB로 이번 증가분은 7.88 MiB다. 압축된 실제 전송량이나 fetch 원인을 확정하는 수치로 사용하지 않는다. 동일 head 재실행에서도 checkout 뒤의 검증 시간이 부족한 사례를 확인했다.

## 적용 범위

| job | 기존 상한 | 제안 상한 |
| --- | ---: | ---: |
| Lint | 20분 | 45분 |
| Proptest | 20분 | 45분 |
| Frontend package | 30분 | 60분 |
| release-test archive builder | 30분 | 60분 |
| release archive builder | 60분 | 60분 유지 |

상한은 정상 소요시간 목표가 아닌 유한 hang 감지 한도다. 관측한 checkout·설치·후속 검사에 시간을 배정한다. Cargo profile·test 목록·검사 명령·assertion·required check·보안 탐지·permissions·trigger·concurrency·cache 정책은 유지한다. Archive caller·reusable validator·계약 검사·운영 가이드를 같은 정책으로 맞춘다. Native·Render Diff·CodeQL·archive worker의 상한은 변경하지 않는다.

## 로컬 검증과 재사용

- Python workflow 계약 69건 PASS, Node CI 분류·정책·controller·duration 계약 183건 PASS: 총 252건, 실패 0건.
- `git diff --check`와 변경 문서 링크를 검사한다.
- 제품 `src/`·루트 `tests/`·Cargo manifest/lock·배포 WASM의 변경은 없다. H4의 전체 Rust 12,466건과 앞선 Native/fresh WASM 31입력·71쪽 측정은 원래 검증 head·범위로 유지한다. 이 CI 보정에서 다시 실행했다고 표시하지 않는다.
- 새 head에서는 PR 전체 변경에 대한 GitHub 필수 검증을 완료한다. 현재 H4의 성공을 새 head의 CI 성공으로 대신하지 않는다.

## 승인·적용·복구

기존 승인은 동일 head의 재실행에 한정됐다. 이 보정은 workflow 실행 한도를 바꾸므로 [GitHub 운영 규칙 5.4](../../../manual/github_operations.md#54-적용-승인)에 따라 diff·검증·rollback을 제시하고 승인 뒤 같은 PR에 commit·push한다. 새 PR은 만들지 않는다. 새 head 전환에 따른 기존 stale 실행 처리는 기존 정책을 따른다.

보정 뒤 필수 CI와 최종 policy 감사가 성공하면 이미 승인된 self-review·merge·원 PR 10건 후속 처리를 계속한다. 이상이 있으면 실제 실패 원인을 조사하며, 복구가 필요하면 이 보정 commit만 revert해 기존 상한을 복원한다. 제품 원 기여와 source 보정은 보존한다. 추가한 시간만으로 실패를 성공 처리하지 않는다.
