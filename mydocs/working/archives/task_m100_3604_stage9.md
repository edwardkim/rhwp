---
kind: working
status: completed
issue: 3604
stage: 9
last_verified: 2026-08-01
---

# #3604 Stage 9: Studio lockfile 정합성 고정

## 관측

- `rhwp-studio/package.json`의 package version은 `0.8.2`지만 tracked lockfile root package
  version은 `0.7.19`였다.
- 현재 npm이 생성하고 유지하는 lockfile은 root version을 `0.8.2`로 맞춘다.
- 나머지 변경은 Rollup Linux optional package의 `libc` metadata 직렬화 제거이며, package name,
  resolved version, integrity 값은 바꾸지 않는다.

## 구현 계획

1. 현재 npm으로 package-lock-only 재생성을 수행해 lockfile이 안정적인지 확인한다.
2. `npm ci --dry-run --ignore-scripts`로 lockfile과 manifest의 설치 계획을 검증한다.
3. lockfile과 이 stage 문서를 함께 일반 커밋한다.

## 안전성 경계

- `node_modules`를 삭제하거나 재설치하지 않는다.
- package.json의 의존성 범위와 resolved package version은 변경하지 않는다.
- 사용자가 추가한 암호 HWPX fixture는 Git에 추가하지 않는다.

## 테스트 결과

| 검증 | 결과 |
| --- | --- |
| `npm install --package-lock-only --ignore-scripts` | 통과: 기존 package-lock 변경을 유지하며 추가 diff 없음 |
| `npm ci --dry-run --ignore-scripts` | 통과: lockfile 기반 설치 계획 생성 |
| `git diff --check -- rhwp-studio/package-lock.json` | 통과 |

현재 Node 20에서는 Puppeteer 25의 Node 22.12 이상 engine warning이 출력되지만, lockfile의
의존성 해석과 dry-run 설치는 성공했다. 이 stage는 Node runtime upgrade를 변경하지 않는다.
