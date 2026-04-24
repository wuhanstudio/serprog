#!/bin/bash
# Build and flash script for STM32 serprog

set -e

echo "Building STM32 serprog firmware..."
cargo build --release

echo ""
echo "Build successful! Binary location:"
echo "  target/thumbv7m-none-eabi/release/stm32-serprog"
echo ""

# Check if user wants to flash
read -p "Flash to Blue Pill now? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "Choose flashing method:"
    echo "1) probe-rs (ST-Link)"
    echo "2) st-flash (ST-Link)"  
    echo "3) dfu-util (DFU mode)"
    echo "4) Exit without flashing"
    read -p "Enter choice [1-4]: " -n 1 -r choice
    echo
    
    case $choice in
        1)
            echo "Flashing with probe-rs..."
            cargo run --release
            ;;
        2)
            echo "Converting to binary..."
            arm-none-eabi-objcopy -O binary \
                target/thumbv7m-none-eabi/release/stm32-serprog \
                firmware.bin
            
            echo "Flashing with st-flash..."
            st-flash write firmware.bin 0x8000000
            ;;
        3)
            echo "Converting to binary..."
            arm-none-eabi-objcopy -O binary \
                target/thumbv7m-none-eabi/release/stm32-serprog \
                firmware.bin
            
            echo ""
            echo "Put Blue Pill in DFU mode:"
            echo "  1. Set BOOT0 jumper to 1"
            echo "  2. Press RESET button"
            echo "  3. Press Enter to continue"
            read
            
            echo "Flashing with dfu-util..."
            dfu-util -a 0 -s 0x08000000 -D firmware.bin
            
            echo ""
            echo "IMPORTANT: Set BOOT0 jumper back to 0 and press RESET!"
            ;;
        4)
            echo "Exiting without flashing."
            exit 0
            ;;
        *)
            echo "Invalid choice. Exiting."
            exit 1
            ;;
    esac
    
    echo ""
    echo "✓ Flashing complete!"
    echo ""
    echo "Connect Blue Pill via USB and run:"
    echo "  flashrom -p serprog:dev=/dev/ttyACM0:4000000"
else
    echo "Skipping flash. To flash later, run this script again."
fi
