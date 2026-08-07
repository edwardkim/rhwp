# PR #4153 검토 기록

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4153](https://github.com/edwardkim/rhwp/pull/4153) |
| 이슈 | [#4152](https://github.com/edwardkim/rhwp/issues/4152) |
| 작성자 | `humdrum00001010` |
| base | `devel` |
| contributor 원 head | `06989c41fcf9ddbc1fe811aaffa0281183613448` |
| 메인터너 보정 head | `3a7c88b1395c9837d272806747519ce4ad441648` |
| 작성 시점 merge 상태 | `CLEAN` / `MERGEABLE` |

라우팅: collaborator 매개 외부 PR. 보조 경로는 접수·로컬 검증과 review-only fast-pass다.
시각·fixture 증적은 renderer, layout, sample, 기준 PDF를 바꾸지 않아 적용하지 않았다.

## 변경 검토

원 contributor 변경은 `tools/object_visual_regression.py`의 `-o` 기본값을 저장소 루트의
`output/ovr`로 정하고 CONTRIBUTING OVR·roundtrip 안내를 `output/`으로 통일한다. 이 경로는
루트 `.gitignore`의 `/output/` 규칙에 포함된다.

초기 검토에서 이슈 #4152가 요구한 편집-스윕 안내가 `out/sweep`으로 남은 것을 확인했다.
메인터너 보정 `3a7c88b13`은 `examples/edit_sweep.rs`의 Rustdoc 세 곳과
`mydocs/manual/verification/edit_sweep.md`의 명령·설명을 `output/sweep`으로 바꿨다.
일반 rhwp CLI의 독립적인 `out/` 관례는 이 이슈 범위 밖이므로 수정하지 않았다.

## 로컬 검증

- `git diff --check`를 실행해 whitespace 오류가 없음을 확인했다.
- `python3 -m py_compile tools/object_visual_regression.py`와 `--help`를 실행해 Python 문법과
  기본값 도움말이 정상임을 확인했다.
- 유효한 `samples/KTX.hwp`로 `-o` 생략 경로를 실행해 `output/ovr`가 생성되고 `/output/` ignore
  규칙에 매칭됨을 확인했다. 현재 `target/release/rhwp`가 없어 실제 OVR 산출까지는 진행하지 못했고,
  도구는 예상대로 바이너리 부재 오류 `2`로 종료했다. 생성한 빈 검증 디렉터리는 제거했다.
- `python3 scripts/check_markdown_links.py mydocs/manual/verification/edit_sweep.md`,
  `git check-ignore -v output/sweep/devel.tsv output/sweep/report.md`, `cargo fmt --check`를 통과했다.
- 최신 `upstream/devel`에 대한 merge simulation은 충돌 없이 완료했고 `git merge --abort`로 정리했다.

## CI와 판정

메인터너 보정 head `3a7c88b13`에서 GitHub Actions full CI, CodeQL, Native Skia, slow shard와
일반 shard 1/3·2/3·3/3, Build & Test가 모두 성공했다. review·오늘할일 head `9edcf0829`도
review-only fast-pass preflight와 Build & Test aggregate를 통과했다.

검토 중 source보다 최신인 `upstream/devel`의 오늘할일 전체를 복사하면 source에 없는 archive link가
깨짐을 Markdown 링크 검사로 확인했다. 최신 devel을 source에 merge하지 않고 현재 PR 항목만 변경되지 않은
section 경계에 넣은 뒤 merge tree 링크 검사로 확인하는 절차를 PR workflow와 collaborator 외부 PR 가이드에
보강했다.

**권고: 수용.** 이번 문서 보강 commit을 push한 뒤에는 같은 PR source branch의 녹색 code candidate를
review-only fast-pass로 재사용할 수 있는지와 최신 head의 preflight·Build & Test aggregate·mergeable
상태를 다시 확인한다. 최종 merge는 작업지시자 승인을 전제로 한다.
