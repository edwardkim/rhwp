# Task #3604 관련 구현 계획

Issue: #3604

## Stage 1: archive와 manual 교체

1. hwp-convert에서 최신 client tarball을 생성하고 manifest/help를 확인한다.
2. rhwp `tools/`의 이전 tarball을 제거하고 새 tarball을 추가한다.
3. manual front matter 확인일과 archive 경로를 갱신한다.
4. password stdin, 지원 방향, 긴 문서 timeout, `response_finished` 뒤 server cleanup 설명을 검증 결과에 맞춰 정리한다.

## Stage 2: 검증·PR

1. archive manifest와 `npx` help, 문서 내 tarball 참조 수, Markdown 링크, diff를 확인한다.
2. 코드·archive·문서·stage 기록을 한 commit으로 고정한다.
3. upstream 원본 저장소 작업 branch에 push하고 `devel` 대상 PR을 한국어 제목·본문으로 생성한다.
4. 최신 PR head CI를 관찰하고 결과를 archive review 기록에 남긴다.

## 보안과 보존

- token, endpoint IP, 비밀번호, server 내부 경로는 commit과 PR 본문에 넣지 않는다.
- client tarball에는 `.env.local`이 아닌 `.env.local.example`만 포함해야 한다.
- 기존 client tarball은 새 archive가 manifest/help 검증을 통과한 뒤에만 제거한다.
