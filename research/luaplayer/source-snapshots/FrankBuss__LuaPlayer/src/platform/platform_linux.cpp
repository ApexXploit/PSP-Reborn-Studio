/*
 * Linux platform implementation for LuaPlayer
 * Uses SDL2 for display, input, and audio
 */

#include "platform.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <pthread.h>

#include <SDL2/SDL.h>
#include <mikmod.h>

/* Display scale factor (default 2x) */
static int g_scale = 2;

/* Double buffer - Lua draws to back, flips to front */
static Color g_framebuffer[2][PLATFORM_LINE_SIZE * PLATFORM_SCREEN_HEIGHT];
static volatile int g_back_buffer = 0;

/* Render buffer - stable copy for SDL to display */
static Color g_render_buffer[PLATFORM_LINE_SIZE * PLATFORM_SCREEN_HEIGHT];
static pthread_mutex_t g_render_mutex = PTHREAD_MUTEX_INITIALIZER;

/* SDL objects */
static SDL_Window* g_window = NULL;
static SDL_Renderer* g_renderer = NULL;
static SDL_Texture* g_texture = NULL;
static volatile int g_running = 1;

/* Exported for compatibility with existing code */
extern "C" {
    u32* g_vram_base = (u32*)g_framebuffer;
    int g_current_controls = 0;
}

/* Script path for Lua thread */
static const char* g_script_path = NULL;

/* External function from luaplayer.cpp */
extern const char* runScript(const char* script, bool isStringBuffer);

/* External function from sound.cpp */
extern void initSound(void);
extern void uninitSound(void);

/* External function from graphics.cpp */
extern void initGraphics(void);

/* Key mappings - SDL scancode to button */
typedef struct {
    SDL_Scancode scancode;
    int button;
} KeyMapping;

static KeyMapping g_key_mappings[] = {
    { SDL_SCANCODE_A, CTRL_SELECT },
    { SDL_SCANCODE_S, CTRL_START },
    { SDL_SCANCODE_Q, CTRL_LTRIGGER },
    { SDL_SCANCODE_W, CTRL_RTRIGGER },
    { SDL_SCANCODE_R, CTRL_TRIANGLE },
    { SDL_SCANCODE_F, CTRL_CIRCLE },
    { SDL_SCANCODE_C, CTRL_CROSS },
    { SDL_SCANCODE_D, CTRL_SQUARE },
    { SDL_SCANCODE_SPACE, CTRL_HOME },
    { SDL_SCANCODE_UP, CTRL_UP },
    { SDL_SCANCODE_RIGHT, CTRL_RIGHT },
    { SDL_SCANCODE_DOWN, CTRL_DOWN },
    { SDL_SCANCODE_LEFT, CTRL_LEFT },
    { SDL_SCANCODE_UNKNOWN, 0 }
};

/*
 * Graphics buffer access
 */
Color* getVramDrawBuffer(void)
{
    return g_framebuffer[g_back_buffer];
}

Color* getVramDisplayBuffer(void)
{
    return g_framebuffer[1 - g_back_buffer];
}

/*
 * Flip buffers - copy completed frame to render buffer, then swap
 */
void emuFlipBuffers(void)
{
    pthread_mutex_lock(&g_render_mutex);
    memcpy(g_render_buffer, g_framebuffer[g_back_buffer],
           PLATFORM_LINE_SIZE * PLATFORM_SCREEN_HEIGHT * sizeof(Color));
    pthread_mutex_unlock(&g_render_mutex);
    g_back_buffer = 1 - g_back_buffer;
}

/*
 * Wait for vsync - enforce 60Hz timing
 */
void emuWaitVsync(void)
{
    static Uint32 last_time = 0;
    Uint32 now = SDL_GetTicks();
    Uint32 frame_time = 1000 / 60;  /* ~16.67ms for 60Hz */

    if (last_time == 0) {
        last_time = now;
        return;
    }

    Uint32 elapsed = now - last_time;
    if (elapsed < frame_time) {
        SDL_Delay(frame_time - elapsed);
    }
    last_time = SDL_GetTicks();
}

/*
 * Check if application is still running
 */
int emuIsRunning(void)
{
    return g_running;
}

/*
 * Process SDL events (called from main loop)
 */
static void processEvents(void)
{
    SDL_Event event;
    while (SDL_PollEvent(&event)) {
        switch (event.type) {
        case SDL_QUIT:
            g_running = 0;
            break;
        case SDL_KEYDOWN:
            if (event.key.keysym.scancode == SDL_SCANCODE_ESCAPE) {
                g_running = 0;
            } else {
                for (int i = 0; g_key_mappings[i].scancode != SDL_SCANCODE_UNKNOWN; i++) {
                    if (g_key_mappings[i].scancode == event.key.keysym.scancode) {
                        g_current_controls |= g_key_mappings[i].button;
                        break;
                    }
                }
            }
            break;
        case SDL_KEYUP:
            for (int i = 0; g_key_mappings[i].scancode != SDL_SCANCODE_UNKNOWN; i++) {
                if (g_key_mappings[i].scancode == event.key.keysym.scancode) {
                    g_current_controls &= ~g_key_mappings[i].button;
                    break;
                }
            }
            break;
        }
    }
}

