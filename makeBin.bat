arm-none-eabi-objcopy -O ihex target/thumbv7em-none-eabihf/release/testDac testDac.hex
arm-none-eabi-objcopy -O binary --strip-all  target/thumbv7em-none-eabihf/release/testDac testDac.bin

rem arm-none-eabi-readelf -S target/thumbv7em-none-eabihf/release/testDac


arm-none-eabi-objcopy -O ihex target/thumbv7em-none-eabihf/debug/testDac testDac_d.hex
arm-none-eabi-objcopy -O binary --strip-all  target/thumbv7em-none-eabihf/debug/testDac testDac_d.bin



