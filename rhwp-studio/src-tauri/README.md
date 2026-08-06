# rhwp-studio Tauri 셸

`rhwp-studio`(Vite + WASM 웹 에디터)를 Windows 데스크톱 앱으로 감싸는 Tauri 프로젝트다. 새 UI 코드를
두지 않는다 — 창을 띄우고 `../dist`를 로드하는 최소 셸이다.

배경과 설계는 [Windows 패키징 설계 스펙](../../docs/superpowers/specs/2026-08-06-rhwp-windows-packaging-design.md),
릴리스 절차는 [RELEASE_RUNBOOK.md](../../docs/windows-packaging/RELEASE_RUNBOOK.md)를 참고한다.

## 로컬에서 실행

사전 준비(Rust, Node, wasm-pack)는 저장소 루트 `mydocs/manual/onboarding_guide.md`를 따른다.

```bash
# 저장소 루트에서
wasm-pack build --target web --out-dir pkg

# rhwp-studio/ 에서
npm ci
npm run tauri:dev
```

`tauri:dev`는 Vite dev 서버(`npm run dev`, 포트 7700)를 띄우고 그 위에 네이티브 창을 연다. 웹 브라우저
버전과 동일한 UI가 뜬다.

## 이 crate가 독립 workspace인 이유

`src-tauri/Cargo.toml`은 저장소 루트 `Cargo.toml`의 workspace에 편입되지 않도록 자체 `[workspace]`
빈 테이블을 갖는다. 편입되면 루트 `cargo build`/`cargo test`가 이 crate까지 함께 빌드하려 하면서
불필요하게 느려지고, 루트 workspace 의존성 해석과 충돌할 수 있다.

## 실제 설치 파일(.exe) 빌드는 Windows에서만

`bundle.targets`가 `nsis`로 고정돼 있어 `npm run tauri:build`는 Windows에서만 의미 있는 산출물을
만든다. macOS/Linux에서는 `tauri:dev`로 셸 동작만 검증한다.