/*
 * Render the framebuffer to screen
 */
static void renderFrame(void)
{
    void* pixels;
    int pitch;
    if (SDL_LockTexture(g_texture, NULL, &pixels, &pitch) == 0) {
        pthread_mutex_lock(&g_render_mutex);
        for (int y = 0; y < PLATFORM_SCREEN_HEIGHT; y++) {
            memcpy((u8*)pixels + y * pitch,
                   g_render_buffer + y * PLATFORM_LINE_SIZE,
                   PLATFORM_SCREEN_WIDTH * sizeof(Color));
        }
        pthread_mutex_unlock(&g_render_mutex);
        SDL_UnlockTexture(g_texture);
    }

    SDL_RenderCopy(g_renderer, g_texture, NULL, NULL);
    SDL_RenderPresent(g_renderer);
}

/* Lua thread */
static void* luaThread(void* arg)
{
    const char* script = (const char*)arg;
    runScript(script, false);
    uninitSound();
    g_running = 0;
    return NULL;
}

/*
 * Main entry point for Linux
 */
int main(int argc, char** argv)
{
    if (argc < 2) {
        printf("usage: luaplayer [-scale N] script.lua\n");
        printf("  -scale N  : Set display scale factor (default: 2)\n");
        return 1;
    }

    /* Parse arguments */
    int arg_idx = 1;
    while (arg_idx < argc && argv[arg_idx][0] == '-') {
        if (strcmp(argv[arg_idx], "-scale") == 0 && arg_idx + 1 < argc) {
            g_scale = atoi(argv[arg_idx + 1]);
            if (g_scale < 1) g_scale = 1;
            if (g_scale > 8) g_scale = 8;
            arg_idx += 2;
        } else {
            fprintf(stderr, "Unknown option: %s\n", argv[arg_idx]);
            return 1;
        }
    }

    if (arg_idx >= argc) {
        fprintf(stderr, "No script file specified\n");
        return 1;
    }

    g_script_path = argv[arg_idx];
    memset(g_framebuffer, 0, sizeof(g_framebuffer));

    /* Initialize SDL */
    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO) < 0) {
        fprintf(stderr, "SDL_Init failed: %s\n", SDL_GetError());
        return 1;
    }

    /* Create window with scaled size */
    g_window = SDL_CreateWindow("Lua Player",
        SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED,
        PLATFORM_SCREEN_WIDTH * g_scale, PLATFORM_SCREEN_HEIGHT * g_scale,
        SDL_WINDOW_SHOWN);
    if (!g_window) {
        fprintf(stderr, "SDL_CreateWindow failed: %s\n", SDL_GetError());
        SDL_Quit();
        return 1;
    }

    /* Create renderer with vsync */
    g_renderer = SDL_CreateRenderer(g_window, -1,
        SDL_RENDERER_ACCELERATED | SDL_RENDERER_PRESENTVSYNC);
    if (!g_renderer) {
        fprintf(stderr, "SDL_CreateRenderer failed: %s\n", SDL_GetError());
        SDL_DestroyWindow(g_window);
        SDL_Quit();
        return 1;
    }

    /* Set logical size for proper scaling */
    SDL_RenderSetLogicalSize(g_renderer, PLATFORM_SCREEN_WIDTH, PLATFORM_SCREEN_HEIGHT);

    /* Create texture for framebuffer - ABGR8888 matches PSP's native format */
    g_texture = SDL_CreateTexture(g_renderer,
        SDL_PIXELFORMAT_ABGR8888,
        SDL_TEXTUREACCESS_STREAMING,
        PLATFORM_SCREEN_WIDTH, PLATFORM_SCREEN_HEIGHT);
    if (!g_texture) {
        fprintf(stderr, "SDL_CreateTexture failed: %s\n", SDL_GetError());
        SDL_DestroyRenderer(g_renderer);
        SDL_DestroyWindow(g_window);
        SDL_Quit();
        return 1;
    }

    /* Initialize sound */
    initSound();

    /* Initialize graphics */
    initGraphics();

    /* Start Lua thread */
    pthread_t thread;
    pthread_create(&thread, NULL, luaThread, (void*)g_script_path);

    /* Main loop - vsync handles timing */
    while (g_running) {
        processEvents();
        MikMod_Update();
        renderFrame();
    }

    /* Signal thread to exit and wait */
    g_running = 0;
    pthread_join(thread, NULL);

    SDL_DestroyTexture(g_texture);
    SDL_DestroyRenderer(g_renderer);
    SDL_DestroyWindow(g_window);
    SDL_Quit();

    return 0;
}
