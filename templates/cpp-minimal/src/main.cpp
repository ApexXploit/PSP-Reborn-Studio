#include <pspkernel.h>
#include <pspdebug.h>
#include <pspctrl.h>
#include <pspdisplay.h>

PSP_MODULE_INFO("PSP Reborn Game", PSP_MODULE_USER, 1, 0);
PSP_MAIN_THREAD_ATTR(PSP_THREAD_ATTR_USER | PSP_THREAD_ATTR_VFPU);

static int exitCallback(int, int, void*) { sceKernelExitGame(); return 0; }

static int callbackThread(SceSize, void*) {
    int callback = sceKernelCreateCallback("Exit Callback", exitCallback, nullptr);
    sceKernelRegisterExitCallback(callback);
    sceKernelSleepThreadCB();
    return 0;
}

static void setupCallbacks() {
    int thread = sceKernelCreateThread("Callback Thread", callbackThread, 0x11, 0xFA0, PSP_THREAD_ATTR_USER, nullptr);
    if (thread >= 0) sceKernelStartThread(thread, 0, nullptr);
}

int main() {
    setupCallbacks();
    pspDebugScreenInit();
    sceCtrlSetSamplingCycle(0);
    sceCtrlSetSamplingMode(PSP_CTRL_MODE_ANALOG);
    pspDebugScreenPrintf("PSP Reborn Studio\n\n");
    pspDebugScreenPrintf("Ton premier jeu C++ PSP fonctionne.\n");
    pspDebugScreenPrintf("Appuie sur X pour changer le message.\n");

    bool pressed = false;
    while (true) {
        SceCtrlData pad{};
        sceCtrlPeekBufferPositive(&pad, 1);
        if ((pad.Buttons & PSP_CTRL_CROSS) && !pressed) {
            pspDebugScreenPrintf("\nBonjour depuis le bouton Croix !\n");
            pressed = true;
        } else if (!(pad.Buttons & PSP_CTRL_CROSS)) pressed = false;
        sceDisplayWaitVblankStart();
    }
}
