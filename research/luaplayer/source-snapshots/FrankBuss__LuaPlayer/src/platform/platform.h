/*
 * Platform abstraction layer for LuaPlayer
 *
 * This is the ONLY header that LuaPlayer source files should include
 * for platform-specific functionality. It provides:
 * - Basic types (u8, u16, u32, Color, etc.)
 * - Screen constants
 * - Controller input
 * - Graphics/GU functions
 * - System functions
 */

#ifndef PLATFORM_H
#define PLATFORM_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * Basic types
 */
typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;

typedef int8_t s8;
typedef int16_t s16;
typedef int32_t s32;
typedef int64_t s64;

typedef u32 Color;
typedef unsigned int SceUID;
typedef unsigned int SceSize;
typedef int SceMode;
typedef unsigned int SceUInt;

#ifndef TRUE
#define TRUE 1
#endif
#ifndef FALSE
#define FALSE 0
#endif
typedef int BOOL;

/*
 * Color macros (ABGR format)
 */
#define COLOR_A(color) ((u8)(((color) >> 24) & 0xFF))
#define COLOR_B(color) ((u8)(((color) >> 16) & 0xFF))
#define COLOR_G(color) ((u8)(((color) >> 8) & 0xFF))
#define COLOR_R(color) ((u8)((color) & 0xFF))
#define COLOR_RGBA(r, g, b, a) ((Color)(((a) << 24) | ((b) << 16) | ((g) << 8) | (r)))

/*
 * Screen constants
 */
#define PLATFORM_SCREEN_WIDTH  480
#define PLATFORM_SCREEN_HEIGHT 272
#define PLATFORM_LINE_SIZE     512

/*
 * Controller button masks (generic names, PSP-compatible values)
 */
#define CTRL_SELECT    0x000001
#define CTRL_START     0x000008
#define CTRL_UP        0x000010
#define CTRL_RIGHT     0x000020
#define CTRL_DOWN      0x000040
#define CTRL_LEFT      0x000080
#define CTRL_LTRIGGER  0x000100
#define CTRL_RTRIGGER  0x000200
#define CTRL_TRIANGLE  0x001000
#define CTRL_CIRCLE    0x002000
#define CTRL_CROSS     0x004000
#define CTRL_SQUARE    0x008000
#define CTRL_HOME      0x010000
#define CTRL_HOLD      0x020000
#define CTRL_NOTE      0x800000

/* Analog mode constant */
#define CTRL_MODE_ANALOG 1

/*
 * Controller data structure
 */
typedef struct CtrlData {
    unsigned int TimeStamp;
    unsigned int Buttons;
    unsigned char Lx;
    unsigned char Ly;
    unsigned char Rsrv[6];
} CtrlData;

/*
 * Directory entry structures
 */
#define FIO_SO_IFDIR  0x0010
#define FIO_SO_IFREG  0x0020
#define FIO_SO_IFLNK  0x0008

typedef struct SceIoStat {
    u32 st_mode;
    u32 st_attr;
    u64 st_size;
} SceIoStat;

typedef struct SceIoDirent {
    SceIoStat d_stat;
    char d_name[256];
} SceIoDirent;

/*
 * 3D Vector types
 */
typedef struct ScePspFVector3 {
    float x, y, z;
} ScePspFVector3;

typedef struct ScePspFMatrix4 {
    ScePspFVector3 x, y, z, w;
} ScePspFMatrix4;

/*
 * GE Context
 */
typedef struct PspGeContext {
    unsigned int context[512];
} PspGeContext;

/*
 * Module/thread option structures (unused on desktop)
 */
typedef struct SceKernelLMOption SceKernelLMOption;
typedef struct SceKernelSMOption SceKernelSMOption;

/*
 * Serial I/O constants
 */
#define SIO_IOCTL_SET_BAUD_RATE 1

/*
 * GU primitive types
 */
#define GU_POINTS           0
#define GU_LINES            1
#define GU_LINE_STRIP       2
#define GU_TRIANGLES        3
#define GU_TRIANGLE_STRIP   4
#define GU_TRIANGLE_FAN     5
#define GU_SPRITES          6

/*
 * GU states
 */
