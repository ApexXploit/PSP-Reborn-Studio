/*
 * PSP SDK function stubs for Linux
 * Provides implementations of PSP functions using the platform layer
 */

#include "platform.h"
#include "md5.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <dirent.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <termios.h>
#include <errno.h>

/* External references to frame buffer and controls from platform_linux.cpp */
extern "C" {
    extern u32* g_vram_base;
    extern int g_current_controls;
}

/* Forward declarations */
Color* getVramDrawBuffer(void);
Color* getVramDisplayBuffer(void);
void emuFlipBuffers(void);
void emuWaitVsync(void);
int emuIsRunning(void);

/* Current texture for sceGuTexImage */
static Color* currentTexture = NULL;
static int currentTextureWidth = 0;
static int currentTextureHeight = 0;
static unsigned int clear_color = 0;

/*
 * Kernel functions
 */

void sceKernelExitGame(void)
{
    exit(0);
}

void sceKernelDelayThread(SceUInt delay)
{
    usleep(delay);
}

void sceKernelDcacheWritebackInvalidateAll(void)
{
    /* No-op on desktop */
}

int sceKernelSleepThread(void)
{
    while (1) {
        sleep(1);
    }
    return 0;
}

SceUID sceKernelLoadModule(const char *path, int flags, SceKernelLMOption *option)
{
    (void)path; (void)flags; (void)option;
    return 0;
}

int sceKernelStartModule(SceUID modid, SceSize argsize, void *argp, int *status, SceKernelSMOption *option)
{
    (void)modid; (void)argsize; (void)argp; (void)status; (void)option;
    return 0;
}

int sceKernelUtilsMd5Digest(u8 *data, u32 size, u8 *digest)
{
    MD5_CTX ctx;
    MD5Init(&ctx);
    MD5Update(&ctx, data, size);
    MD5Final(digest, &ctx);
    return 0;
}

/*
 * Display functions
 */

void sceDisplaySetFrameBuf(void *topaddr, int bufferwidth, int pixelformat, int sync)
{
    (void)topaddr; (void)bufferwidth; (void)pixelformat; (void)sync;
}

int sceDisplaySetMode(int mode, int width, int height)
{
    (void)mode; (void)width; (void)height;
    return 0;
}

int sceDisplayWaitVblankStart(void)
{
    /* Check if we should exit */
    if (!emuIsRunning()) {
        exit(0);
    }

    /* Wait for SDL to present the frame */
    emuWaitVsync();

    return 0;
}

/*
 * Controller functions
 */

int ctrlReadBufferPositive(CtrlData *pad_data, int count)
{
    (void)count;
    pad_data->Buttons = g_current_controls;
    pad_data->Lx = 128;
    pad_data->Ly = 128;
    return 0;
}

int ctrlSetSamplingMode(int mode)
{
    (void)mode;
    return 0;
}

/*
 * File I/O functions
 */

SceUID sceIoOpen(const char *file, int flags, SceMode mode)
{
    (void)mode;
    int fd;
    if (strncmp(file, "sio:", 4) == 0) {
        fd = open("/dev/ttyS0", O_RDWR | O_NOCTTY | O_NDELAY);
        if (fd != -1) {
            fcntl(fd, F_SETFL, 0);
            fcntl(fd, F_SETFL, FNDELAY);
        }
    } else {
        fd = open(file, flags);
    }
    return fd;
}

int sceIoRead(SceUID fd, void *data, SceSize size)
{
    return read(fd, data, size);
}

int sceIoWrite(SceUID fd, const void *data, SceSize size)
{
    return write(fd, data, size);
}

int sceIoIoctl(SceUID fd, unsigned int cmd, void *indata, int inlen, void *outdata, int outlen)
{
    (void)inlen; (void)outdata; (void)outlen;

    if (cmd == SIO_IOCTL_SET_BAUD_RATE) {
        int pspbaudrate = *((int*)indata);
        int baudrate;

        switch (pspbaudrate) {
            case 38400: baudrate = B38400; break;
            case 19200: baudrate = B19200; break;
            case 9600:  baudrate = B9600;  break;
            case 4800:  baudrate = B4800;  break;
            case 2400:  baudrate = B2400;  break;
            case 1200:  baudrate = B1200;  break;
            case 600:   baudrate = B600;   break;
            case 300:   baudrate = B300;   break;
            default:    baudrate = B9600;  break;
        }

        struct termios options;
        tcgetattr(fd, &options);
        cfsetispeed(&options, baudrate);
        cfsetospeed(&options, baudrate);
        options.c_cflag |= (CLOCAL | CREAD);
        options.c_cflag &= ~PARENB;
        options.c_cflag &= ~CSTOPB;
        options.c_cflag &= ~CSIZE;
        options.c_cflag |= CS8;
        tcsetattr(fd, TCSANOW, &options);
    }
    return 0;
}

