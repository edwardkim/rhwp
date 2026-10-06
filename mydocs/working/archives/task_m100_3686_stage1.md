---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3686.md
last_verified: 2026-08-01
---

# Task #3686 Stage 1~3 보고 — 액션 SHA 고정 + 갱신 경로

## 결과 — 이동 가능 참조 0건

| 구분 | 착수 전 | 완료 후 |
|---|---:|---:|
| SHA 고정 | 0 | **65** |
| 태그·브랜치 참조 | 65 | **0** |

`git diff --stat`: 9파일 / 65 insertions / 65 deletions (1:1 치환).

## 계획 대비 정정 — 12건이 추가로 발견됨

계획서는 **53건**(SemVer 태그 참조)으로 집계했으나, 실제로는
**`dtolnay/rust-toolchain@stable` 12건**이 더 있었다. 정규식이 `@v[0-9]` 패턴만 세어
비-SemVer 참조를 놓친 것이다.

이 12건은 **태그가 아니라 브랜치**(`refs/heads/stable`, commit 직접 참조)라 **위험이
더 크다** — 태그는 저자가 의도적으로 옮겨야 하지만 브랜치는 커밋마다 자동으로 움직인다.
`4cda84d5c5c54efe2404f9d843567869ab1699d4` 로 고정하고 `# stable 브랜치 (2026-08-01 시점)`
주석을 달았다(SemVer 버전이 없으므로 시점을 명시).

**교훈**: "태그 고정"이라는 프레임이 브랜치 참조를 시야에서 지웠다. 다음에는
`uses:` 전수에서 40자 hex 가 아닌 것을 세는 방식으로 접근한다.

## 고정 목록

### 서드파티 (18건)

| 액션 | 건수 | SHA | 버전 |
|---|---:|---|---|
| `dtolnay/rust-toolchain` | 12 | `4cda84d5…` | stable 브랜치 시점 |
| `Swatinem/rust-cache` | 3 | `e18b4977…` | v2.9.1 |
| `taiki-e/install-action` | 2 | `6a1bd70e…` | v2.85.5 |
| `softprops/action-gh-release` | 1 | `3d0d9888…` | v3.0.2 |

### 공식 `actions/*` (47건)

checkout 15(`3d3c42e5…` v7.0.1) · upload-artifact 7(`043fb46d…` v7.0.1) ·
setup-node 7(`82076278…` v7.0.0) · github-script 6(`3a2844b7…` v9.0.0) ·
download-artifact 5(`3e5f45b2…` v8.0.1) · cache 5(`55cc8345…` v6.1.0) ·
upload-pages-artifact 1(`fc324d35…` v5.0.0) · deploy-pages 1(`cd2ce8fc…` v5.0.0)

## 태그 이동 대조 (계획서 §2 규약)

계획 수립 시 조회한 SHA 와 구현 직전 재조회 SHA 를 **11종 전부 대조 — 전건 일치**.
태그 이동 흔적 없음.

## Stage 3 — 이미 충족돼 있었다

`.github/dependabot.yml` 에 **`github-actions` ecosystem 이 이미 등록**되어 있다
(directory `/`, target-branch `devel`, weekly, PR 한도 10). 고정 후 갱신 경로가
확보된 상태이므로 **추가 작업 불요**.

계획서는 이를 "신규 활성화"로 잡았으나 실측 결과 기존 설정으로 충족된다. 고정만
하고 갱신 경로가 없는 위험(계획서 §5)은 발생하지 않는다.

## 검증

- 전 워크플로 YAML 문법 파싱 통과.
- `shared-key: native-skia` + `save-if`(#3123 배선) 유지 확인.
- 로컬 composite action(`.github/actions/install-wasm-pack`) 내부에 외부 `uses:`
  참조 없음 — 숨은 미고정 없음.
- 잔여 기능 검증은 **CI 실제 통과**(Stage 4).