#define GU_ALPHA_TEST           0
#define GU_DEPTH_TEST           1
#define GU_SCISSOR_TEST         2
#define GU_STENCIL_TEST         3
#define GU_BLEND                4
#define GU_CULL_FACE            5
#define GU_DITHER               6
#define GU_FOG                  7
#define GU_CLIP_PLANES          8
#define GU_TEXTURE_2D           9
#define GU_LIGHTING             10
#define GU_LIGHT0               11
#define GU_LIGHT1               12
#define GU_LIGHT2               13
#define GU_LIGHT3               14
#define GU_LINE_SMOOTH          15
#define GU_PATCH_CULL_FACE      16
#define GU_COLOR_TEST           17
#define GU_COLOR_LOGIC_OP       18
#define GU_FACE_NORMAL_REVERSE  19
#define GU_PATCH_FACE           20
#define GU_FRAGMENT_2X          21

/*
 * GU clear flags
 */
#define GU_COLOR_BUFFER_BIT     1
#define GU_STENCIL_BUFFER_BIT   2
#define GU_DEPTH_BUFFER_BIT     4

/*
 * GU texture formats
 */
#define GU_PSM_5650     0
#define GU_PSM_5551     1
#define GU_PSM_4444     2
#define GU_PSM_8888     3
#define GU_PSM_T4       4
#define GU_PSM_T8       5
#define GU_PSM_T16      6
#define GU_PSM_T32      7

/*
 * GU vertex declarations
 */
#define GU_TEXTURE_8BIT     (1<<0)
#define GU_TEXTURE_16BIT    (2<<0)
#define GU_TEXTURE_32BITF   (3<<0)
#define GU_TEXTURE_BITS     (3<<0)
#define GU_COLOR_RES1       (1<<2)
#define GU_COLOR_RES2       (2<<2)
#define GU_COLOR_RES3       (3<<2)
#define GU_COLOR_5650       (4<<2)
#define GU_COLOR_5551       (5<<2)
#define GU_COLOR_4444       (6<<2)
#define GU_COLOR_8888       (7<<2)
#define GU_COLOR_BITS       (7<<2)
#define GU_NORMAL_8BIT      (1<<5)
#define GU_NORMAL_16BIT     (2<<5)
#define GU_NORMAL_32BITF    (3<<5)
#define GU_NORMAL_BITS      (3<<5)
#define GU_VERTEX_8BIT      (1<<7)
#define GU_VERTEX_16BIT     (2<<7)
#define GU_VERTEX_32BITF    (3<<7)
#define GU_VERTEX_BITS      (3<<7)
#define GU_WEIGHT_8BIT      (1<<9)
#define GU_WEIGHT_16BIT     (2<<9)
#define GU_WEIGHT_32BITF    (3<<9)
#define GU_WEIGHT_BITS      (3<<9)
#define GU_INDEX_8BIT       (1<<11)
#define GU_INDEX_16BIT      (2<<11)
#define GU_INDEX_BITS       (3<<11)
#define GU_WEIGHTS_BITS     (7<<14)
#define GU_VERTICES_BITS    (7<<18)
#define GU_TRANSFORM_3D     (0<<23)
#define GU_TRANSFORM_2D     (1<<23)
#define GU_TRANSFORM_BITS   (1<<23)

/*
 * GU matrix modes
 */
#define GU_PROJECTION   0
#define GU_VIEW         1
#define GU_MODEL        2
#define GU_TEXTURE      3

/*
 * GU light types
 */
#define GU_DIRECTIONAL          0
#define GU_POINTLIGHT           1
#define GU_SPOTLIGHT            2
#define GU_AMBIENT              1
#define GU_DIFFUSE              2
#define GU_SPECULAR             4
#define GU_AMBIENT_AND_DIFFUSE  3
#define GU_DIFFUSE_AND_SPECULAR 6

/*
 * GU comparison functions
 */
#define GU_NEVER    0
#define GU_ALWAYS   1
#define GU_EQUAL    2
#define GU_NOTEQUAL 3
#define GU_LESS     4
#define GU_LEQUAL   5
#define GU_GREATER  6
#define GU_GEQUAL   7

/*
 * GU front face
 */
#define GU_CW   0
#define GU_CCW  1

/*
 * GU shade model
 */
#define GU_FLAT   0
#define GU_SMOOTH 1

/*
 * GU logic operations
 */
