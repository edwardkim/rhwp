#!/usr/bin/env python3
"""#8639 변경 추적 설정 대조군: 원본의 HWPX 변환본에서 `trackchageConfig flags` 만 0 으로 바꾼다.

생성 절차(커밋한 `samples/issue8639/policy_research_review_request-trackchange0.hwpx`):

1. `rhwp export-hwpx samples/issue8639/policy_research_review_request.hwp exported.hwpx`
   (#8639 작업 head `cb1ab1d5af` 트리로 빌드한 rhwp). 원본 flags 는 56.
2. `python3 make_trackchange0_variant.py exported.hwpx out.hwpx`
   — `Contents/header.xml` 의 `<hh:trackchageConfig flags="56"/>` 만 `flags="0"` 으로 바꾸고,
   다른 파트는 바이트 그대로 다시 묶는다(`mimetype` 은 무압축 첫 항목).

같은 변환본(flags 56 그대로)은 한/글 2024 Print 에서 원본 HWP 와 같은 저장 줄을 쓰고,
flags 0 변형만 문단을 현재 글꼴로 다시 조판한다. 독립 Print:
`pdf/issue8639/policy_research_review_request-trackchange0-print-2024.pdf`
(MCP engine 2024, Hancom 13.0.0.3901).
"""
import sys
import zipfile


def main(src: str, dst: str) -> None:
    zin = zipfile.ZipFile(src)
    names = zin.namelist()
    if names[0] != "mimetype":
        raise SystemExit("mimetype 이 첫 항목이 아니다")
    out = zipfile.ZipFile(dst, "w")
    for name in names:
        data = zin.read(name)
        if name == "Contents/header.xml":
            text = data.decode("utf-8")
            old = '<hh:trackchageConfig flags="56"/>'
            if text.count(old) != 1:
                raise SystemExit("flags=56 trackchageConfig 가 정확히 하나가 아니다")
            data = text.replace(old, '<hh:trackchageConfig flags="0"/>').encode("utf-8")
        compress = zipfile.ZIP_STORED if name == "mimetype" else zipfile.ZIP_DEFLATED
        out.writestr(name, data, compress_type=compress)
    out.close()


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
