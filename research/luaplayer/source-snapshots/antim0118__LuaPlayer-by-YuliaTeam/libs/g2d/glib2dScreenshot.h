#ifndef GLIB2DSCREENSHOT_H
#define GLIB2DSCREENSHOT_H

#include <stdlib.h>
#include <pspdisplay.h>
#include "glib2d.h"

typedef enum
{
  G2D_ERR_OPEN = -1,
  G2D_ERR_WRITESTRUCT = -2,
  G2D_ERR_INFOSTRUCT = -3,
  G2D_ERR_MEMALLOC = -4,
} g2dScreenshotErrorCode;

int g2dScreenshot(const char *path, int width, int height);

#endif