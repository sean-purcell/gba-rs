int main () {
    *(unsigned int *)0x04000000 = 0x0405;

    ((unsigned short*)0x06000000)[120+80*160] = 0x001F;
    ((unsigned short*)0x06000000)[136+80*160] = 0x03E0;
    ((unsigned short*)0x06000000)[120+96*160] = 0x7C00;

    ((unsigned short*)0x0600A000)[120+80*160] = 0x03E0;
    ((unsigned short*)0x0600A000)[136+80*160] = 0x03E0;
    ((unsigned short*)0x0600A000)[120+96*160] = 0x7C00;

    (*(unsigned int *)0x04000000) = 0x0415;

    while(1);



    return 0;
}