SceUID sceIoDopen(const char *dirname)
{
    DIR* dir;
    if (dirname && strlen(dirname) == 0) {
        dir = opendir(".");
    } else {
        dir = opendir(dirname);
    }
    if (dir != NULL) {
        return (SceUID)(intptr_t)dir;
    }
    return -1;
}

int sceIoDread(SceUID fd, SceIoDirent *dir)
{
    struct stat statbuf;
    struct dirent* ent = readdir((DIR*)(intptr_t)fd);
    if (ent != NULL) {
        strcpy(dir->d_name, ent->d_name);
        stat(dir->d_name, &statbuf);
        dir->d_stat.st_size = statbuf.st_size;
        dir->d_stat.st_attr = 0;
        if (S_ISDIR(statbuf.st_mode)) dir->d_stat.st_attr |= FIO_SO_IFDIR;
        if (S_ISLNK(statbuf.st_mode)) dir->d_stat.st_attr |= FIO_SO_IFLNK;
        if (S_ISREG(statbuf.st_mode)) dir->d_stat.st_attr |= FIO_SO_IFREG;
        return 1;
    }
    return -1;
}

int sceIoDclose(SceUID fd)
{
    if (closedir((DIR*)(intptr_t)fd) == 0)
        return 0;
    return -1;
}

/*
 * GU functions - mostly stubs, some with basic software implementations
 */

void sceGuInit(void) {}
int sceGuDisplay(int state) { (void)state; return 0; }
void sceGuStart(int cid, void* list) { (void)cid; (void)list; }
int sceGuFinish(void) { return 0; }
int sceGuSync(int mode, int a1) { (void)mode; (void)a1; return 0; }

void* sceGuSwapBuffers(void)
{
    emuFlipBuffers();
    return NULL;
}

void sceGuClearColor(unsigned int color)
{
    clear_color = color;
}

void sceGuClear(int flags)
{
    (void)flags;
    Color* dest = getVramDrawBuffer();
    for (int n = 0; n < PLATFORM_LINE_SIZE * PLATFORM_SCREEN_HEIGHT; n++) {
        dest[n] = clear_color;
    }
}

void sceGuClearDepth(unsigned int depth) { (void)depth; }
void sceGuDrawBuffer(int psm, void* fbp, int fbw) { (void)psm; (void)fbp; (void)fbw; }
void sceGuDispBuffer(int width, int height, void* dispbp, int dispbw) { (void)width; (void)height; (void)dispbp; (void)dispbw; }
void sceGuDepthBuffer(void* zbp, int zbw) { (void)zbp; (void)zbw; }
void sceGuOffset(unsigned int x, unsigned int y) { (void)x; (void)y; }
void sceGuViewport(int cx, int cy, int width, int height) { (void)cx; (void)cy; (void)width; (void)height; }
void sceGuDepthRange(int near, int far) { (void)near; (void)far; }
void sceGuScissor(int x, int y, int w, int h) { (void)x; (void)y; (void)w; (void)h; }
void sceGuEnable(int state) { (void)state; }
void sceGuDisable(int state) { (void)state; }
void sceGuAlphaFunc(int func, int value, int mask) { (void)func; (void)value; (void)mask; }
void sceGuDepthFunc(int function) { (void)function; }
void sceGuFrontFace(int order) { (void)order; }
void sceGuShadeModel(int mode) { (void)mode; }
void sceGuBlendFunc(int op, int src, int dest, unsigned int srcfix, unsigned int destfix) { (void)op; (void)src; (void)dest; (void)srcfix; (void)destfix; }
void sceGuTexMode(int tpsm, int maxmips, int a2, int swizzle) { (void)tpsm; (void)maxmips; (void)a2; (void)swizzle; }
void sceGuTexFunc(int tfx, int tcc) { (void)tfx; (void)tcc; }
void sceGuTexFilter(int min, int mag) { (void)min; (void)mag; }

void sceGuTexImage(int mipmap, int width, int height, int tbw, const void* tbp)
{
    (void)mipmap; (void)tbw;
    currentTexture = (Color*)tbp;
    currentTextureWidth = width;
    currentTextureHeight = height;
}

void sceGuTexScale(float u, float v) { (void)u; (void)v; }
void sceGuTexOffset(float u, float v) { (void)u; (void)v; }
void sceGuTexEnvColor(unsigned int color) { (void)color; }

