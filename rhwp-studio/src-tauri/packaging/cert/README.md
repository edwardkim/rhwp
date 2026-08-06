# 인증서 배치 위치

이 폴더는 릴리스 빌드 시점에만 채워진다. 저장소에는 커밋하지 않는다(`.gitignore` 참고).

`scripts/windows-packaging/generate-self-signed-cert.ps1`로 생성한 공개 인증서
(`rhwp-studio-cert.cer`)를 `npm run tauri:build` 실행 전에 이 폴더에 복사한다.

```
rhwp-studio/src-tauri/packaging/cert/rhwp-studio-cert.cer
```

개인키(`.pfx`)는 이 폴더가 아니라 릴리스 담당자 로컬 또는 사내 비밀 저장소에만 보관한다 — 설치
파일에는 서명에만 쓰이고 포함되지 않는다.

자세한 절차는 `docs/windows-packaging/RELEASE_RUNBOOK.md`를 따른다.
