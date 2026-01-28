/* Memories definition */
MEMORY
{
   RAM    (xrw)    : ORIGIN = 0x20000000,   LENGTH = 512K
  FLASH    (rx)    : ORIGIN = 0x8000000,   LENGTH = 2048K

    /* QUADSPI peripheral - Ref manual page 78*/
  /*QUADSPI (rx) : ORIGIN = 0xA0001000, LENGTH = 0x00001FFF*/
}

/* stm32f767zit */


