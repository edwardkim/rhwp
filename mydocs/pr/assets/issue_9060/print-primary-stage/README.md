# Print 주 프로그램의 실제 소비 검증

> 이 문서는 당시 단계의 분석 기록이다. 최종 결과와 대표 증적은 [검증 기록](../verification-record.md)과 [최종 대상 증적](../final/main/README.md)을 따른다. 원시 로그·중간 분석 JSON·중간 PNG는 제출에서 제외했다.

소스 `43adf3f7fe412e71797456887e2be44b8ef8d14f`를 새로 최적화해 만든 WASM으로
실제 FontFace 공급과 Print glyph paint를 검증했다. [영수증](../verification-record.md)은
생산 SHA·package 해시·소스 유지·실행 명령과 자료를 연결한다. 글꼴 파일은 포함하지 않는다.

- [최적화 빌드](../verification-record.md): 종료 코드 0, 7분 47초.
- [실제 브라우저](../verification-record.md): 23/23 통과. [CSS 이름 해소 보정 전](../verification-record.md)은
  새 주 TTF·완전 CFF 결측 두 반례가 실패했다. 기대값 완화 없이 제품 파서를 보정했다.
- [Studio 전체 Node](../verification-record.md): 2,794/2,794 통과. 같은 Studio 구현의 결과를 재사용하며
  새 WASM의 브라우저 검증과 구분한다.
- [원 PR의 fresh WASM 전 14쪽의 검증 요약](../verification-record.md): 모두 공식 raw 100.00%.
- 보호 선행: [중첩 표 17쪽의 검증 요약](../verification-record.md), [글머리표 9쪽의 검증 요약](../verification-record.md),
  [그림 경계 1쪽의 검증 요약](../verification-record.md)은 모두 정본 대비 하락 0쪽.
- 중첩 표의 기존 하락 5쪽은 2쪽 99.97→99.98%, 10쪽 98.60→98.83%,
  11쪽 98.93→99.10%, 14쪽 98.47→98.70%, 15쪽 99.07→99.19%로 복구했다.
- [최소 글꼴 노출](../verification-record.md)은 저장소 공개 5개 자원만 보인다.
  시스템 두 글꼴 경로와 `RHWP_FONT_PATH`를 격리한 전체 Rust는 별도로 진행 중이다.

이 단계는 전체 보호 36개 데이터셋·전체 Rust·최종 Native 검사·원격 CI·PR 생성·머지의
완료 기록이 아니다. 일반 조판·저장 폭·Screen의 선택을 바꾸지 않고 실제 공급 프로그램의
Print 윤곽과 CSS 이름·빈 기호 제외 정책을 연결한다. 보호 TSV의 실제 상향은 최종 전쪽
검증이 모두 통과한 뒤 수행한다. 100.00%는 공식 2px 실루엣 표시값이며 RGB 동일성을 뜻하지 않는다.
