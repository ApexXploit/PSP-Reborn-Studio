/*-----------------------------------------------------------------------------------------------------------------------#
#------------- This file is part of: ------------------------------------------------------------------------------------#
#---█████----------------------------███████████--████------------------------------------------█████-█████-███████████--#
#--░░███----------------------------░░███░░░░░███░░███-----------------------------------------░░███-░░███-░█░░░███░░░█--#
#---░███--------█████-████--██████---░███----░███-░███---██████---█████-████--██████--████████--░░███-███--░---░███--░---#
#---░███-------░░███-░███--░░░░░███--░██████████--░███--░░░░░███-░░███-░███--███░░███░░███░░███--░░█████-------░███------#
#---░███--------░███-░███---███████--░███░░░░░░---░███---███████--░███-░███-░███████--░███-░░░----░░███--------░███------#
#---░███------█-░███-░███--███░░███--░███---------░███--███░░███--░███-░███-░███░░░---░███---------░███--------░███------#
#---███████████-░░████████░░████████-█████--------█████░░████████-░░███████-░░██████--█████--------█████-------█████-----#
#--░░░░░░░░░░░---░░░░░░░░--░░░░░░░░-░░░░░--------░░░░░--░░░░░░░░---░░░░░███--░░░░░░--░░░░░--------░░░░░-------░░░░░------#
#------------------------------------------------------------------███-░███----------------------------------------------#
#-----------------------------------------------------------------░░██████-----------------------------------------------#
#------------------------------------------------------------------░░░░░░------------------------------------------------#
#------------------------------------------------------------------------------------ Made by Kodilo --------------------#
#- Special Thanks to: ---------------------------------------------------------------------------------------------------#
#------------------------------------------------------------------------------------------------------------------------#
#- BenHur for intraFont -------------------------------------------------------------------------------------------------#
#- Geecko for gLib2D ----------------------------------------------------------------------------------------------------#
#- Arshia001 for PSPAALIB -----------------------------------------------------------------------------------------------#
#- jonny & Raphael for PMP Mod ------------------------------------------------------------------------------------------#
#- InsertWittyName & MK2k for PGE source code ---------------------------------------------------------------------------#
#- Rinnegatamante & Nanni for example of awesome Lua Player -------------------------------------------------------------#
#-----------------------------------------------------------------------------------------------------------------------*/

#include "LUA.h"
#include "../libs/include_res/output_wav.c"
#include "../libs/include_res/output_pgf.c"

intraFont *luaFont;

void save_to_file(const char *filename, unsigned char *data, int size) {
    FILE *file = fopen(filename, "wb");
    if (file == NULL) {
        perror("Не удалось открыть файл для записи");
        return;
    }

    fwrite(data, sizeof(unsigned char), size, file);

    fclose(file);
}

int checkFileSum(const char *path, uint32_t size) {
    FILE *f = fopen(path, "rb");
    if (f == NULL) return 1;
    fseek(f, 0, SEEK_END);
    long file_size = ftell(f);
    fseek(f, 0, SEEK_SET);

    uint8_t *file_data = malloc(file_size);
    fread(file_data, 1, file_size, f);

    uint32_t crc = 0xFFFFFFFF;
    const uint8_t *bytes = (const uint8_t *)file_data;
    size_t i, j;
    for (i = 0; i < file_size; i++) {
        crc ^= bytes[i];
        for (j = 0; j < 8; j++) {
            crc = (crc >> 1) ^ (CRC32_POLYNOMIAL & (-(crc & 1)));
        }
    }

    fclose(f);
    free(file_data);

    if (size == ~crc) return 0;
    else return 1;
}

static int LUA_exit(lua_State *L) {
    if (lua_gettop(L) != 0)
        return luaL_error(L, "LUA.exit() takes no arguments");

    pmp_shutdown();
    sceKernelExitGame();
    return 0;
}

static int LUA_delay(lua_State *L) {
    if (lua_gettop(L) != 1)
        return luaL_error(L, "LUA.sleep(ms) takes 1 argument");

    sceKernelDelayThreadCB(luaL_checknumber(L, 1) * 1000);

    return 0;
}

