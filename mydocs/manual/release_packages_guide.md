---
kind: guide
status: active
canonical: mydocs/manual/release_packages_guide.md
last_verified: 2026-08-09
---

# 패키지 릴리스 가이드 — PyPI 휠·npm (R63, #4336)

`.github/workflows/release-packages.yml` 의 운영 문서다. 이 파이프라인은 **켜기
전까지 아무것도 게시하지 않는다** — 시크릿이 없으면 빌드·검증까지만 수행하고
게시 단계를 로그에 사유를 남기며 건너뛴다. 즉:

- **워크플로 머지** = "배포 능력"의 채택 (태그마다 휠·sdist 가 만들어지고 4플랫폼
  설치 스모크가 검증됨 — 게시는 없음)
- **시크릿 등록** = "배포 실행"의 채택 (다음 태그부터 실제 게시)

결정 권한은 전부 메인테이너에게 있다.

## 1. 무엇이 만들어지는가

| 산출물 | 내용 |
|---|---|
| 플랫폼 휠 4종 | `rhwp-X.Y.Z-py3-none-{manylinux_2_39_x86_64, macosx_10_12_x86_64, macosx_11_0_arm64, win_amd64}.whl` — `rhwp/_bin/` 에 해당 플랫폼 릴리스 바이너리 동봉 |
| sdist 1종 | 순수 파이썬 — 바이너리 없음, 설치 후 `RHWP_BIN`/`PATH` 탐색(종전과 동일) |
| npm `@rhwp/node` | 커밋된 생성 타입 기준 빌드본 — 바이너리 미동봉(§5 후속), `RHWP_BIN`/`PATH` 래퍼 |

런타임은 무변경이다 — `src/rhwp/_binary.py` 의 3단 탐색(`RHWP_BIN` → 패키지
동봉 `_bin/` → `PATH`)은 `bindings_foundation.md` §3 이 이미 고정한 계약이고,
휠 동봉은 그 2순위 자리를 채울 뿐이다. 동봉은
`bindings/python/hatch_build.py` 훅이 수행하며, 환경변수
(`RHWP_WHEEL_BUNDLE`·`RHWP_WHEEL_TAG`)가 **둘 다** 있을 때만 개입한다(하나만
있으면 즉시 실패 — 태그 없는 동봉도, 동봉 없는 태그도 소비자를 속이는 휠이다).

## 2. 버전 정합 (version_policy.md §6)

태그 `vX.Y.Z` = `Cargo.toml` = 바인딩 패키지 버전. `tools/set_package_version.py`
가 두 역할을 한다:

- `--check` — version-gate 잡에서 태그와 `Cargo.toml` 불일치 시 전체 중단.
- 정렬 — 빌드 직전 pyproject·package.json 버전을 태그로 일시 정렬(커밋하지
  않음 — 저장소의 바인딩 버전은 개발 트리 표지이고, 배포 버전의 원천은 태그다).

## 3. 켜는 절차 (메인테이너)

1. **PyPI**: `rhwp` 패키지명 확보(첫 업로드가 곧 선점) → API 토큰 발급 → 저장소
   시크릿 `PYPI_API_TOKEN` 등록. 대안: PyPI **Trusted Publishing**(OIDC — 토큰
   없이 저장소·워크플로를 PyPI 쪽에 등록). 채택 시 publish-pypi 잡을
   `pypa/gh-action-pypi-publish` 로 바꾸는 후속 커밋이 필요하다.
2. **npm**: `@rhwp` 스코프는 이미 `@rhwp/core`·`@rhwp/editor` 게시로 확보돼
   있다 → automation 토큰 발급 → 시크릿 `NPM_TOKEN` 등록.
3. 다음 `v*` 태그부터 자동 게시된다. 첫 게시는 `workflow_dispatch` 로 기존
   태그를 지정해 수동으로 돌려보는 것을 권장한다.

## 4. 스모크가 증명하는 것

각 플랫폼 잡이 자기 휠을 실제로 `pip install` 한 뒤 **`RHWP_BIN` 없이**:

1. `find_binary()` 결과가 `_bin` 경로(동봉 2순위)인지 단언
2. 그 바이너리로 `--version` 실행

로컬 선행 실증(2026-08-09, Windows): 디버그 바이너리 동봉 휠을 깨끗한 venv 에
설치 → `site-packages/rhwp/_bin/rhwp.exe` 발견 → `rhwp v0.8.2` → 실물 문서
`rhwp.info()` 17쪽 파싱까지 확인.

## 5. 한계와 후속 (정직 목록)

- **manylinux 하한**: linux 휠은 빌드 러너(ubuntu-latest) glibc 기준
  `manylinux_2_39` 로 정직 표기 — 구형 배포판은 sdist+`RHWP_BIN` 경로.
  더 낮은 하한(musl/zig 크로스)은 수요 확인 후 별도 단계.
- **npm 바이너리 동봉 없음**: `@rhwp/node` 는 래퍼로 먼저 게시한다. esbuild 식
  플랫폼별 optionalDependencies 동봉과 `npx` MCP 원라인은 후속(R39 연계).
- **서명 없음**: 게시물 서명은 version_policy.md §8 의 도구 결정 후 이
  파이프라인에 얹는다(SHA256SUMS 와 병행).

## 6. 롤백

- PyPI: `yank` (설치 차단, 기록 보존) — 삭제보다 yank 를 기본으로.
- npm: 72시간 내 `npm unpublish`, 이후는 `npm deprecate`.
- 워크플로 자체를 끄려면 시크릿 제거(게시만 멈춤) 또는 워크플로 disable.
