#include <stdio.h>
#include <string.h>
#include "../ferris.xpm"


int char2bits[256];

int main() {
    int w, h, n_colors, i, j;
    sscanf(ferris_xpm[0], "%d %d %d", &w, &h, &n_colors);
    for(i=0; i<n_colors; i++) {
        char2bits[ferris_xpm[1+i][0]] = i;
    }
    for(j=0; j<h; j++) {
        char *line = ferris_xpm[j+5];
        for(i=0; i<w; i+=4) {
            int out = (char2bits[line[i]] << 6) | (char2bits[line[i+1]] << 4) | (char2bits[line[i+2]] << 2) | (char2bits[line[i+3]] << 0);
            putchar(out);
        }
    }
}
