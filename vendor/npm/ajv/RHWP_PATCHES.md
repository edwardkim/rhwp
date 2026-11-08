# Ajv 8.20.0의 fast-uri 4 소비 패치 (#7856)

공식 npm `ajv@8.20.0` 배포본을 기준으로 한 빌드 도구 전용 패키지다. upstream 버전 번호와
MIT LICENSE·README·원 소스를 보존한다. 원본 tarball URL·integrity와 각 파일 SHA-256은
`UPSTREAM_FILES.json`에 있다. 원본에서 변경한 파일은 아래로 한정한다.

- `package.json`: `fast-uri` 의존을 검증 대상 4.2.2로 지정한다. 로컬 설치에 필요한 배포 파일과
  이 고지·원장만 포함하고, 배포본 소비에 필요 없는 개발 의존성·upstream build lifecycle은 제거한다.
- `lib/runtime/uri.ts`, `dist/runtime/uri.js`, `dist/runtime/uri.d.ts`: Ajv가 실제 사용하는
  `parse`·`resolve`·`serialize`를 명시적으로 연결한다. codegen용 `code`는 전용 어댑터 객체가
  소유하고 공용 fast-uri namespace에 쓰지 않는다. URI 오류를 삼키거나 구형 파싱으로 되돌리지 않는다.
- 변경 전 JS 위치를 가리키는 `dist/runtime/uri.js.map`은 제거한다.

Studio와 VS Code는 이 디렉터리를 직접 devDependency로 지정하고 Ajv 8 소비자만 `$ajv`
override로 같은 패키지를 사용한다. Ajv 6 소비자는 유지한다. 각 프로젝트의 `.npmrc`에서
`install-links=true`로 외부 디렉터리 심링크 대신 패키지를 설치한다. lockfile도 실제 4.2.2
의존과 로컬 출처를 기록하므로 `npm ci`로 재현되며 수동 node_modules 수정은 필요 없다.

`scripts/tests/frontend-dependency-compat.test.mjs`는 각 실제 설치 경로의 Ajv·fast-uri로
스키마·상대/절대/fragment 참조, URI escape/host/port 오류와 어댑터 소유권을 검증한다.
Studio/PWA 빌드와 VS Code compile·package는 해당 프로젝트의 실제 소비 경로로 별도 확인한다.
상세 실행 결과는 `mydocs/working/task_m100_7946_stage1.md`에 연결한다.