#define GU_CLEAR          0
#define GU_AND            1
#define GU_AND_REVERSE    2
#define GU_COPY           3
#define GU_AND_INVERTED   4
#define GU_NOOP           5
#define GU_XOR            6
#define GU_OR             7
#define GU_NOR            8
#define GU_EQUIV          9
#define GU_INVERTED       10
#define GU_OR_REVERSE     11
#define GU_COPY_INVERTED  12
#define GU_OR_INVERTED    13
#define GU_NAND           14
#define GU_SET            15

/*
 * GU texture mapping modes
 */
#define GU_TEXTURE_COORDS   0
#define GU_TEXTURE_MATRIX   1
#define GU_ENVIRONMENT_MAP  2

/*
 * GU texture projection map mode
 */
#define GU_POSITION   0
#define GU_UV         1

/*
 * GU texture functions
 */
#define GU_TFX_MODULATE  0
#define GU_TFX_DECAL     1
#define GU_TFX_BLEND     2
#define GU_TFX_REPLACE   3
#define GU_TFX_ADD       4

#define GU_TCC_RGB   0
#define GU_TCC_RGBA  1

/*
 * GU texture filter
 */
#define GU_NEAREST                0
#define GU_LINEAR                 1
#define GU_NEAREST_MIPMAP_NEAREST 4
#define GU_LINEAR_MIPMAP_NEAREST  5
#define GU_NEAREST_MIPMAP_LINEAR  6
#define GU_LINEAR_MIPMAP_LINEAR   7

/*
 * GU blend operations
 */
#define GU_ADD              0
#define GU_SUBTRACT         1
#define GU_REVERSE_SUBTRACT 2
#define GU_MIN              3
#define GU_MAX              4
#define GU_ABS              5

/*
 * GU blend factors
 */
#define GU_SRC_COLOR           0
#define GU_ONE_MINUS_SRC_COLOR 1
#define GU_SRC_ALPHA           2
#define GU_ONE_MINUS_SRC_ALPHA 3
#define GU_DST_ALPHA           4
#define GU_ONE_MINUS_DST_ALPHA 5
#define GU_DST_COLOR           6
#define GU_ONE_MINUS_DST_COLOR 7
#define GU_FIX                 10

/*
 * GU texture wrap modes
 */
#define GU_REPEAT   0
#define GU_CLAMP    1

/*
 * GU stencil operations
 */
#define GU_KEEP     0
#define GU_ZERO     1
#define GU_REPLACE  2
#define GU_INVERT   3
#define GU_INCR     4
#define GU_DECR     5

/*
 * GU misc constants
 */
#define GU_PI               3.141593f
#define GU_TRUE             1
#define GU_FALSE            0
#define GU_DIRECT           0
#define GU_CALL             1
#define GU_SEND             2
#define GU_TAIL             3
#define GU_HEAD             4
#define GU_NORMALIZED_NORMAL    0
#define GU_NORMAL               1
#define GU_UNKNOWN_LIGHT_COMPONENT  0

/*
 * Kernel functions
 */
void sceKernelExitGame(void);
void sceKernelDelayThread(SceUInt delay);
void sceKernelDcacheWritebackInvalidateAll(void);
int sceKernelSleepThread(void);
SceUID sceKernelLoadModule(const char *path, int flags, SceKernelLMOption *option);
int sceKernelStartModule(SceUID modid, SceSize argsize, void *argp, int *status, SceKernelSMOption *option);
int sceKernelUtilsMd5Digest(u8 *data, u32 size, u8 *digest);

/*
 * Display functions
 */
void sceDisplaySetFrameBuf(void *topaddr, int bufferwidth, int pixelformat, int sync);
int sceDisplaySetMode(int mode, int width, int height);
int sceDisplayWaitVblankStart(void);

/*
 * Controller functions
 */
int ctrlReadBufferPositive(CtrlData *pad_data, int count);
int ctrlSetSamplingMode(int mode);

/*
 * File I/O flags
 */
#define IO_O_RDONLY    0x0001
#define IO_O_WRONLY    0x0002
#define IO_O_RDWR      0x0003
#define IO_O_CREAT     0x0200
#define IO_O_TRUNC     0x0400
#define IO_O_APPEND    0x0100

/*
 * File I/O functions
 */
