---
kind: review-implementation
status: completed-local
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# humdrum00001010 PR 3건 누적·보정 기록

## 검토 브랜치와 적용 순서

가시성 검토 브랜치 `review/humdrum00001010-20260807`은 최신 `upstream/devel`에서 만들었다. source
commit은 rewrite·amend·force-push하지 않았고, 아래 순서로만 누적했다.

| 단계 | 원 PR | source commit |
| --- | --- | --- |
| 1 | #4124 | `e7b58d8e179b41b96cebe24378041ef0cfca8186` |
| 2 | #4127 | `66fe73b9`, `2dd8a834`, `81ae3e64`, `fddbbda6`, `5eca7e85` |
| 3 | #4131 고유분 | `2fdc600f`, `766e533e`, `d3879a51`, `f40ee14b`, `1cc01130`, `72d4ac96`, `6dc0d9fc`, `2f4b36d6`, `bd2a65e9` |
| 4 | 메인터너 보정 | #4124 picker 흐름 분리와 SecurityError/AbortError 실제 회귀 추가 |

`git merge-base --is-ancestor`로 #4127 head가 #4131 head의 ancestor임을 확인했다. 따라서 #4131을
별도로 전체 cherry-pick하지 않고 고유 9개 commit만 적용해 중복을 피했다. 모든 단계 뒤
`git diff --check`를 통과했고 현재 source 누적분은 `upstream/devel`보다 15 commit 앞선 상태다.

## 메인터너 보정 범위

보정은 #4124에만 적용한다. `openDocumentViaPicker`는 저장 전 확인, native picker, 취소 판정, hidden input
fallback, handle read 실패 안내를 한 모듈로 모으고 `file:open`은 UI service를 주입한다. 이로써 Node 24의
표준 테스트 runner에서도 alias 해석에 의존하지 않고 실제 실패·취소 동작을 검증한다.

다른 두 PR의 renderer·layout 동작은 보정하지 않았다. #4131은 #4127 위에 쌓인 구조이므로 수용·원격 반영 시
순서를 유지해야 하며, #4127을 건너뛰는 독립 병합은 하지 않는다.

## 완료한 검증

1. Rust focused regressions 5개와 기존 인라인/미주 캐럿 회귀 9개를 통과했다.
2. 작업지시자가 현재 누적 브랜치에서 `cargo test --profile release-test --tests`를 정상 종료까지 실행했다.
3. Native Skia 라이브러리 58개와 placeholder/PDF 통합 6개를 통과했다.
4. `cargo clippy --all-targets -- -D warnings`, `wasm-pack build --target web --out-dir pkg`, Studio build를 통과했다.
5. Studio 단위 765개와 새 picker 단위 2개를 통과했다.
6. Linux Chromium UI에서 실제 `file:open` command에 `SecurityError`를 주입했다. alert 없이 hidden input이
   한 번 열리고 `skipUnsavedGuard=true`가 설정됐다.

## 원격 반영 전 조건

- 이 기록과 보정 code/test는 하나의 일반 commit으로 고정한다.
- 작업지시자 승인 전에는 contributor branch 또는 원본 저장소에 push하지 않는다.
- 원격 반영 직전 세 PR의 head SHA, required checks, `mergeable`, `mergeStateStatus`를 다시 조회한다.
- #4124가 GitHub mergeability 재계산을 마칠 때까지 `UNKNOWN`을 merge 가능 판정으로 해석하지 않는다.
