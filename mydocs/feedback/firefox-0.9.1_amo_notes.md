# Firefox AMO — 0.9.1 reviewer notes

상태: 제출 전 후보. 실제 패키지·source SHA·SHA-256과 배포 결과는 최종 포장 뒤 기록합니다.

- 0.9.1 엔진의 표·글꼴·저장 재열기·작은 화면 대화창 개선을 포함합니다.
- 문서는 브라우저 안의 WASM으로 처리합니다. HWP/HWPX 링크 감지·미리보기·컨텍스트 메뉴·로컬 파일 열기에 기존 match/host 권한을 사용합니다.
- v0.9.0과 실제 manifest를 대조했으며 버전 외 필드가 같습니다. 새 permissions·host_permissions·CSP 변경은 없습니다. 추가 외부 endpoint의 포장·실행 검증은 별도로 수행합니다.
- Chrome과 Edge는 같은 ZIP을 사용합니다. Firefox는 해당 ZIP과 재빌드 가능한 AMO source ZIP을 함께 제출합니다.
- PDF 가져오기는 CLI·WASM·Studio의 지원 계약을 따릅니다. 모든 웹 PDF를 자동으로 가로채는 확장이 아닙니다.
- 심사·공개 전환과 독립 설치 검증은 아직 완료되지 않았습니다.

[0.9.1 전체 변경과 기여자](../release/release_notes_v0.9.1.md)

## 검토용 source ZIP 재빌드

최종 source ZIP은 제품 source·workspace·vendor·lockfile·공개 재배포 자원만 포함합니다.
대형 샘플·개인 폰트·환경 비밀·빌드 산출물은 제외합니다. Node 24.15.0과 저장소
`rust-toolchain.toml`의 Rust 1.93.1·wasm32 target, wasm-pack 0.15.0을 사용합니다.
wasm-pack이 없다면 `cargo install wasm-pack --version 0.15.0 --locked`로 준비합니다.
다음 경로를 실제 재검증합니다.

```sh
cd rhwp-source
npm --prefix rhwp-studio ci
npm --prefix rhwp-firefox ci
sh scripts/wasm-pack-locked.sh --target web --release
npm --prefix rhwp-firefox run build
```

source ZIP hash와 실제 재빌드 완료·사용한 도구·생성 manifest는 포장 원장에 기록합니다.
위 명령을 제시한 것만으로 재빌드 통과나 AMO 승인 완료로 판정하지 않습니다.
