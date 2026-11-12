# hyunmyungJaneLee 6개 PR 통합 검토 증적

브랜치 `review/integration-20261112-hyunmyungjanelee`, 기준 `80b1c354c29b7a02fb2b3c14a274da7f5b653ada`, 검증한 code head `0c9819d698bcbe3e3d83f563a0154ccb00eb6f07`. 원 contributor의 29개 커밋을 순차 체리픽했으며 원 브랜치는 변경하지 않았다.

## 완료한 검증

- 공개 28종·37쪽 Native/fresh WASM: 전쪽 90% 이상, 최저 93.02%/92.33%. [Native base/후보](native-comparison.tsv), [WASM 후보](wasm-candidate.tsv). Native는 devel 대비 2%p 초과 하락 0쪽. devel fresh WASM 비교는 아직 미실행이다.
- 같은 입력 자체를 웹한글기안기로 Print한 정식 PDF를 재사용했다. Git blob 일치·전쪽 쪽수·페이지 번호·PNG 해시를 확인했다([Native](native-validation.json), [WASM](wasm-validation.json)).
- 최소 글꼴 집중 97 PASS / 0 FAIL, [전체 회귀](full-nextest-result.json) 13,347 PASS / 0 FAIL / 45 skip. CoreText 528 OS 글꼴만 노출하고 일반 사용자 글꼴 목록은 유지했다([격리 증거](minimal-font-focused-result.json)).
- [Rust lint/build/Native Skia 3종/manifest](rust-gates.json) 모두 PASS. WASM은 Mac locked wrapper `--no-opt` 로컬 대체 빌드이며 Docker 최적화 결과가 아니다.
- [실제 브라우저 초기화 응답](browser-runtime-result.json)의 JS·WASM은 root pkg·Studio public·동결 package와 바이트 해시가 같다.
- [편람 HWPX Native gate](handbook-native-gate.json) 전383쪽 PASS, 공식 보호값 하락 0쪽.

## 증적과 남은 검증

각 PR 디렉터리에 그 PR 입력의 같은 쪽 raw·PDF·review·overlay PNG와 전쪽 TSV를 함께 보관한다. [실제 판독한 대표 쪽](native-direct-reading.json)과 전쪽 수치의 범위는 구분한다. 과거 head의 점수를 현재 통과로 인용하지 않는다.

devel fresh WASM 비교와 정본 보호 Native/fresh WASM 전쪽 판정은 진행 중이다. 중첩 표의 초기 공급 조건 누락과 편람 HWP145쪽은 같은 조건의 devel/후보 PNG가 동일함을 확인했다. 올바른 수용 환경·일반 공급 조건의 문서 전체로 재검증하며, 기준값이나 쪽별 최댓값을 이어 붙이지 않는다.

#8897의 미공개 그림 전용 칸3건은 공개16종 통과로 해소됐다고 판단하지 않는다. #8900의 #8678은 일부 해결 범위여서 전체 종료 대상으로 삼지 않는다. #8895의 기존 도장 차이는 원 메인터너 댓글의 제한된 수용 범위를 유지한다. 모든 필수 검증과 해당 보류 사유 해소 전에 승인·병합하지 않는다.
