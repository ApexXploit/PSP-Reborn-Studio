#ifndef __LUAPLAYER_H
#define __LUAPLAYER_H

#include <stdlib.h>
#include "platform/platform.h"

extern "C" {
#include "lua.h"
#include "lualib.h"
#include "lauxlib.h"
}

#define MAX(a, b) ((a) > (b) ? (a) : (b))
#define CLAMP(val, min, max) ((val)>(max)?(max):((val)<(min)?(min):(val)))

#define UserdataStubs(HANDLE, DATATYPE) \
DATATYPE *to##HANDLE (lua_State *L, int index) \
{ \
  DATATYPE* handle  = (DATATYPE*)lua_touserdata(L, index); \
  if (handle == NULL) luaL_typeerror(L, index, #HANDLE); \
  return handle; \
} \
DATATYPE* push##HANDLE(lua_State *L) { \
	DATATYPE * newvalue = (DATATYPE*)lua_newuserdata(L, sizeof(DATATYPE)); \
	luaL_getmetatable(L, #HANDLE); \
	lua_pushvalue(L, -1); \
	lua_setmetatable(L, -3); \
	lua_pushstring(L, "__index"); \
	lua_getglobal(L, #HANDLE); \
	lua_settable(L, -3); \
	lua_pop(L, 1); \
	return newvalue; \
}

#define UserdataRegister(HANDLE, METHODS, METAMETHODS) \
int HANDLE##_register(lua_State *L) { \
	luaL_newmetatable(L, #HANDLE); \
	lua_pushliteral(L, "__index"); \
	lua_pushvalue(L, -2); \
	lua_rawset(L, -3); \
	\
	luaL_setfuncs(L, METAMETHODS, 0); \
	\
	luaL_newlib(L, METHODS); \
	lua_setglobal(L, #HANDLE); \
	\
	lua_getglobal(L, #HANDLE); \
	luaL_getmetatable(L, #HANDLE); \
	lua_setmetatable(L, -2); \
	lua_pop(L, 1); \
	return 1; \
}

extern const char * runScript(const char* script, bool isStringBuffer);

extern void luaSound_init(lua_State *L);
extern void luaControls_init(lua_State *L);
extern void luaGraphics_init(lua_State *L);
extern void lua3D_init(lua_State *L);
extern void luaTimer_init(lua_State *L);
extern void luaSystem_init(lua_State *L);
extern void luaWlan_init(lua_State *L);

extern void stackDump (lua_State *L);


#endif

