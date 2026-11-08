#ifndef RHWP_NATIVE_FFI_H
#define RHWP_NATIVE_FFI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

char *rhwp_export_text(const char *input_path, const char *output_dir, int page);
char *rhwp_export_markdown(const char *input_path, const char *output_dir, int page);
/* Print-profile direct PDF; page=-1 means all, otherwise zero-based.
 * Input <=128 MiB, selected pages <=256. Destination must not exist.
 * All pages are rendered before creating the file. Requires native-skia;
 * valid arguments in disabled builds return {"ok":false,"code":"featureDisabled",...}.
 * Free the returned UTF-8 JSON with rhwp_string_free. */
char *rhwp_export_pdf_direct(const char *input_path, const char *output_file, int page);
char *rhwp_read_text(const char *input_path, int page);
/* Native host ABI v1. All handles belong to the calling OS thread.
 * All returned JSON strings use rhwp_string_free. Inputs are borrowed only
 * during the call; nonempty byte spans must point to readable memory.
 * Empty document bytes create a blank document. Close documents explicitly.
 * Output buffers have independent lifetime and must be released separately. */
char *rhwp_session_capabilities(void);
/* Host JSON operations; data is borrowed for binary-payload operations only. */
char *rhwp_native_host_request(const uint8_t *request, size_t request_len, const uint8_t *data, size_t data_len);
char *rhwp_session_open(const uint8_t *data, size_t len, const uint8_t *password, size_t password_len);
char *rhwp_session_request(uint64_t handle, const uint8_t *request, size_t request_len);
char *rhwp_session_close(uint64_t handle);
/* destination must be writable for capacity bytes. Too-small destinations
 * are rejected before copying. This call does not release the source buffer. */
char *rhwp_session_buffer_copy(uint64_t buffer, uint8_t *destination, size_t capacity);
char *rhwp_session_buffer_release(uint64_t buffer);
/* Presentation ABI v1 is the explicit cross-thread exception: snapshot,
 * CPU frame producer, and consumer lease handles are process-wide. Legacy
 * session/worker/buffer handles above still belong to their owner OS thread.
 * Configure a live worker source through owner-thread host operation
 * presentationConfigure, then submit exportRenderSnapshot and poll it there.
 * rhwp_presentation_request accepts snapshotInfo/snapshotCheck/snapshotRelease,
 * renderCpu/renderCpuBatch,
 * frameInfo/frameAcquire/frameRelease, leaseInfo/leaseRelease, and stats JSON.
 * renderCpu/renderCpuBatch are synchronous work for a render thread; acquisition
 * and data access never implicitly render. This registry exposes CPU frames.
 * snapshotCheck compares the full source/view/device/DPR stamp without replay;
 * snapshotInfo.current checks document/revision/font identity only.
 * Every returned JSON string needs rhwp_string_free, on any OS thread. */
char *rhwp_presentation_request(const uint8_t *request, size_t request_len);
/* Immutable CPU bytes stay valid until this specific consumer lease is released,
 * including after producer frame release or source close/staleness. New stale
 * acquisitions are rejected. The caller coordinates pointer use versus release;
 * never release concurrently, write the bytes, or unload the library while a
 * lease/call/worker is live. byte_length must point to writable size_t storage.
 * Invalid/released lease: NULL and length=0. NULL byte_length: NULL. CPU lease
 * return does not establish GPU sampling completion or allow GPU target reuse. */
const uint8_t *rhwp_presentation_lease_data(uint64_t lease, size_t *byte_length);
/* Opt-in Linux GL/Ganesh host-FBO adapter. GPU presenter handles belong to the
 * creating OS thread and are not P89 cross-thread presentation handles.
 * Loader is called only during create/recreate, with that context current.
 * is_current is called before render/clearCaches/retire/release and returns exactly 1 only
 * for that current context identity. Callbacks must not unwind/reenter this ABI.
 * Host keeps callback function pointers, user_data, and GL procedures alive
 * through release/abandon. Drop/abandon never calls is_current or GL.
 * Empty config uses bounded defaults. Every JSON string uses rhwp_string_free.
 * CPU-only/non-Linux builds return featureDisabled without calling callbacks. */
typedef const void *(*RhwpGlGetProcAddress)(void *user_data, const char *name);
typedef int32_t (*RhwpGlIsCurrent)(void *user_data, uint64_t context_id);
char *rhwp_gl_presenter_create(uint64_t context_id, uint64_t device_generation,
    RhwpGlGetProcAddress get_proc_address, RhwpGlIsCurrent is_current,
    void *user_data, const uint8_t *config, size_t config_len);
/* Explicit same-handle recovery on a genuinely new matching current context and
 * strictly newer device generation than the last attempt. No automatic retry.
 * Empty config keeps the previous limits. Keep old callbacks alive until success
 * or abandon; success transfers their lifetime to the new callbacks/user_data.
 * Process context credits count live+unproven orphan contexts (max 4). Normal
 * current-context retire/release returns one credit; abandon/Drop does not.
 * Credit exhaustion rejects new native GPU construction, with no forgiveness.
 * Failed admitted recovery leaves the TLS handle in recoveryFailed state. */
