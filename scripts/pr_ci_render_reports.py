"""Read the sole bounded evidence.json member. Never extract PR files to disk."""
import io
import sys
import zipfile


def main():
    with zipfile.ZipFile(io.BytesIO(sys.stdin.buffer.read())) as archive:
        entries = archive.infolist()
        if len(entries) != 1:
            return 1
        entry = entries[0]
        if entry.filename != 'evidence.json' or entry.file_size > 2 * 1024 * 1024:
            return 1
        sys.stdout.buffer.write(archive.read(entry))
    return 0


if __name__ == '__main__':
    sys.exit(main())
