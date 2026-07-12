#include "lua/common.h"

#define BATCH_MAX 5000

typedef enum
{
    BATCH_TYPE_TEX
} BATCH_TYPES;

typedef struct
{
    char type;
    g2dImage *img;
    int x, y, w, h;
    int srcx, srcy, srcw, srch;
    // 37 bytes
} BatchData;

/// @brief Add texture to batch
/// @return index in batch
int BatchAddDraw(g2dImage *img, int x, int y, int w, int h, int srcx, int srcy, int srcw, int srch);

/// @return 1 on success, 0 on fail
int BatchRender();

void BatchClear();
