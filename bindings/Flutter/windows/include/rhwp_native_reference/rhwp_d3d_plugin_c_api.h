#ifndef RHWP_WINDOWS_D3D_PLUGIN_C_API_H_
#define RHWP_WINDOWS_D3D_PLUGIN_C_API_H_
#include <flutter_plugin_registrar.h>
#ifdef _WIN32
#ifdef RHWP_NATIVE_REFERENCE_PLUGIN_IMPL
#define RHWP_PLUGIN_EXPORT __declspec(dllexport)
#else
#define RHWP_PLUGIN_EXPORT __declspec(dllimport)
#endif
#else
#define RHWP_PLUGIN_EXPORT __attribute__((visibility("default")))
#endif
#ifdef __cplusplus
extern "C" {
#endif
RHWP_PLUGIN_EXPORT void RhwpD3dPluginRegisterWithRegistrar(FlutterDesktopPluginRegistrarRef registrar);
#ifdef __cplusplus
}
#endif
#endif
