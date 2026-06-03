# Task #354 Stage 1 — execSync → execFileSync 전환 + 빌드 검증

## 변경

`rhwp-firefox/build.mjs`:

```diff
-import { execSync } from 'child_process';
+import { execFileSync } from 'child_process';

-function run(cmd, cwd = __dirname) {
-  console.log(`> ${cmd}`);
-  execSync(cmd, { stdio: 'inherit', cwd });
-}
+// [Task #354] execSync (shell 보간) → execFileSync (인자 배열, shell:false) 전환.
+// CodeQL 경고 (js/shell-command-injection-from-environment) 해소 + 공백 포함
+// 경로에서도 인자 배열이 안전하게 처리됨.
+function run(cmd, args, cwd = __dirname) {
+  console.log(`> ${cmd} ${args.join(' ')}`);
+  execFileSync(cmd, args, { stdio: 'inherit', cwd, shell: false });
+}

-run(`npx vite build --config ${resolve(__dirname, 'vite.config.ts')}`, studioDir);
+run('npx', ['vite', 'build', '--config', resolve(__dirname, 'vite.config.ts')], studioDir);
```

## 검증

| 항목 | 결과 |
|------|------|
| `npm run build` | ✅ Vite 빌드 + 폰트 복사 + manifest 복사 정상 통과 |
| dist `manifest.json` version | 0.2.2 ✅ |
| dist `manifest.json` strict_min_version | 142.0 ✅ |
| dist 워닝 패턴 (innerHTML / Function / document.write) | wasm/rhwp.js 1건 (wasm-bindgen 표준, 변경 없음) ✅ |
| `web-ext lint --source-dir=rhwp-firefox/dist` | errors 0, warnings 2 (이전과 동일) ✅ |

## Stage 1 완료 조건 점검

- [x] 코드 변경 (1 import + 1 함수 + 1 호출부)
- [x] 빌드 정상 통과
- [x] dist 결과물 (`manifest.json` 등) 변경 전과 동일 메타
- [x] `web-ext lint` errors 0 유지

## 다음 단계

머지 후 GitHub CodeQL 자동 재실행으로 Alert #17 자동 close 기대 (Stage 3).
