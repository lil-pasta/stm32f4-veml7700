# veml7700 light sensor
Small project to read ambient light with a STM32f4 blackpill and a veml7700 breakout sensor from Adafruit.

The driver itself should be generic, however all of the code in main.rs is specific to the stm32f4xx family. 

## Usage
may have to edit the info in the openocd.cfg and debug.gdb files to make sure everything points to the right place

also the memory.x is specific to the stm32f4.

also, also, you may need to double check your target entery .cargo/config