int get_freeRam() {
    void *buf[128];
    int i, j;

    for (i = 0; i < 128; i++) {
        buf[i] = malloc(512 * 1024);
        if (!buf[i])
            break;
    }

    int result = i;

    for (j = result - 1; j >= 0; j--) {
        free(buf[j]);
    }

    return (result * 512 * 1024);
}

static int LUA_freeRAM(lua_State *L) {
    if (lua_gettop(L) != 0)
        return luaL_error(L, "LUA.getRAM() takes no arguments");

    lua_pushnumber(L, get_freeRam());

    return 1;
}

static int LUA_getRand(lua_State *L) {
    if (lua_gettop(L) != 2)
        return luaL_error(L, "LUA.getRandom(min, max) takes 2 arguments");

    srand(time(NULL));
    int min = luaL_checknumber(L, 1);
    int max = luaL_checknumber(L, 2);

    int random = rand() % (max - min + 1) + min;
    lua_pushnumber(L, random);

    return 1;
}

static int LUA_screenshot(lua_State *L) {
    if (lua_gettop(L) != 1 && lua_gettop(L) != 3)
        return luaL_error(L, "LUA.screenshot(path, width?, height?) takes 1 or 3 arguments");

    const char *path = luaL_checkstring(L, 1);
    int w = luaL_optinteger(L, 2, 480);
    int h = luaL_optinteger(L, 3, 272);
    
    g2dScreenshotErrorCode errorCode = g2dScreenshot(path, w, h);

    if (errorCode == G2D_ERR_OPEN)
        return luaL_error(L, "LUA.screenshot: Cannot create/open path file.");
    else if (errorCode == G2D_ERR_WRITESTRUCT)
        return luaL_error(L, "LUA.screenshot: Cannot create file's write structure.");
    else if (errorCode == G2D_ERR_INFOSTRUCT)
        return luaL_error(L, "LUA.screenshot: Cannot create file's info structure.");
    else if (errorCode == G2D_ERR_MEMALLOC)
        return luaL_error(L, "LUA.screenshot: Memory allocation failed.");

    return 0;
}

static int LUA_print(lua_State *L) {
    if (lua_gettop(L) != 3)
        return luaL_error(L, "LUA.print(x, y, text) takes 3 arguments");

    intraFontSetStyle(luaFont, 0.9, WHITE, 0, 0, 0);

    int x = luaL_checkint(L, 1);
    int y = luaL_checkint(L, 2) + intraFontTextHeight(luaFont);
    const char *text = luaL_checkstring(L, 3);

    lua_pushnumber(L, intraFontPrint(luaFont, x, y, text));

    return 1;
}

static const luaL_Reg LUA_methods[] = {
    {"exit",       LUA_exit},
    {"quit",       LUA_exit},
    {"sleep",      LUA_delay},
    {"getRAM",     LUA_freeRAM},
    {"getRandom",  LUA_getRand},
    {"screenshot", LUA_screenshot},
    {"print",      LUA_print},
    {0, 0}
};

int LUA_init(lua_State *L) {
    save_to_file("templuaFont.pgf", LUAFont, size_LUAFont);
    save_to_file("temp_system_sound.wav", system_sound, size_system_sound);

    if (checkFileSum("templuaFont.pgf", 0x48437F2D) == 0 && checkFileSum("temp_system_sound.wav", 0xD78A8334) == 0) {
        luaFont = intraFontLoad("templuaFont.pgf", INTRAFONT_CACHE_LARGE | INTRAFONT_STRING_UTF8);
        if (AalibLoad("temp_system_sound.wav", PSPAALIB_CHANNEL_WAV_32, TRUE) != 0) sceKernelExitGame();

        remove("templuaFont.pgf");
        remove("temp_system_sound.wav");
    } else {
        remove("templuaFont.pgf");
        remove("temp_system_sound.wav");
        sceKernelExitGame();
    }

    luaL_register(L, "LUA", LUA_methods);

    return 0;
}