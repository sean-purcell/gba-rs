//
// m3_demo.c
// Basic mode 3 drawing routines
//
// (20060104 - 20060104, cearn)

#include "toolbox.h"

int main()
{
	int ii, jj;

	REG_DISPCNT= DCNT_MODE3 | DCNT_BG2;

    m3_line(132+11, 9, 226, 12+7, RGB15(7, 0, 7));
    while(1);


	return 0;
}
