# rhwp ICC CMM fork

- Upstream: qcms 0.3.0, https://github.com/FirefoxGraphics/qcms (MIT).
- Archive SHA-256: `edecfcd5d755a5e5d98e24cf43113e7cdaec5a070edd0f6b250c03a573da30fa`.
- `upstream/Cargo.toml.orig`, `UPSTREAM_FILES.json`, `COPYING` preserve origin and notices.
- Profile-dependent tag, curve and CLUT allocations, their copies and transform work consume the originating synchronous request through `rhwp-codec-resource`. Failures return `None`; the request boundary checks the sticky error before accepting fallback pixels.
- Fixed 256-entry gamma tables and fixed profile/matrix structs retain upstream storage. This is not a process heap guarantee.
- The standalone C bindings are outside the rhwp import path and are not enabled.
- This internal package is not an upstream release and is not published.

- Upstream crate VCS commit: `5ba6ed804b59273afc37794d5b04d27fb8835e48`. COPYING source is the same commit.

`LICENSE`는 원본 `COPYING`과 바이트가 같은 배포용 이름이다. 기존 source 사전 검사의
라이선스 파일 탐색 계약을 충족하며 원본 `COPYING`·출처·해시는 함께 보존한다.

## C bindings 테스트의 포인터·자원 실패 경계 (2026-11-09)

- 테스트의 일곱 transform 역참조는 nullable FFI 반환값을 검사한 reference로 바꾼다.
  미초기화 transform은 변환 함수를 호출하기 전에 명확한 테스트 실패가 된다.
- 기존 요청 자원 한도 보정에서 LUT helper가 `Option`을 반환하도록 바뀐 계약을
  선택 기능 `c_bindings`의 `qcms_profile_get_lut`에도 연결한다. 실패 시 sticky budget 오류를
  유지하고 호출자의 출력 배열을 덮어쓰거나 C ABI를 통해 unwind하지 않는다.
- C bindings는 rhwp의 기본 import 경로에 새로 활성화하지 않는다. 생산용 색 변환 계산과
  기존 출력 기대값은 변경하지 않으며, null 변환과 예산 실패·정상 LUT를 실제 경로로 검증한다.
