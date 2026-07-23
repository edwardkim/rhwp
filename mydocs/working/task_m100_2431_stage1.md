# 단계 완료보고서 — #2431 Stage 1 Native Skia rust-cache 통합

## 결과

`native-skia-tests` job의 writer 없는 `Linux-cargo-*` 수동 reader를 제거하고,
`Swatinem/rust-cache@v2`의 별도 `shared-key: native-skia`로 통합했다.

PR run은 restore-only로 유지하고, 다음 trusted branch push에서만 cache를 저장한다.

- `refs/heads/devel`
- `refs/heads/main`

## 변경 범위

변경한 workflow는 `.github/workflows/ci.yml` 한 곳이다.

- 제거: `actions/cache/restore@v5`, 수동 path/key/restore-key
- 추가: `Swatinem/rust-cache@v2`
- 추가: `shared-key: native-skia`
- 유지: 기존 devel/main push `save-if` 정책
- 불변: Native Skia 패키지 설치, test command, profile 분기, `Build & Test` 집계

계획·오늘할일·이 단계 보고서를 제외한 제품 코드와 다른 workflow는 변경하지 않았다.

## 검증

| 검증 | 결과 |
|------|------|
| `git diff --check`와 `git diff upstream/devel...HEAD --check` | 통과 |
| Ruby `YAML.load_file` | `yaml ok` |
| `actionlint .github/workflows/ci.yml` | 예외 없이 통과 |
| 최신 devel 재배치 | `29b5547e2`, 충돌 해결 후 2커밋 재배치 |
| 변경 파일과 diff 수동 대조 | workflow 변경 1곳, Native Skia cache step만 변경 |

이전 기준선에서 발견됐던 `SC2012`는 upstream #3064 작업에 포함된 `find` 전환으로 해소됐다.
최신 기준선에서는 예외 없이 전체 `actionlint`를 통과했다.

## 2026-07-23 재판단

- `upstream/devel`이 최초 기준선보다 46커밋 진행됐지만 #2431 C의 중복 구현은 없었다.
- 이슈 착수 코멘트 이후 메인테이너의 추가 답변과 #2431 중복 PR은 없다.
- 전체 cache는 43개 / 10,149,416,965 B로 기본 10GB 한도를 여전히 넘는다.
- 닫힌 PR ref cache는 13개 / 804,483,656 B이며 메인테이너 확인 전 삭제하지 않는다.
- `Linux-cargo-*`는 2개 / 2,211,692,024 B다. 특히 writer가 없는 fallback cache가
  2026-07-23 08:24:48 KST에도 접근돼 C의 필요성이 다시 확인됐다.
- C 구현은 최신 CI 의존성 구조와 Native Skia 명령을 유지하고 cache step만 교체한다.

## 원격에서 남은 검증

1. C PR run에서 `native-skia` key 조회와 PR save 차단 확인
2. merge 후 devel push에서 `native-skia` cache save 확인
3. 후속 PR에서 devel cache restore와 Native Skia compile 시간 확인
4. warm lifecycle 확인 뒤 legacy `Linux-cargo-*` cleanup 승인 판단

첫 C PR은 trusted writer가 실행되기 전이므로 cold일 수 있다. 따라서 PR run만으로 cache lifecycle 완료를
판정하지 않는다.

## 미수행 작업

- cache 삭제 없음
- push와 Draft PR 생성은 작업지시자 승인 후 게시 준비 중
- A 이벤트 보정과 기존 stale PR cache cleanup은 메인테이너 답변 대기
- D SHA pinning, A cleanup workflow, B npm cache 비활성화는 별도 단계와 PR로 유지
