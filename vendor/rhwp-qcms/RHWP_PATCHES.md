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
