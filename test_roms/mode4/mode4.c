int main () {
    *(unsigned int *)0x04000000 = 0x0404;

    ((unsigned short*)0x05000000)[0] = 0x7fff;

    ((unsigned short*)0x05000000)[1] = 0x001F;
    ((unsigned short*)0x05000000)[2] = 0x03E0;
    ((unsigned short*)0x05000000)[3] = 0x7C00;

    ((unsigned char*)0x06000000)[120+80*240] = 1;
    ((unsigned char*)0x06000000)[136+80*240] = 2;
    ((unsigned char*)0x06000000)[120+96*240] = 3;

    while(1);



    return 0;
}
