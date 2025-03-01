# AM62L - Launch UI

This is the TI AML62L Launch UI (out-of-the-box-experience).

To run the TI AM62L target.

```
# Set the LinuxKMS SKIA Software renderer backend
export SLINT_BACKEND=linuxkms-skia-software

# Turn off the console cursor
echo 0 > /sys/class/graphics/fbcon/cursor_blink

# Run the binary and optionally set the MQTT host
./AM62L_Launch_UI --mqtt-host=192.168.1.45
```