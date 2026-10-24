# Stage81: WASM Print 캡처의 Canvas 이력 분리

Issue: #7445. 부모 HEAD는 `6ec84f00a5a72f33af8e22b0d7daea50d68e193f`, base는
`d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`이다. 현재 기존 문서의 캡처 도구만 보정한다.
새 HWP 원본·PDF·글꼴·제품 renderer·DPI·비교 문턱은 추가하거나 변경하지 않았다.

## 원인 분리와 공통 캡처 경로

로드맵 4쪽은 2쪽을 워밍업한 Canvas와 8쪽을 워밍업한 Canvas에서 9,515픽셀 차이가 났다.
같은 Chrome, 같은 WASM·문서·렌더 트리·글꼴 목록과 등록 순서, 같은 context 속성임을 확인했다.
두 조건을 각각 반복해 같은 차이를 재현했다. Canvas의 width/height 재설정과
`willReadFrequently`만으로는 선행 paint 이력을 분리하지 못했다.

문서·글꼴·그림 준비를 그대로 둔 상태에서 실제 캡처 Canvas만 새로 만들면 짧은/긴 워밍업의
4쪽 PNG가 완전히 같았다. 이를 exporter의 모든 쪽에 공통 적용한다. 각 쪽의 새 Canvas에
최초 2D context의 반복 readback 속성을 적용·검증하며 해당 속성도 페이지 manifest에 기록한다.
같은 쪽의 비동기 그림 재생은 그 쪽 Canvas에서 기존 안정화 절차를 유지한다.
문서나 쪽 번호에 따른 분기, PNG 편집·마스킹, glyph/font 또는 레이아웃 변경은 없다.

## 실제 검증

- 기존 공개 Noto/FontFace 브라우저 검사에 워밍업 Canvas 소유 이력을 관측 가능한 adapter를
  넣었다. 재사용을 허용한 기존 도구에서 실제 1 FAIL(exit 1), 보정 뒤 1 PASS(exit 0)다.
  이 adapter의 상태 검사는 실물 Chrome 차이의 자동 탐지 계약이며 한컴 정확도 검사가 아니다.
- 관련 Node 서버·readback·원본 글꼴 공급 검사는 7 PASS / 0 FAIL이다.
- 같은 동결 fresh WASM과 기존 로드맵을 새 도구로 각각 16쪽 묶음·64쪽 묶음으로 출력했다.
  실제 128개 PNG의 64쌍 모두 디코딩 픽셀이 완전히 같고, 대응 렌더 트리도 모두 같다.
  앞서 이 보정 전의 같은 묶음 실험은 36/64쪽에서 차이가 있었다.
- 새 64쪽 TSV 최저는 96.66835%, 90% 미만 0쪽이다. 전체 302쪽 통과로 확대하지 않는다.
  입력/PDF·준비 글꼴·WASM 바이트와 기존 canonical 96DPI/2px/90% 조건을 유지했다.
- Native의 같은 묶음 실험은 기존 64쌍 모두 같았다. 이번 exporter는 WASM Canvas만
  바꾸므로 Native producer를 새로 빌드하거나 Native PNG를 새 출력이라고 표시하지 않는다.

현재 Rust/WASM source는 동결 `f1f52d57`과 같다. 사용한 pkg는 해당 source의 locked
`--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화 통과가 아니다.
실행별 source/tool/font/pkg hash·Canvas 조건과 전쪽 TSV는 ignored output의
`rebase-d9fd85ed-20261023/wasm-fresh-canvas-batch-verification/`에 보존했다.
수정 전 반복 분리·실패/성공 검사는 같은 상위의 `wasm-canvas-history-isolation/`에 있다.

## 기존 전쪽 실행의 보존과 남은 작업

기존 50조건 비교는 capture lock을 확보하고 활성 자식 캡처가 없는 배치 경계에서 종료했다.
완료 10조건과 부분 PNG/TSV/로그·producer는 보존했다. 이전 WASM 결과를 새 도구 출력으로
바꾸거나 혼합하지 않는다. 동일 Native의 재사용은 입력·PDF·producer·PNG/TSV hash와
실제 소비 경로의 비영향을 확인한 경우만 기록하며 새 WASM은 다시 산출한다.

C354의 JPEG 전달과 C356의 무손실 물리 프레임 보정, 최종 전체 문서 검증은 아직 필요하다.
이번 캡처 도구 검증은 제품 보정 완료나 PR 제출 준비 완료가 아니다. 지침과 원격은 바꾸지 않았다.
