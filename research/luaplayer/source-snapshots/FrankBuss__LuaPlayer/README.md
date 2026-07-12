# Lua Player

http://www.luaplayer.org/

## What is Lua Player?

Lua Player is a script player for the Sony PSP. With it, you can run applications written in the language Lua. Writing Lua code is both faster and easier than writing for the PSP directly.

## Install on PSP

Included in the distribution you have downloaded is:
- The LuaPlayer application specific for your PSP version
- Several sample applications

### Installation instructions: 1.5 firmware

Copy the `luaplayer` and `luaplayer%` folders to your `(memorystick)/PSP/GAME/`

### Installation instructions: 1.0 firmware

Copy the `luaplayer` folder to your `(memorystick)/PSP/GAME/`

## Finding and installing LuaPlayer apps and games

There's a list of apps and games available at http://www.luaplayer.org/gallery/

To install them, simply drop the app folder into the `/PSP/GAME/luaplayer/Applications/` folder.

## Writing Lua software for PSP

Lua is an interpreted language. This means that all applications are distributed in source form, which in turn means that you can look at other people's code. Just go to the Applications folder, and then look at the `index.lua` inside the applications. There's also documentation inside the Documentation folder, and a `Samples` folder.

To learn the Lua language, read the Lua book at http://www.lua.org/pil/ and use the resources at the Lua Wiki: http://lua-users.org/wiki/

Feel free to drop by at the LuaPlayer forums at http://forums.ps2dev.org/viewforum.php?f=21

This forum is for LuaPlayer- and Lua on PSP-specific questions and discussions, *not* general Lua questions.

### Function reference

See `doc/` for all functions and libraries that are specific to LuaPlayer.

### Making a Lowser-compatible application

Making your app play nicely with Lowser is very simple:

1. The main script file should be called `index.lua`. Place it, with all its resources in a folder with the application's name (spaces and everything is allowed; make the name nice: "Foobar's Magical Quest", not "foobar_game")
2. Instruct your downloaders to install your game in `memorystick:/PSP/GAME/luaplayer/Applications/`
3. Make your game *exitable*, please. Just make sure that the end of the file is reachable. Your main loop could look something like this:
```lua
while not Controls.read():start() do
    -- your app's code
end
```

### Making your Lua application stand-alone

If you don't want your application to rely on an already-installed LuaPlayer on your end user's PSP, you might want to create a stand-alone version of LuaPlayer bundling only your LuaPlayer application. This standalone version will not include the file browser/application launcher Lowser. It does, however, require that you are familiar with the unix build system, have the latest PSP toolchain installed, and the dependencies listed below.

Put your lua app files (either a Lowser-compatible folder or package, or just loose files with a main `script.lua`) in `standalone/app` folder. Modify the `Makefile.psp-standalone` makefile to match your app name, change `title-icon.png`, and run:

```bash
make -f Makefile.psp-standalone release10  # for firmware 1.0
# or
make -f Makefile.psp-standalone release15  # for firmware 1.5
```

from the luaplayer directory.

### The LuaPlayer startup sequence

When LuaPlayer starts up, it will look for a script to load in the following locations, and in the following order:

1. `./luaplayer/script.lua`
2. `./luaplayer/[Application bundle or package]/index.lua`
3. `./luaplayer/[Application bundle or package]/script.lua`
4. `./luaplayer/System/system.lua`

## Building on Linux

LuaPlayer can be built as a desktop application on Linux for development and testing.

### Required packages (Debian/Ubuntu)

```bash
sudo apt install build-essential cmake \
    liblua5.4-dev libfreetype-dev libpng-dev libjpeg-dev \
    libmikmod-dev libsdl2-dev
```

### Build instructions

```bash
mkdir build
cd build
cmake ..
make
```

The `luaplayer` executable will be created in the build directory. Run it with a Lua script:

```bash
./luaplayer [-scale N] path/to/script.lua
```

Options:
- `-scale N` : Set display scale factor (1-8, default: 2)

### Keyboard controls

| Key | PSP Button |
|-----|------------|
| Arrow keys | D-pad |
| A | Select |
| S | Start |
| Q | L Trigger |
| W | R Trigger |
| R | Triangle |
| F | Circle |
| C | Cross |
| D | Square |
| ESC | Exit |

## Contributing code to the LuaPlayer project

If you want to help out with writing code for the LuaPlayer project, you're reading the right section. Lots of work needs to be done before LP can be considered stable. There's a bunch of undocumented bugs all over the place, for example.

Current maintainers are Frank 'Shine' Buss (fb@frank-buss.de) and Joachim 'Nevyn' Bengtsson (joachimb@gmail.com).

You're welcome to submit code patches, ideas and bug reports to http://forums.ps2dev.org/viewforum.php?f=21

### Dependencies

All the dependencies are available at `svn://svn.ps2dev.org/`

Required to build LuaPlayer:
- A recent version of oopo's PSP toolchain
- zlib, libpng, libjpeg, freetype, liblua, liblualib, mikmodlib

### Acquisition and build

```bash
svn checkout svn://svn.ps2dev.org/pspware/LuaPlayer/
```

Build options:
```bash
make            # Creates the eboot
make kxploit    # Creates luaplayer and luaplayer% folders
make release10  # Entire distribution folder with 1.0 binaries
make release15  # ditto for 1.5
```
