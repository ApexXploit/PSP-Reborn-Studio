#ifndef SOUND__H
#define SOUND__H
/*
 * Sound interface for LuaPlayer
 * Uses MikMod for MOD/XM/S3M/IT playback and WAV samples
 */

#include "platform/platform.h"
#include <mikmod.h>

typedef SAMPLE* Sound;
typedef int Voice;

/**
 * Initialize MikMod (sound and music).
 */
extern void initSound(void);

/**
 * Deinitialize MikMod (sound and music)
 */
extern void uninitSound(void);

/**
 * Load a module music file (mod, xm, s3m, it, ...) and play it.
 * Filename must be a full path.
 *
 * @pre filename != NULL
 * @param filename - filename of the music file to load
 * @param loop - whether to loop the music
 */
extern void loadAndPlayMusicFile(char* filename, BOOL loop);

/**
 * Stop and unload music.
 */
extern void stopAndUnloadMusic(void);

/**
 * Load a WAV file. NOTE: MUST BE MONO!
 * Filename must be a full path.
 *
 * @pre filename != NULL
 * @param filename - filename of the wav file to load
 * @return pointer to a new allocated Sound struct, or NULL on failure
 */
extern Sound loadSound(char* filename);

/**
 * Unload the loaded sound
 *
 * @pre handle != NULL
 * @param handle - the loaded wav file
 */
extern void unloadSound(Sound handle);

/**
 * Play sample with volume 255, pan 127.
 *
 * @pre handle != NULL
 * @param handle - Handle of the loaded wav file.
 * @return handle of the currently playing voice
 */
extern Voice playSound(Sound handle);

/**
 * Stop sample.
 *
 * @param handle - Handle of the playing voice.
 */
extern void stopSound(Voice handle);

extern void resumeSound(Voice handle, Sound* soundhandle);

void setSoundLooping(Sound handle, int loopmode, unsigned long loopstart, unsigned long loopend);
unsigned long getSoundLengthInSamples(Sound handle);
unsigned long getSoundSampleSpeed(Sound handle);

/**
 * Sets the volume of the specified voice.
 *
 * @pre vol >= 0 && vol <= 255
 * @param handle - Handle of the currently playing voice.
 * @param vol - Volume (0-255)
 */
extern void setVoiceVolume(Voice handle, UWORD vol);

/**
 * Sets the panning of the specified voice. 127 is middle,
 * 0 is full left and 255 is full right.
 *
 * @pre pan >= 0 && pan <= 255
 * @param handle - Handle of the currently playing voice.
 * @param pan - Panning (0-255)
 */
extern void setVoicePanning(Voice handle, ULONG pan);

/**
 * Sets the speed/frequency of the specified voice.
 *
 * @param handle - Handle of the currently playing voice.
 * @param freq - Frequency in Hz
 */
extern void setVoiceFrequency(Voice handle, ULONG freq);

/**
 * Checks whether music is playing at the moment.
 *
 * @return whether music is playing or not.
 */
extern BOOL musicIsPlaying(void);

/**
 * Checks whether a specific voice is playing at the moment.
 *
 * @param handle - Handle of the currently playing voice.
 * @return whether the voice is playing or not.
 */
extern BOOL voiceIsPlaying(Voice handle);

/**
 * Sets the music volume.
 *
 * @param arg - volume (0-128), or 9999 to query current value
 * @return current volume
 */
extern unsigned setMusicVolume(unsigned arg);

/**
 * Sets the sample/SFX/wav volume.
 *
 * @param arg - volume (0-128), or 9999 to query current value
 * @return current volume
 */
extern unsigned setSFXVolume(unsigned arg);

/**
 * Sets the reverb.
 *
 * @param arg - reverb amount (0-15), or 9999 to query current value
 * @return current reverb
 */
extern unsigned setReverb(unsigned arg);

/**
 * Sets panoramic separation. 0: mono. 128: full separation.
 *
 * @param arg - separation (0-128), or 9999 to query current value
 * @return current separation
 */
extern unsigned setPanSep(unsigned arg);

extern void musicPause(void);
extern void musicResume(void);

#endif
