# v0.9.0 동기화·종료 정리

완료 시각: 2026-11-08T15:06:29.394760+00:00 (UTC).

사용자가 지정한 devel 동기화·릴리즈 전용 브랜치·output/debug 정리를 완료했다.
[실행 메타데이터](../pr/assets/release-0.9.0-execution/cleanup.json)에 삭제 범위·용량·확인 결과를 기록했다.

| 항목 | 결과 |
| --- | --- |
| devel | `2a9a88a0dd`에서 `4aa8f5fc2b`로 3개 커밋 fast-forward; 원격과 동일 |
| 전용 브랜치 | `release/0.9.0-crates-metadata`의 로컬·upstream ref 삭제; 최종 PR head를 지정한 lease 사용 |
| 원 PR | #8784 MERGED, merge SHA의 devel 포함·검토 문서와 증적 반영·duration 갱신 성공 확인 |
| output | 디렉터리 전체 4.47 GiB 제거, 경로 부재 확인 |
| target/pr-review/debug | 30.36 GiB 제거, 경로 부재 확인 |
| 실제 여유 공간 | 154.32 → 189.15 GiB; 34.83 GiB 증가 |
| 재사용 캐시 | release·release-test·WASM 등 다른 profile은 유지 |
| 소비 검증 환경 | `local/crates-io-consumption/` 유지, 주요 파일의 삭제 전후 SHA-256 동일 |
| 별도 작업공간 | `/home/edward/mygithub/rhwp-worktrees/pr-8296-seal-pua` 유지; 이번 릴리즈 소유가 아니며 untracked node_modules·venv 존재 |

삭제 전에 Cargo·rustc·wasm-pack·wasm-opt, Docker 컨테이너와 대상 경로의 exe·fd·map·명령 인자를 확인했다.
활성 사용은 없었다. 종료 정리용 복제본·압축본은 만들지 않았다.
임시 자료에만 있던 필수 최종 검증 결과는 [배포 보고서](../report/release_0_9_0_execution_report.md)와
연결된 최종 증적으로 반영했다. 소비 환경은 사용자의 명시 보존 지시를 따른다.

정리 완료 시점에는 보고서·증적·이 기록이 로컬 변경이었다. 이후 최종 기록 후속 PR에
npm publisher 안내 보완과 함께 제출한다. 태그 CI의 미완료 검사와 Chrome·Edge 공개 전환
확인은 배포 보고서의 잔여 항목으로 유지한다. 이번 정리를 전체 릴리즈 후속 작업 완료로
확대하지 않는다.
