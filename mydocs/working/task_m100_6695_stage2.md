# #6695 Stage 2 — PDF 오라클 단일 경로 원자 전환

## 1. 판정

**Stage 2 구현·로컬 검증 완료**다. 메인테이너 결과 승인 전이며 원격에는 push하지 않았다.

- 한컴 PDF 오라클: `pdf/**` 1,178개, 1,079,000,023 bytes
- 폐기된 최상위 경로: `pdf-2010/`, `pdf-2020/`, `pdf-large/` 모두 없음
- 전환 원장: 19/19개 목적지의 크기·SHA-256·`%PDF-` magic 일치
- LFS: 최종 staged tree의 LFS 파일 0개, `pdf/**`의 `filter=lfs` 적용 0개
- 크기: 1,178개 모두 50 MiB 미만, 최대 50,228,784 bytes
- 정책 검사: 위반 0개
- 정책 단위 테스트: 7/7 통과

Stage 1에서 고정한 bytes를 바꾸지 않고 17개 신규 목적지를 만들었으며, 기존 `pdf/`와
byte-identical한 2개는 목적지를 보존하고 `pdf-large/` source만 제거했다.

## 2. 자산 전환

### 2.1 실제 PDF 복원

작업트리에 131-byte pointer만 있던 `pdf-large/hwpx/issue_1133.pdf`는 로컬 LFS object store에서
복원했다. 복원 직후 다음 값을 다시 확인했다.

| 항목 | 값 |
| --- | --- |
| actual byte | 110,523 |
| SHA-256 | `ca066fdc5cda9fd8c47c9a29ba9a546e70c6acdaa96df4d6c66e0b8e02cdbc45` |
| magic | `%PDF-1.4` |

복원 뒤 `pdf/hwpx/issue_1133.pdf`로 옮겨 일반 Git blob으로 stage했다. 네트워크 다운로드나 원격
LFS 변경은 수행하지 않았다.

### 2.2 이동 결과

- `pdf-2020/pr-1674-2020.pdf` 1개를 `pdf/pr-1674-2020.pdf`로 이동했다.
- `pdf-large/hwpx/**` 14개와 `pdf-large/issue2006/**` 1개를 상대 구조를 보존해 이동했다.
- `pdf-large/hwpx/2026_oss_rst.pdf` 일반 blob도 `pdf/hwpx/**`로 이동했다.
- `pdf-large/` 최상위의 동일 PDF 2개는 기존 `pdf/` 목적지를 보존하고 source만 제거했다.
- 신규 Git blob으로 들어오는 실제 PDF bytes는 17개, 55,214,491 bytes다.
- `pdf-large/README.md`의 유효한 규칙은 `pdf/README.md`에 통합하고 폐기 디렉터리를 제거했다.

LFS pointer에서 실제 blob으로 바뀐 파일은 Git의 rename 유사도 표시에 의존하지 않는다. Stage 1
원장에 기록한 content SHA-256을 새 목적지에서 직접 다시 계산해 19/19 일치를 확인했다.

## 3. 단일 경로 가이드

`pdf/README.md`와 `CONTRIBUTING.md`에 다음 계약을 명시했다.

1. 한컴 PDF 정본 오라클은 버전·크기와 무관하게 `pdf/**`만 사용한다.
2. 버전·입력 형식·폰트 조건은 디렉터리가 아니라 파일명 suffix로 보존한다.
3. 정본 오라클은 일반 Git blob이며 Git LFS pointer를 사용하지 않는다.
4. 파일 하나는 50 MiB(52,428,800 bytes) **미만**이어야 한다.
5. 초과 자산은 바로 커밋하지 않고 축소 fixture·페이지 발췌·외부 증적 방식을 이슈에서 승인받는다.
6. `samples/**`, `mydocs/**/assets`, 도구의 tiny fixture처럼 역할이 다른 PDF는 각 소유 경로를
   유지하며, 한컴 정본 오라클로 승격할 때만 `pdf/**` 규칙을 적용한다.

삭제한 `pdf-large/README.md`를 직접 가리키던 `pdf/issue5447/README.md`와 LFS 정책을 설명하던
`pdf/pr3740/README.md`도 새 정본으로 연결했다. `.gitattributes`의
`pdf-large/**/*.pdf filter=lfs` 규칙은 제거했다.

## 4. 정책 검사기

신규 `scripts/check_pdf_repository_policy.py`는 저장소 전체의 기능별 PDF를 무차별 이동시키지 않고,
정본 영역에 아래 fail-closed 검사를 적용한다.

- 폐기된 최상위 `pdf-2010/`, `pdf-2020/`, `pdf-large/` 존재 여부
- `pdf/**` PDF의 `%PDF-` magic과 50 MiB 미만 상한
- 작업트리에 드러난 LFS pointer
- `.gitattributes`로 `pdf/**`에 적용되는 `filter=lfs`
- 작업트리가 hydrate되어 있어도 Git index에 숨은 LFS pointer

Git index 검사는 `git-lfs` 출력에만 의존하지 않는다. `git ls-files --stage`와
`git cat-file --batch-check`로 작은 blob 후보만 읽어 pointer header를 확인한다. 따라서 실제 PDF
1.079 GB를 매번 hash하지 않으면서도 hydrate된 포인터를 놓치지 않는다.

단위 테스트 7개는 정상 Git blob, 폐기 경로, 작업트리 pointer, hydrated/index pointer, LFS attribute,
PDF magic 오류, 정확히 50 MiB인 경계 파일을 각각 판별한다.

## 5. 검증 결과

```text
python3 -m unittest scripts.tests.test_pdf_repository_policy -v
Ran 7 tests
OK

python3 scripts/check_pdf_repository_policy.py
PDF repository policy: OK (1178 PDFs, each < 52428800 bytes, no LFS pointers)
WSL2 cold/warm 상태가 섞인 1회 참고값: 0.24초, 최대 RSS 52,608 KiB

Stage 1 SHA-256 원장 재검산
ledgerRows=19, verified=19, errors=0

git lfs ls-files --name-only $(git write-tree)
0 files

git check-attr filter -- <대표 신규 PDF 3개>
모두 unspecified
```

추가로 `git diff --cached --check`를 통과했다. 이 단계에는 Rust source·Rust test 변경이 없으므로
Rust lint 묶음 대상이 아니다.

## 6. 다음 단계 경계

실행 workflow·oracle generator·LLM verifier·현행 운영 문서에는 아직 폐기 경로 문자열이 남아 있다.
이는 Stage 1 소비자 지도에 고정한 Stage 3 범위이며, Stage 2에서 수기 일괄 치환하지 않았다.

메인테이너가 Stage 2 결과를 승인하면 Stage 3에서 다음 순서로 처리한다.

1. workflow와 CI classifier/policy의 PDF-only fast-pass를 `pdf/**` 하나로 축소한다.
2. Oracle Public source/test를 단일 root로 바꾸고 정본 생성기로 산출물을 재생성한다.
3. LLM verifier 계약·fixture·코퍼스를 정본 generator로 갱신한다.
4. active source/test와 현행 운영·기술 문서를 새 경로로 바꾼다.
5. 과거 완료 기록은 보존하고 memory 문서에는 superseded 안내만 추가한다.
