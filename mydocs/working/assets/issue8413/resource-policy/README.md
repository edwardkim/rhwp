# 확대 자원 정책의 8K 제어군 시각 증거

2026-11-01, 96dpi. 공개 합성 원본은 [resource-8k.pdf](../../../../../pdf/issue8413/resource-policy/resource-8k.pdf)다.
Native CLI와 fresh WASM으로 각각 가져와 저장한 HWPX를 해당 backend로 출력했다.
전체 두 쪽의 `silhouette_raw_match_percent`는 Native/fresh WASM 모두 100.00%이며
회색 그림의 경계·여백을 직접 대조했다. 글꼴이나 문자는 없는 제어군이다.
사용자 원본 127쪽의 완료 증거로 대신하지 않는다.

검증 소스는 `c9ffaa47462fd1237e141ef6007d61b209cdaa8c` 이후 자원 정책 변경이며
제품 diff SHA-256은 `f98d3c002a22df343e97dc850131a6b806a35ae964e6242ac841796ba2e1a9e5`다.
최적화 WASM SHA-256은 `048d8d735cd47c7df3d89a1ab3606085bbee19ada6a2e76b205aa13f176a033f`다.
Studio의 실제 브라우저가 읽은 바이너리와 같은 해시를 확인했다.