char *rhwp_gl_presenter_recreate(uint64_t presenter, uint64_t context_id, uint64_t device_generation,
    RhwpGlGetProcAddress get_proc_address, RhwpGlIsCurrent is_current,
    void *user_data, const uint8_t *config, size_t config_len);
/* info/render/clearCaches/retire/release/abandon JSON. Host owns current-context creation, target
 * validity/format/origin/size, GL state restoration, swap, and window present.
 * All pages replay strictly to GPU scratch before one host-FBO draw/submit.
 * Late resource errors preserve the previous host pixels; a GPU/device failure
 * after publication begins cannot roll those pixels back. No whole-frame CPU
 * readback, external texture export, or cross-context fence is provided.
 * release requires the matching current context and generation. Context loss
 * uses abandon. clearCaches also requires that context/generation current.
 * retire performs proven current-context resource cleanup but keeps the handle
 * for later recreate or abandon; info exposes recovery state and process credits.
 * Typed errors distinguish stale/document/resource/context/device outcomes.
 * deviceFailure=true is reserved for Skia-reported device loss; generic
 * allocation/submission failure does not establish a device-loss diagnosis.
 * TLS owner teardown also abandons before native objects drop. */
char *rhwp_gl_presenter_request(uint64_t presenter, const uint8_t *request, size_t request_len);
/* Optional macOS Metal/Ganesh producer ABI v1. Capability probe always exists;
 * disabled builds return featureDisabled without dereferencing any pointers.
 * Device/queue/texture are typed borrowed Objective-C pointers, NEVER JSON
 * numbers. Device and queue match; Ganesh retains them. Create/render/request
 * handles belong to one dedicated producer OS thread, not the Flutter UI thread.
 * Every string is released with rhwp_string_free. Config/request <=64 KiB. */
char *rhwp_metal_capabilities(void);
char *rhwp_metal_presenter_create(const void *device, const void *command_queue,
    uint64_t device_generation, const uint8_t *config, size_t config_len);
/* Borrowed texture: matching device, BGRA8Unorm, 2D, single sample, shader-read
 * and render-target usage. Render JSON contains stamp, target{width,height,
 * deviceGeneration}, placements, clearColor. Caller keeps target exclusively
 * writable and retained for the call. Strict scratch replay precedes target
 * write. Success waits for PRODUCER GPU completion, with no CPU readback;
 * Flutter CONSUMER completion/reuse permission is not provided. Do not publish
 * failure/stale results or recycle a published texture without proven consumer
 * GPU completion. Source/device generations are checked before and after GPU
 * wait. Request accepts info/release/abandon on the same producer thread. */
char *rhwp_metal_presenter_render(uint64_t presenter, const void *texture,
    const uint8_t *request, size_t request_len);
char *rhwp_metal_presenter_request(uint64_t presenter, const uint8_t *request, size_t request_len);
/* Bounded cross-thread {stamp,snapshots:[1..8 IDs]} check. No presenter/TLS/
 * layout/replay is accessed. Handles remain alive for this call; checks include
 * source document/revision/font AND configured view/device generations.
 * Success returns {ok:true,current:true}; stale/error means do not acquire.
 * This does not freeze future source changes or establish a GPU fence. */
char *rhwp_metal_frame_check(const uint8_t *request, size_t request_len);
/* Optional Windows D3D12/Ganesh producer ABI. These typed COM pointers are C
 * arguments only; never encode pointer integers in JSON. The symbols exist in
 * disabled/non-Windows builds and return featureDisabled without reading spans.
 * Returned strings use rhwp_string_free on their calling thread. */
struct IDXGIAdapter1;
struct ID3D12Device;
struct ID3D12CommandQueue;
struct ID3D12Resource;
char *rhwp_d3d_capabilities(void);
char *rhwp_d3d_presenter_create(const struct IDXGIAdapter1 *adapter,
    const struct ID3D12Device *device, const struct ID3D12CommandQueue *queue,
    uint64_t device_generation, const uint8_t *config, size_t config_len);
/* resource is a fresh shared DEFAULT-heap BGRA8 2D target, single sample/mip/layer,
 * ALLOW_RENDER_TARGET|ALLOW_SIMULTANEOUS_ACCESS, initially COMMON, same device.
 * Success proves producer GPU completion and COMMON decay, never Flutter GPU
 * consumption. A D3D12 pointer is NOT a Flutter D3D11/DXGI surface descriptor.
 * Failure without unpublishedTargetSafeToDiscard:true requires quarantine. */
char *rhwp_d3d_presenter_render(uint64_t presenter,
    const struct ID3D12Resource *resource, const uint8_t *request, size_t request_len);
/* Owner-thread info/release/abandon. Unregister/descriptor release callbacks do
 * not establish consumer GPU completion or permit immutable target recycling. */
char *rhwp_d3d_presenter_request(uint64_t presenter, const uint8_t *request, size_t request_len);
/* Cross-thread full stamp/P89 snapshot validation; no Core/layout/GPU work. */
char *rhwp_d3d_frame_check(const uint8_t *request, size_t request_len);
void rhwp_string_free(char *value);

#ifdef __cplusplus
}
#endif

#endif
