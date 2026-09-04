# Task M100 #1423 최종 보고서 — Cargo.lock git track

- 이슈: #1423 "Cargo.lock 커밋 필요성"
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1423`
- 작성일: 2026-06-20

## 1. 개요

`Cargo.lock` 을 git 추적 대상으로 전환했다. rhwp 는 `rhwp` 바이너리(export-svg/pdf/dump 등)
를 산출하는 bin 패키지라, Cargo 공식 가이드가 권장하는 "재현 가능 빌드를 위한 Cargo.lock
커밋" 대상이다.

## 2. 변경

- `.gitignore`: `# Rust` 섹션의 `Cargo.lock` 줄 제거 + 추적 근거 주석.
- `Cargo.lock`(신규 추적): 현재 상태 그대로 add/commit. **`cargo update` 미실행** —
  재현성 목적으로 임의 의존성 변경을 하지 않았다(현 lock = `rhwp 0.7.16` 정합).

## 3. 검증

- `git check-ignore Cargo.lock`: ignore 해제 확인. `git add` 후 `A`(신규 추적).
- `cargo metadata --locked`: 무오류 (lock ↔ Cargo.toml 정합).
- `cargo build --locked`(lib) / `cargo build --locked --bin rhwp`: 둘 다 성공.
- 빌드 후 `Cargo.lock` 불변 (의존성 변경 없음 — 재현성 확인).

## 4. 추가 이점 — CI 캐시 정상화

CI workflow 다수가 이미 `hashFiles('**/Cargo.lock')` 를 cargo 캐시 키로 쓰고 있다
(ci.yml, codeql, render-diff, npm-publish, deploy-pages, release-binary,
full-renderer-sweep). render-diff.yml 은 `Cargo.lock` 변경을 트리거 경로로 둔다.

lock 이 미추적일 때는 이 캐시 키가 빈/불완전 해시로 계산됐다. **lock 추적으로 cargo 캐시
키가 정상 동작**해 CI 캐시 효율이 개선된다(충돌 없음 — 인프라가 이미 lock 존재 전제).
오프라인/Nix vendoring/하위 프로젝트 SSOT 라는 제안 목적도 충족.

## 5. bindings/Native 는 제외 (iOS FFI 영역)

루트 `.gitignore` 의 `Cargo.lock` 제거로 하위 패키지 `bindings/Native/Cargo.lock`
(rhwp-native-ffi, cdylib/staticlib, iOS/Swift FFI)도 노출됐다. 그러나:

- 이 lock 은 outdated(toml 5/14 > lock 5/4)이고, 정합화(`generate-lockfile`)하면 138개
  의존성이 최신 호환으로 갱신된다(임의 변경).
- 더 근본적으로 `bindings/Native/src/lib.rs:216-217` 이 메인 `rhwp_core` 의 clipboard
  메서드 시그니처 변경(3→4 인자)을 못 따라가 **소스 자체가 컴파일 실패**(E0061). lock 과
  무관한 코드 drift.
- bindings/Native 는 iOS FFI 영역(`ios/devel` 분기, 맥북 전용)으로 Linux 메인 환경과 장기
  분리돼 있다.

→ 작업지시자 결정으로 **bindings/Native 는 제외**. `.gitignore` 에 `bindings/Native/Cargo.lock`
을 명시 추가해 추적하지 않는다. 소스 drift 정리는 iOS 영역 별도 타스크.

## 6. 후속

- 의존성 갱신(`cargo update`)은 본 타스크 범위 밖 — 필요 시 별도 PR 로 관리한다.
- bindings/Native 소스 drift(FFI clipboard 시그니처) 는 ios/devel 동기화 시 별도 처리.

## 6. 산출물

- 수행계획서: `mydocs/plans/task_m100_1423.md`
- 최종 보고서: 본 문서
- `.gitignore`(루트 Cargo.lock 제거 + bindings/Native/Cargo.lock 명시 제외),
  `Cargo.lock`(루트, 신규 추적)
