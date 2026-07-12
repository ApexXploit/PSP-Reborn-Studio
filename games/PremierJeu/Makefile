TARGET = game
OBJS = src/main.o

CFLAGS = -O2 -G0 -Wall
CXXFLAGS = $(CFLAGS) -std=c++17 -fno-exceptions -fno-rtti
ASFLAGS = $(CFLAGS)

EXTRA_TARGETS = EBOOT.PBP
PSP_EBOOT_TITLE = PremierJeu

PSPSDK := $(shell psp-config --pspsdk-path)
include $(PSPSDK)/lib/build.mak
