# 서드파티 고지

이 문서는 개발·CI 분석 도구의 서드파티 고지를 보완한다. 기존 라이브러리·리소스의
라이선스와 저작권 고지는 [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md)를 함께 확인한다.

## 개발·CI 분석 도구

### eslint-plugin-sonarjs 4.1.0

| 항목 | 내용 |
| --- | --- |
| 패키지 | `eslint-plugin-sonarjs` |
| 버전 | `4.1.0` |
| 선언 라이선스 | `LGPL-3.0-only` — npm 메타데이터, 배포본 `package.json`, RHWP lockfile의 표기 |
| 배포본 라이선스 파일 | `SONAR Source-Available License v1.0` — 배포본 `package/LICENSE`의 표기 |
| 공식 업스트림 | [SonarSource/SonarJS](https://github.com/SonarSource/SonarJS) |
| 공식 배포본 | [npm registry의 4.1.0 archive](https://registry.npmjs.org/eslint-plugin-sonarjs/-/eslint-plugin-sonarjs-4.1.0.tgz) |
| 라이선스 원문 | [배포 메타데이터의 gitHead에 해당하는 업스트림 LICENSE.txt](https://github.com/SonarSource/SonarJS/blob/4ce51a6eec04a87ee57b30792c3103823aad5bdb/LICENSE.txt) |

#### 사용 범위

`private: true`인 `@rhwp/frontend-metrics-tool`이
[scripts/frontend-metrics/package.json](scripts/frontend-metrics/package.json)에 직접
`devDependency`로 선언한다. [lockfile](scripts/frontend-metrics/package-lock.json)에도
버전 `4.1.0`, `dev: true`, 라이선스 `LGPL-3.0-only`로 기록되어 있다.
[scripts/frontend-metrics.mjs](scripts/frontend-metrics.mjs)는
`sonarjs/cognitive-complexity` 규칙으로 프런트엔드 인지 복잡도를 측정하며,
`devel` CI가 이 분석 도구를 설치·실행한다.

현재 확인한 제품 패키징 설정에서는 이 분석 도구를 제품 런타임에 포함하는 경로를
찾지 못했다. 실제 배포 ZIP·VSIX·npm tarball을 독립 검사한 결과는 아니므로 모든 배포
산출물의 미포함을 보증하지 않는다.

#### 라이선스 표기 불일치

2026-10-26 확인한 [공식 4.1.0 메타데이터](https://registry.npmjs.org/eslint-plugin-sonarjs/4.1.0)의
`license`는 `LGPL-3.0-only`이다. 그러나 공식 archive의 `package/LICENSE`는 위 표의
`SONAR Source-Available License v1.0` 원문이며, 메타데이터의 `gitHead`에 해당하는
업스트림 `LICENSE.txt`와 바이트 단위로 일치한다. 확인한 archive의 SHA-512는 RHWP
lockfile의 `integrity`와 일치한다. 선언 라이선스와 동봉 원문의 불일치를 그대로 기록하며,
이 고지를 적용 라이선스의 확정이나 법적 의무 충족의 판정으로 해석하지 않는다.
