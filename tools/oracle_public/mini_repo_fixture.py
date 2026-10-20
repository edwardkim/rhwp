"""도구 계약 검사용 임시 트리. 실제 HWP/HWPX/PDF를 읽지 않는다."""

from pathlib import Path


def write_mini_tree(root: Path) -> None:
    files = [
        "samples/document_kor.hwp",
        "samples/document_kor.hwpx",
        "samples/lonely.hwp",
        "samples/편람.hwp",
        "samples/편람.hwpx",
        "samples/hwp3-sample.hwp",
        "samples/hwp3-sample-hwpx.hwpx",
        "samples/3-09월_회의_자료_2022.hwp",
        "samples/basic/calendar_year.hwp",
        "samples/multi.hwp",
        "samples/readme.txt",
        "pdf/document_kor-2022.pdf",
        "pdf/document_kor-2020.pdf",
        "pdf/편람-hwp-2020.pdf",
        "pdf/편람-hwpx-2020.pdf",
        "pdf/hwp3-sample-hwpx-2022.pdf",
        "pdf/3-09월_회의_자료_2022.pdf",
        "pdf/basic/calendar_year-2022.pdf",
        "pdf/multi-2018.pdf",
        "pdf/multi-2024.pdf",
        "pdf/unused-2022.pdf",
        "pdf/hwpx/lonely.pdf",
    ]
    for rel in files:
        path = root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("placeholder\n", encoding="utf-8", newline="\n")