void sceGuCopyImage(int psm, int sx, int sy, int width, int height, int srcw, void* src, int dx, int dy, int destw, void* dest)
{
    (void)psm;
    for (int y = 0; y < height; y++) {
        for (int x = 0; x < width; x++) {
            ((Color*)dest)[x + dx + (y + dy) * destw] = ((Color*)src)[x + sx + (y + sy) * srcw];
        }
    }
}

typedef struct {
    unsigned short u, v;
    short x, y, z;
} Vertex;

void sceGuDrawArray(int prim, int vtype, int count, const void* indices, const void* vertices)
{
    (void)prim; (void)vtype; (void)count; (void)indices;
    Vertex* v = (Vertex*)vertices;
    int sx = v[0].u;
    int sy = v[0].v;
    int dx = v[0].x;
    int dy = v[0].y;
    int width = v[1].x - v[0].x;
    int height = v[1].y - v[0].y;
    Color* dest = getVramDrawBuffer();
    for (int y = 0; y < height; y++) {
        for (int x = 0; x < width; x++) {
            Color color = currentTexture[x + sx + (y + sy) * currentTextureWidth];
            if (color & 0xFF000000) {
                dest[x + dx + (y + dy) * PLATFORM_LINE_SIZE] = color;
            }
        }
    }
}

static char guMemory[1024];
void* sceGuGetMemory(int size) { (void)size; return guMemory; }

void sceGuAmbientColor(unsigned int color) { (void)color; }
void sceGuAmbient(int color) { (void)color; }
void sceGuLight(int light, int type, int components, const ScePspFVector3* position) { (void)light; (void)type; (void)components; (void)position; }
void sceGuLightAtt(int light, float atten0, float atten1, float atten2) { (void)light; (void)atten0; (void)atten1; (void)atten2; }
void sceGuLightColor(int light, int component, unsigned int color) { (void)light; (void)component; (void)color; }
void sceGuLightMode(int mode) { (void)mode; }
void sceGuLightSpot(int index, const ScePspFVector3* direction, float f12, float f13) { (void)index; (void)direction; (void)f12; (void)f13; }

/*
 * GUM functions
 */

void sceGumMatrixMode(int mode) { (void)mode; }
void sceGumLoadIdentity(void) {}
void sceGumPerspective(float fovy, float aspect, float near, float far) { (void)fovy; (void)aspect; (void)near; (void)far; }
void sceGumRotateXYZ(const ScePspFVector3* v) { (void)v; }
void sceGumTranslate(const ScePspFVector3* v) { (void)v; }
void sceGumDrawArray(int prim, int vtype, int count, const void* indices, const void* vertices) { (void)prim; (void)vtype; (void)count; (void)indices; (void)vertices; }

/*
 * GE functions
 */

int sceGeSaveContext(PspGeContext *context) { (void)context; return 0; }
int sceGeRestoreContext(const PspGeContext *context) { (void)context; return 0; }

/*
 * USB functions (stubs)
 */

int sceUsbStart(const char* driverName, int unknown1, int unknown2) { (void)driverName; (void)unknown1; (void)unknown2; return 0; }
int sceUsbstorBootSetCapacity(u32 size) { (void)size; return 0; }
int sceUsbActivate(u32 flag) { (void)flag; return 0; }
int sceUsbDeactivate(void) { return 0; }

/*
 * Power functions
 */

int scePowerIsPowerOnline(void) { return 1; }
int scePowerIsBatteryExist(void) { return 1; }
int scePowerIsBatteryCharging(void) { return 0; }
int scePowerGetBatteryChargingStatus(void) { return 0; }
int scePowerIsLowBattery(void) { return 0; }
int scePowerGetBatteryLifePercent(void) { return 100; }
int scePowerGetBatteryLifeTime(void) { return 0; }
int scePowerGetBatteryTemp(void) { return 25; }
int scePowerGetBatteryVolt(void) { return 4200; }
int scePowerTick(int type) { (void)type; return 0; }

/*
 * Wlan stub (networking not supported on Linux build)
 */
extern "C" {
#include "lua.h"
}
void luaWlan_init(lua_State *L) { (void)L; }

/*
 * PSP module loading stubs (not supported on Linux)
 */
extern "C" {
SceUID psploadlib(const char *name, char *init)
{
    (void)name; (void)init;
    return -1;
}

void **findFunction(SceUID id, const char *library, const char *name)
{
    (void)id; (void)library; (void)name;
    return NULL;
}
}
