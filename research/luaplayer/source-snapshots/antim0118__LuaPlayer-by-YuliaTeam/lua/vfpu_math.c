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

#include "vfpu_math.h"

static int MATH_ACOSF(lua_State *L) {
    lua_pushnumber(L, vfpu_acosf((float)lua_tonumber(L, 1)));
    return 1;
}

static int MATH_ASINF(lua_State *L) {
    lua_pushnumber(L, vfpu_asinf((float)lua_tonumber(L, 1)));
    return 1;
}

static int MATH_ATAN2F(lua_State *L) {
    lua_pushnumber(L, vfpu_atan2f((float)lua_tonumber(L, 1), (float)lua_tonumber(L, 2)));
    return 1;
}

static int MATH_ATANF(lua_State *L) {
    lua_pushnumber(L, vfpu_atanf((float)lua_tonumber(L, 1)));
    return 1;
}

static int MATH_COSHF(lua_State *L) {
    lua_pushnumber(L, vfpu_coshf((float)lua_tonumber(L, 1)));
    return 1;
}

static int MATH_COSF(lua_State *L) {
    lua_pushnumber(L, vfpu_cosf((float)lua_tonumber(L, 1)));
    return 1;
}

static int MATH_EXPF(lua_State *L) {
    lua_pushnumber(L, vfpu_expf((float)lua_tonumber(L, 1)));
    return 1;
}

static int MATH_FMODF(lua_State *L) {
    lua_pushnumber(L, vfpu_fmodf((float)lua_tonumber(L, 1), (float)lua_tonumber(L, 2)));
    return 1;
}

static int MATH_LOGF(lua_State *L) {
    lua_pushnumber(L, vfpu_logf((float)lua_tonumber(L, 1)));
    return 1;
}

static int MATH_POWF(lua_State *L) {
    lua_pushnumber(L, vfpu_powf((float)lua_tonumber(L, 1), (float)lua_tonumber(L, 2)));
    return 1;
}

static int MATH_RANDOM(lua_State *L) {
    lua_Number r = (lua_Number)(vfpu_randf(0, RAND_MAX)) / (lua_Number)RAND_MAX;
    switch (lua_gettop(L)) {  /* check number of arguments */
    case 0: {  /* no arguments */
        lua_pushnumber(L, r);  /* Number between 0 and 1 */
        break;
    }
    case 1: {  /* only upper limit */
        int u = luaL_checkint(L, 1);
        luaL_argcheck(L, 1 <= u, 1, "interval is empty");
        lua_pushnumber(L, floor(r * u) + 1);  /* int between 1 and `u' */
        break;
    }
    case 2: {  /* lower and upper limits */
        int l = luaL_checkint(L, 1);
        int u = luaL_checkint(L, 2);
        luaL_argcheck(L, l <= u, 2, "interval is empty");
        lua_pushnumber(L, floor(r * (u - l + 1)) + l);  /* int between `l' and `u' */
        break;
    }
    default: return luaL_error(L, "wrong number of arguments");
    }
    return 1;
}

static int MATH_RANDOMSEED(lua_State *L) {
    vfpu_srand(luaL_checkint(L, 1));
    return 0;
}

static int MATH_SINHF(lua_State *L) {
    lua_pushnumber(L, vfpu_sinhf((float)lua_tonumber(L, 1)));
    return 1;
}

static int MATH_SINF(lua_State *L) {
    lua_pushnumber(L, vfpu_sinf((float)lua_tonumber(L, 1)));
    return 1;
}

static int MATH_TANHF(lua_State *L) {
    lua_pushnumber(L, vfpu_tanhf((float)lua_tonumber(L, 1)));
    return 1;
}

static int MATH_TANF(lua_State *L) {
    lua_pushnumber(L, vfpu_tanf((float)lua_tonumber(L, 1)));
    return 1;
}

static const luaL_Reg VFPU_methods[] = {
    {"acos", MATH_ACOSF},
    {"asin", MATH_ASINF},
    {"atan2", MATH_ATAN2F},
    {"atan", MATH_ATANF},
    {"cosh", MATH_COSHF},
    {"cos", MATH_COSF},
    {"exp", MATH_EXPF},
    {"fmod", MATH_FMODF},
    {"log", MATH_LOGF},
    {"pow", MATH_POWF},
    {"random", MATH_RANDOM},
    {"randomseed", MATH_RANDOMSEED},
    {"sinh", MATH_SINHF},
    {"sin", MATH_SINF},
    {"tanh", MATH_TANHF},
    {"tan", MATH_TANF},
    {0,0}
};

int VFPU_init(lua_State *L) {
    luaL_register(L, "math", VFPU_methods);

    return 1;
}