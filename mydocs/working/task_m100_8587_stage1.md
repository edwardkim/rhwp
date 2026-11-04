# #8587 — duration API 게시·최종 trailing fast-pass 출처와 편람 환경 전달

2026-11-04 KST. [이슈 #8587](https://github.com/edwardkim/rhwp/issues/8587)을 먼저 등록했고,
사용자가 로컬 검증 완료 뒤 PR 제출과 ubuntu-ted의 지정 커밋 포함을 승인했다.
base는 `96827b3accd98ad0b80f76260b42fba24eded504`다. 검토·오늘할일은 아직 작성하지 않았다.
CI workflow 변경이므로 해당 code PR의 CI·머지 뒤 옵션 2로 따로 기록한다.

## 재현과 확정 범위

#8583 병합 후 duration은 실제 worker 자료 검증·46개 target 생성 뒤 Git push가 3회 실패했다.
첫 오류는 `fatal error in commit_refs_rules`, 이후는 `Internal Server Error`다.
metrics HEAD는 계속 `34cd7ca05391b929f9df695bea7a959e3b713be4`였고, 보호·ruleset 조회는 비어 있었다.
job의 `contents: write`도 확인했다. 권한·보호 규칙은 변경하지 않았다.

중간에 들어온 #8203을 포함한 merge simulation과 실제 #8583 merge tree는
`ac02bb51bfd48cda72a6a7e1f3a00240e450fa26`로 같다. detached HEAD 경고와 약 1만 줄 교체는
이전 #8565의 성공 run에도 있었고 관련 workflow·생성 코드는 같았다. 리베이스 생략이
이번 서버 오류를 일으켰다는 근거는 없다.

실제 B–E artifact에서 재생성한 JSON은 1,096,050 bytes·46개 target이며 로컬 bare FF와 원격
push dry-run이 통과했다. 같은 부모·tree를 비강제 Git Data API로
`a52efebae445294701637b0ce4892e416b878454`에 게시하고 원격 ref·blob을 확인했다.
전송 경로·인증 주체·시각이 함께 달라 GitHub 내부 원인을 특정하지 않는다.
기존 workflow의 실패를 성공으로 바꾸거나 retry만으로 해결했다고 표시하지 않는다.

## 수정과 경계

`325fdcf136`은 Git branch 전환·push를 Git Data API 게시로 대체한다. 정책 파싱·크기·blob 식별자를
확인하고 같은 데이터는 새 commit 없이 끝낸다. 기존 tree를 보존하고 이전 metrics HEAD를 단일
부모로 지정하며 `force:false`로 갱신한다. 게시 직전 devel·metrics를 재조회해 준비 중 전진을
보류하고, ref 경쟁 거절은 최신 자료를 보존한다. 응답 유실은 정확한 후보 ref의 재조회로 구분하며
서버·권한 오류는 진단과 함께 실패한다. 무조건 retry·권한 확대·CI fallback을 추가하지 않는다.
두 ref 재조회는 원자적 트랜잭션이 아니므로 동시 devel 전진을 영구 배제한다고 주장하지 않는다.

최종 head의 최신 CI가 성공인지 확인하고, review-only tail을 검증해 실제 측정 code CI를 연결한다.
최종 head의 실패·진행 중·재실행·다른 fork·source 변경은 과거 녹색으로 대신하지 않는다.
최종 run/head/attempt와 측정 run/head/worker attempt/artifact를 결과에 따로 기록한다.
생략 worker는 측정 없음으로 표시하며 실제 실패와 혼동하는 Notice를 없앴다.

ubuntu-ted의 `a4fb27f651e17dc1b238b93005086fc165fe9e1b`는 출처를 남겨 `4c892a9886`으로
체리픽했다. `--handbook-gate` 단독·추가 실행이 요청된 `--font-environment`를 실제 Native
전쪽 출력까지 전달한다. 옵션 생략·쪽수·90%·#8389 하한은 유지한다. 원격 작업장은 읽기만 했고
그곳의 변경이나 다른 커밋을 가져오지 않았다. 인자 전달은 Native 전용 검사 경로이며 Rust·WASM
renderer와 일반 대상의 기존 환경 전달을 바꾸지 않는다.

## 실행한 검증

- Node 수집·게시·artifact 검증·정책: 157 PASS / 0 FAIL.
- workflow 계약 4모듈: 45 PASS / 0 FAIL. 관련 새 publisher로 이동한 게시 대상·비강제 계약도 확인했다.
- `actionlint .github/workflows/refresh-nextest-duration.yml`: PASS.
- Visual Sweep Python 전체: 247 PASS / 0 FAIL.
- 실제 workflow 준비 shell을 CI와 같은 origin을 가진 임시 Git 문맥에서 실행: exit 0,
  기존 46-target JSON SHA-256과 동일. 최초 YAML 모듈 부재·symlink entrypoint 진단은 통과 근거에서 제외했다.
- 실제 #8583 API 수집: 최종 trailing run `41808163341`·head `0ebb66d4e5`와 code run
  `41802090733`·head `4446720c72`·최신 attempt 2 / B–E 실제 attempt 1을 연결했다.
  같은 자료의 실제 publisher 재조회는 `unchanged`로 끝나 추가 쓰기가 없었다.
- 명시적 글꼴 환경 Native 편람: 전383쪽·누락/중복0·최저99.96463%·130쪽, #8389 기록 하한 하락0,
  201.964초·exit0. 145쪽 raw100%이며 독립 PDF와 raw/review/overlay를 직접 열어 확인했다.

Native 명령은 다음과 같다.

```sh
venv/bin/python scripts/visual_sweep.py --handbook-gate \
  --font-environment mydocs/pr/assets/planet6897_integration_final/pr8389_8397/handbook-font-environment.json \
  --rhwp-bin output/pr-review/issue8533-wasm-advance-20261104/rhwp-final-native \
  --page-workers 8 --out output/pr-review/issue8587-duration-publish-20261104/handbook-font-environment
```

이번 검사는 Python 전달 경로를 실행했으며 #8583에서 검증한 Native 바이너리 SHA
`8ae681680593809015ebb5c0a19ddaa5269ba11aa56d467882d6be81ab3d8728`를 재사용했다.
새 Rust build 또는 새 WASM 전쪽 실행으로 표시하지 않는다. [생산 신원·게이트·실제 API 결과](../pr/assets/issue8587/validation.json),
[전383쪽 TSV](../pr/assets/issue8587/handbook-native.tsv), [145쪽 review](../pr/assets/issue8587/native-p145-review.png),
[overlay](../pr/assets/issue8587/native-p145-overlay.png)에 정식 증거를 연결했다.
기존 #8583의 WASM 전쪽 결과는 별도 범위이며 이번 Native 옵션 결과와 합치지 않는다.

실행 로그·임시 driver는 `output/pr-review/issue8587-duration-publish-20261104/` 및 기존 #8583
output에 있다. 제출할 것은 위 정식 증적뿐이다. Cargo·WASM·제품 source 변경은 없으므로
GitHub 운영 매뉴얼 O3의 해당 command·workflow 계약을 실행했고 제품 전체 빌드는 재실행하지 않았다.
새 PR의 정확한 head Actions와 병합 후 GITHUB_TOKEN 실제 게시 확인은 아직 완료하지 않았다.
