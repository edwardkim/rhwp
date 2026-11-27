# Native C# end-to-end smoke

Run from the repository root with a Windows x64 .NET SDK and the Native Skia DLL built from the same checkout.

```powershell
cargo build --locked --release -p rhwp-native-ffi --features native-skia --target x86_64-pc-windows-msvc
$library = 'target\x86_64-pc-windows-msvc\release\rhwp_native_ffi.dll'
$document = 'samples\issue5599_oracle\3191107_leave_request_form.hwpx'
dotnet run --project bindings/csharp/tests/RhwpNativeSmoke.csproj -p:TargetFramework=net10.0 -- $library $document --live-pdf
```

Use the actual library path if Cargo uses a different target directory. With a .NET 9 SDK, omit the target-framework property. A Linux x64 smoke uses the `librhwp_native_ffi.so` path from the host build. Omit `--live-pdf` for a build without `native-skia`.

The smoke edits the applicant's name and birthday in the public leave-request form and reads back the child's original birthday. It exercises undo/redo, atomic IME, Print PDF on a dirty revision, HWP and HWPX publication and reopen, rejection of an old document ID, request cancellation, and the release of each managed worker handle. Native calls and buffer ownership stay on one OS thread. The program prints a JSON result with the OS and architecture and exits with code 1 if a case fails.