SceUID sceIoOpen(const char *file, int flags, SceMode mode);
int sceIoRead(SceUID fd, void *data, SceSize size);
int sceIoWrite(SceUID fd, const void *data, SceSize size);
int sceIoIoctl(SceUID fd, unsigned int cmd, void *indata, int inlen, void *outdata, int outlen);
SceUID sceIoDopen(const char *dirname);
int sceIoDread(SceUID fd, SceIoDirent *dir);
int sceIoDclose(SceUID fd);

/*
 * GU functions
 */
void sceGuInit(void);
int sceGuDisplay(int state);
void sceGuStart(int cid, void* list);
int sceGuFinish(void);
int sceGuSync(int mode, int a1);
void* sceGuSwapBuffers(void);
void sceGuClear(int flags);
void sceGuClearColor(unsigned int color);
void sceGuClearDepth(unsigned int depth);
void sceGuDrawBuffer(int psm, void* fbp, int fbw);
void sceGuDispBuffer(int width, int height, void* dispbp, int dispbw);
void sceGuDepthBuffer(void* zbp, int zbw);
void sceGuOffset(unsigned int x, unsigned int y);
void sceGuViewport(int cx, int cy, int width, int height);
void sceGuDepthRange(int near, int far);
void sceGuScissor(int x, int y, int w, int h);
void sceGuEnable(int state);
void sceGuDisable(int state);
void sceGuAlphaFunc(int func, int value, int mask);
void sceGuDepthFunc(int function);
void sceGuFrontFace(int order);
void sceGuShadeModel(int mode);
void sceGuBlendFunc(int op, int src, int dest, unsigned int srcfix, unsigned int destfix);
void sceGuTexMode(int tpsm, int maxmips, int a2, int swizzle);
void sceGuTexFunc(int tfx, int tcc);
void sceGuTexFilter(int min, int mag);
void sceGuTexImage(int mipmap, int width, int height, int tbw, const void* tbp);
void sceGuTexScale(float u, float v);
void sceGuTexOffset(float u, float v);
void sceGuTexEnvColor(unsigned int color);
void sceGuCopyImage(int psm, int sx, int sy, int width, int height, int srcw, void* src, int dx, int dy, int destw, void* dest);
void sceGuDrawArray(int prim, int vtype, int count, const void* indices, const void* vertices);
void* sceGuGetMemory(int size);
void sceGuAmbientColor(unsigned int color);
void sceGuAmbient(int color);
void sceGuLight(int light, int type, int components, const ScePspFVector3* position);
void sceGuLightAtt(int light, float atten0, float atten1, float atten2);
void sceGuLightColor(int light, int component, unsigned int color);
void sceGuLightMode(int mode);
void sceGuLightSpot(int index, const ScePspFVector3* direction, float f12, float f13);

/*
 * GUM functions
 */
void sceGumMatrixMode(int mode);
void sceGumLoadIdentity(void);
void sceGumPerspective(float fovy, float aspect, float near, float far);
void sceGumRotateXYZ(const ScePspFVector3* v);
void sceGumTranslate(const ScePspFVector3* v);
void sceGumDrawArray(int prim, int vtype, int count, const void* indices, const void* vertices);

/*
 * GE functions
 */
int sceGeSaveContext(PspGeContext *context);
int sceGeRestoreContext(const PspGeContext *context);

/*
 * USB functions
 */
#define USB_BUS_DRIVERNAME   "USBBusDriver"
#define USB_STOR_DRIVERNAME  "USBStor"

int sceUsbStart(const char* driverName, int unknown1, int unknown2);
int sceUsbstorBootSetCapacity(u32 size);
int sceUsbActivate(u32 flag);
int sceUsbDeactivate(void);

/*
 * Power functions
 */
int scePowerIsPowerOnline(void);
int scePowerIsBatteryExist(void);
int scePowerIsBatteryCharging(void);
int scePowerGetBatteryChargingStatus(void);
int scePowerIsLowBattery(void);
int scePowerGetBatteryLifePercent(void);
int scePowerGetBatteryLifeTime(void);
int scePowerGetBatteryTemp(void);
int scePowerGetBatteryVolt(void);
int scePowerTick(int type);

/*
 * Frame buffer access (provided by platform implementation)
 */
extern u32* g_vram_base;
Color* getVramDrawBuffer(void);
Color* getVramDisplayBuffer(void);

#ifdef __cplusplus
}
#endif

#endif /* PLATFORM_H */
