#include <stdlib.h>
#include <unistd.h>
#include <string.h>

#include "luaplayer.h"


static lua_State *L;


const char * runScript(const char* script, bool isStringBuffer )
{
	L = luaL_newstate();

	// Standard libraries
	luaL_openlibs(L);

	// Add table.getn for Lua 5.0 compatibility (removed in 5.1+)
	luaL_dostring(L, "table.getn = function(t) return #t end");

	// luasystem.cpp defines our loadlib.
	// luaopen_loadlib(L);

	// Modules
	luaSound_init(L);
	luaControls_init(L);
	luaGraphics_init(L);
	lua3D_init(L);
	luaTimer_init(L);
	luaSystem_init(L);
	luaWlan_init(L);
	
	int s = 0;
	const char * errMsg = NULL;

	if(!isStringBuffer) 
		s = luaL_loadfile(L, script);
	else 
		s = luaL_loadbuffer(L, script, strlen(script), NULL);
		
	if (s == 0) {
		s = lua_pcall(L, 0, LUA_MULTRET, 0);
	}
	if (s) {
		errMsg = lua_tostring(L, -1);
		printf("error: %s\n", lua_tostring(L, -1));
		lua_pop(L, 1); // remove error message
	}
	lua_close(L);
	
	return errMsg;
}
