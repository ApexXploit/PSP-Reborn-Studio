#include <stdlib.h>
#include <string.h>
#include <mikmod.h>

#include "sound.h"

#define MAX_MUSIC_CHAN 128
#define MAX_SFX_CHAN 32
static MODULE *musichandle = NULL;

// Error handler for MikMod
static void my_mikmod_error_handler(void)
{
	printf("LuaPlayer's Mikmod has a critical error:\n");
	printf("MikMod error: %s\n", MikMod_strerror(MikMod_errno));
}

// For internal use

void initSound(void) {
	MikMod_RegisterErrorHandler(my_mikmod_error_handler);
	MikMod_RegisterAllLoaders();
	MikMod_RegisterAllDrivers();
	md_mode = DMODE_16BITS | DMODE_STEREO | DMODE_SOFT_SNDFX | DMODE_SOFT_MUSIC;
	md_reverb = 0;
	md_pansep = 128;
	md_volume = 128;
	md_musicvolume = 96;
	md_sndfxvolume = 128;
	musichandle = NULL;
	if (MikMod_Init("")) {
		fprintf(stderr, "MikMod init failed: %s\n", MikMod_strerror(MikMod_errno));
		return;
	}
	MikMod_SetNumVoices(MAX_MUSIC_CHAN, MAX_SFX_CHAN);
	MikMod_EnableOutput();
}

void uninitSound(void) {
	if (musichandle) Player_Free(musichandle);
	musichandle = NULL;
	MikMod_DisableOutput();
	MikMod_Exit();
}

// For external Lua use
static unsigned oldvol = 0;

void loadAndPlayMusicFile(char* filename, BOOL loop) {
	if (musichandle) stopAndUnloadMusic();
	if (oldvol) md_musicvolume = oldvol;
	musichandle = Player_Load(filename, MAX_MUSIC_CHAN, 0);
	if (musichandle) {
		musichandle->loop = loop;
		Player_Start(musichandle);
	}
}

void stopAndUnloadMusic(void) {
	Player_Stop();
	if (musichandle) Player_Free(musichandle);
	musichandle = NULL;
}

void musicPause(void) {
	oldvol = md_musicvolume;
	md_musicvolume = 0;
	Player_TogglePause();
}

void musicResume(void) {
	Player_TogglePause();
	md_musicvolume = oldvol;
}


Sound loadSound(char* filename) {
	return Sample_Load(filename);
}

void unloadSound(Sound handle) {
	if (handle) Sample_Free(handle);
}


Voice playSound(Sound handle) {
	if (handle) {
		Voice voice = Sample_Play(handle, 0, 0);
		setVoicePanning(voice, 127);
		setVoiceVolume(voice, 255);
		return voice;
	}
	return 0;
}

void stopSound(Voice handle) {
	Voice_Stop(handle);
}

void resumeSound(Voice handle, Sound* soundhandle) {
	printf("Sample resuming not supported.\n");
}


void setSoundLooping(Sound handle, int loopmode, unsigned long loopstart, unsigned long loopend) {
	if (!handle) return;
	if (loopmode == 0) {
		handle->loopstart = 0;
		handle->loopend = 0;
		handle->flags &= ~SF_LOOP;
	} else if (loopmode == 1) {
		handle->loopstart = 0;
		handle->loopend = handle->length;
		handle->flags |= SF_LOOP;
	} else if (loopmode == 2) {
		handle->loopstart = loopstart;
		handle->loopend = loopend;
		handle->flags |= SF_LOOP;
	}
}

unsigned long getSoundLengthInSamples(Sound handle) {
	if (!handle) return 0;
	return handle->length;
}

unsigned long getSoundSampleSpeed(Sound handle) {
	if (!handle) return 0;
	return handle->speed;
}


void setVoiceVolume(Voice handle, UWORD vol) {
	if (vol > 255) vol = 255;
	Voice_SetVolume(handle, vol);
}

void setVoicePanning(Voice handle, ULONG pan) {
	if (pan > 255) pan = 255;
	Voice_SetPanning(handle, pan);
}

void setVoiceFrequency(Voice handle, ULONG freq) {
	if (freq > 100000) freq = 100000;
	Voice_SetFrequency(handle, freq);
}

BOOL musicIsPlaying(void) {
	return Player_Active();
}

BOOL voiceIsPlaying(Voice handle) {
	return !Voice_Stopped(handle);
}


unsigned setMusicVolume(unsigned arg) {
	if (arg != 9999) {
		if (arg > 128) arg = 128;
		md_musicvolume = arg;
	}
	return md_musicvolume;
}

unsigned setSFXVolume(unsigned arg) {
	if (arg != 9999) {
		if (arg > 128) arg = 128;
		md_sndfxvolume = arg;
	}
	return md_sndfxvolume;
}

unsigned setReverb(unsigned arg) {
	if (arg != 9999) {
		if (arg > 15) arg = 15;
		md_reverb = arg;
	}
	return md_reverb;
}

unsigned setPanSep(unsigned arg) {
	if (arg != 9999) {
		if (arg > 128) arg = 128;
		md_pansep = arg;
	}
	return md_pansep;
}
