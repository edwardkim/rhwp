# 구현계획서 — #2431 C Native Skia rust-cache 통합

## 목표

`native-skia-tests` job이 writer 없는 `Linux-cargo-*` namespace를 계속 fallback restore하는 문제를
해소한다. 기본 test archive와 lint에서 이미 사용하는 `Swatinem/rust-cache` 체계로 통합하되,
Native Skia feature 산출물은 별도 namespace로 분리한다.

## 현재 동작

`.github/workflows/ci.yml`의 `native-skia-tests` job은 다음 경로를 cache한다.

- `~/.cargo/registry`
- `~/.cargo/git`
- `target`

key는 `Linux-cargo-${Cargo.lock hash}`이고 `Linux-cargo-` restore key를 사용하지만, 이 namespace를
갱신하는 save step은 없다. 2026-07-23 재계측에서도 2026-04-04에 생성된 426,792,350 B fallback이
08:24:48 KST에 다시 접근돼, 오래된 cache를 계속 복원하는 동작이 유지됨을 확인했다.

`upstream/devel` `29b5547e2`의 #3064 CI fail-fast 변경은 Native Skia의 선행 gate만 강화했으며
cache reader는 바꾸지 않았다. PR 게시 중 진행된 최신 `cbddc1cd8`의 #3124는 문서 변경뿐이다.
따라서 최신 CI 의존성 구조를 보존하면서 아래 cache step만 교체한다.

## 파일별 변경

### `.github/workflows/ci.yml`

1. Native Skia의 `actions/cache/restore@v5` step과 수동 path/key 설정을 제거한다.
2. Rust toolchain 설치 뒤 `Swatinem/rust-cache@v2` step을 추가한다.
3. `shared-key: native-skia`로 default-feature archive와 lint cache를 분리한다.
4. `save-if`는 다음 조건만 허용한다.
   - event: `push`
   - ref: `refs/heads/devel` 또는 `refs/heads/main`
5. Native Skia 패키지 설치, test command, profile 분기와 required check 집계는 변경하지 않는다.

## 의도한 lifecycle

| 실행 | 동작 |
|------|------|
| C 구현 PR | matching cache가 없으면 cold, save는 차단 |
| C merge 후 devel push | restore 후 `native-skia` cache save |
| 후속 PR | devel에서 생성한 `native-skia` cache restore |
| Cargo/Rust 환경 변경 | rust-cache 환경 hash에 따라 새 trusted cache 생성 |

첫 PR에서 warm hit를 완료 조건으로 삼지 않는다. trusted branch writer가 처음 실행된 뒤의 후속 PR에서
restore를 확인해야 lifecycle 전체가 검증된다.

## 범위 제외

- `Swatinem/rust-cache` SHA pinning: D PR
- PR close cleanup workflow: A PR
- frontend npm cache 비활성화: B PR
- legacy `Linux-cargo-*` 삭제
- Native Skia test command, profile, runner package 변경

## 검증

- `actionlint .github/workflows/ci.yml`
- Ruby YAML parse
- `git diff --check`
- Native Skia job에 `shared-key: native-skia`와 trusted-branch `save-if`만 들어갔는지 diff 검토
- 다른 cache job과 required check 표면이 바뀌지 않았는지 정적 대조

## 위험과 완화

- C PR 자체는 writer가 차단되므로 cold일 수 있다. merge 후 devel과 후속 PR을 나누어 관측한다.
- 새 cache가 저장된 직후에는 legacy cache와 일시 공존해 quota가 늘 수 있다. 현재 닫힌 PR cache의
  일회성 cleanup 승인을 별도로 받고, legacy cache는 warm 검증 전 삭제하지 않는다.
- C에서는 기존 사용과 동일하게 `@v2`를 사용한다. full SHA 고정은 메인테이너가 분리 승인한 D에서
  세 사용처를 함께 처리한다.
