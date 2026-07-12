#include "Batch.h"

// TODO: сделать структуру данных с динамическим расширением:
//       например выделено 100 объектов, начал выделяться 101-ый и массив расширился до 200.
//       такой подход будет с целью оптимизации производительности при массовом добавлении.
//       в данный момент BATCH_MAX=5000, этого базово достаточно
static BatchData batch_data[BATCH_MAX] = { 0 }; // ~180kb
static int batch_lastnum = -1;

static int getNextIndex() {
    if (batch_lastnum >= BATCH_MAX - 1)
        return -1;
    batch_lastnum++;
    return batch_lastnum;
}

static void ClearBatch(int idx) {
    batch_data[idx].type = 0;
    batch_data[idx].img = NULL;
    batch_data[idx].x = 0;
    batch_data[idx].y = 0;
    batch_data[idx].w = 0;
    batch_data[idx].h = 0;
    batch_data[idx].srcx = 0;
    batch_data[idx].srcy = 0;
    batch_data[idx].srcw = 0;
    batch_data[idx].srch = 0;
}

int BatchAddDraw(g2dImage *img, int x, int y, int w, int h, int srcx, int srcy, int srcw, int srch) {
    int idx = getNextIndex();
    if (idx == -1) return -1;

    ClearBatch(idx);
    batch_data[idx].type = BATCH_TYPE_TEX;
    batch_data[idx].img = img;
    batch_data[idx].x = x;
    batch_data[idx].y = y;
    batch_data[idx].w = w;
    batch_data[idx].h = h;
    batch_data[idx].srcx = srcx;
    batch_data[idx].srcy = srcy;
    batch_data[idx].srcw = srcw;
    batch_data[idx].srch = srch;

    return idx;
}

int BatchRender() {
    bool changedTex = true;
    for (size_t i = 0; i <= batch_lastnum; i++) {
        BatchData *data = &batch_data[i];

        if (changedTex) {
            g2dBeginRects(data->img);
            changedTex = false;
        }

        g2dSetTexLinear(false);
        g2dSetTexRepeat(true);
        g2dSetCoordXY(data->x, data->y);
        g2dSetCropXY(data->srcx, data->srcy);
        g2dSetCropWH(data->srcw, data->srch);
        g2dSetScaleWH(data->w, data->h);
        g2dAdd();

        if (i == batch_lastnum || (&batch_data[i + 1])->img != data->img) {
            //если последний объект, либо в следующем объекте текстурка поменяется
            g2dEnd();
            changedTex = true;
        }
    }

    return 1;
}

void BatchClear() {
    batch_lastnum = -1;
}
